//! Compiles and evaluates snippets of program text.

use dada_brew::prelude::*;
use dada_collections::Map;
use dada_execute::kernel::Kernel;
use dada_execute::machine::op::{MachineOp, MachineOpExtMut};
use dada_execute::machine::Machine;
use dada_ir::code::syntax::Expr;
use dada_ir::diagnostic::Diagnostic;
use dada_ir::filename::Filename;
use dada_ir::span::{FileSpan, Span};
use dada_ir::token::Token;
use dada_lex::prelude::*;
use dada_parse::prelude::*;
use salsa::DebugWithDb;
use std::collections::HashSet;

pub struct Evaluator<'me> {
    db: &'me mut dada_db::Db,
    kernel: &'me mut dyn Kernel,
    filename: Filename,
    eval_state: EvalState,
    seen_diagnostics: HashSet<Diagnostic>,
}

#[derive(Clone)]
struct EvalState {
    machine: Machine,
    source_parts: SourceParts,
}

pub struct Suggestion {
    pub error: eyre::Report,
    pub suggestion: &'static str,
}

impl<'me> Evaluator<'me> {
    pub fn new(db: &'me mut dada_db::Db, kernel: &'me mut dyn Kernel) -> Evaluator<'me> {
        let filename = Filename::from(db, "<repl-input>");
        let eval_state = EvalState {
            machine: Machine::default(),
            source_parts: SourceParts::default(),
        };
        let seen_diagnostics = HashSet::default();

        let mut evaluator = Evaluator {
            db,
            kernel,
            filename,
            eval_state,
            seen_diagnostics,
        };

        evaluator.prep_machine();

        evaluator
    }

    fn prep_machine(&mut self) {
        assert!(self.eval_state.machine.stack.frames.len() == 0);

        let module_source = self.eval_state.source_parts.create_source();

        self.db.update_file(self.filename, module_source.text);

        let repl_fn = self
            .db
            .function_named(self.filename, &module_source.next_expr_fn_name)
            .expect("repl fn");
        let bir = repl_fn.brew(self.db);

        let nil_prev_result_value =
            (&mut self.eval_state.machine as &mut dyn MachineOp).our_value(());
        let arguments = vec![nil_prev_result_value];
        self.eval_state.machine.push_frame(self.db, bir, arguments);
    }

    pub fn add_item(&mut self, name: ItemName, text: ItemText) -> eyre::Result<()> {
        let state_backup = self.eval_state.clone();
        let res: eyre::Result<()> = try {
            self.eval_state.source_parts.add_item(name, text)?;

            let module_source = self.eval_state.source_parts.create_source();

            self.db.update_file(self.filename, module_source.text);
            self.deduplicate_and_log_diagnostics()?;
        };

        if res.is_err() {
            self.eval_state = state_backup;
        }

        res
    }

    pub async fn eval_expr(&mut self, text: ExprText) -> eyre::Result<Option<Suggestion>> {
        let res = self.eval_expr_maybe_binding(text.clone(), None).await;

        match res {
            Ok(()) => Ok(None),
            Err(e) => {
                self.get_repl_suggestions(text, e)
            }
        }
    }

    pub async fn eval_binding_expr(&mut self, name: String, text: ExprText) -> eyre::Result<()> {
        self.eval_expr_maybe_binding(text, Some(name)).await
    }

    async fn eval_expr_maybe_binding(
        &mut self,
        text: ExprText,
        new_binding: Option<String>,
    ) -> eyre::Result<()> {
        let state_backup = self.eval_state.clone();
        let res: eyre::Result<()> = try {
            self.eval_state.source_parts.add_expr_fn(text, new_binding);

            let module_source = self.eval_state.source_parts.create_source();

            self.db.update_file(self.filename, module_source.text);
            self.deduplicate_and_log_diagnostics()?;

            self.replace_top_frame_with(&module_source.this_expr_fn_name.expect("fn"));

            let res = dada_execute::interpret_until_for_repl(
                self.db,
                self.kernel,
                &mut self.eval_state.machine,
                &module_source.next_expr_fn_name,
            )
            .await?;
        };

        if res.is_err() {
            self.eval_state = state_backup;
        }

        res
    }

    fn deduplicate_and_log_diagnostics(&mut self) -> eyre::Result<()> {
        let diagnostics = self.db.diagnostics(self.filename);
        let mut new_diagnostics = 0_usize;
        for diagnostic in diagnostics {
            if !self.seen_diagnostics.contains(&diagnostic) {
                dada_error_format::print_diagnostic(self.db, &diagnostic)?;
                self.seen_diagnostics.insert(diagnostic);
                new_diagnostics = new_diagnostics.checked_add(1).expect("overflow");
            }
        }

        if new_diagnostics > 0 {
            Err(eyre::eyre!("compilation failed"))?;
        }

        Ok(())
    }

    fn replace_top_frame_with(&mut self, fn_name: &str) {
        let repl_fn = self
            .db
            .function_named(self.filename, fn_name)
            .expect("repl fn");

        let bir = repl_fn.brew(self.db);

        let bir_data = bir.data(self.db);
        let num_arguments = bir_data.num_parameters;

        let top_frame = self.eval_state.machine.stack.frames.pop().expect("frame");
        let prev_arguments = top_frame.locals.into_iter().take(num_arguments);
        let arguments: Vec<_> = prev_arguments.collect();

        self.eval_state.machine.push_frame(self.db, bir, arguments);
    }

    pub fn get_source(&self) -> String {
        self.eval_state.source_parts.create_source().text
    }

    fn get_repl_suggestions(&self, text: ExprText, e: eyre::Report) -> eyre::Result<Option<Suggestion>> {
        let text = text.trim();
        let suggestion = match text {
            "help" | "help()" => {
                Some("maybe you meant to type `:help`")
            }
            "exit" | "exit()" | "quit" | "quit()" => {
                Some("maybe you meant to type `:exit`")
            }
            _ => None,
        };
        match suggestion {
            Some(suggestion) => {
                Ok(Some(Suggestion {
                    error: e,
                    suggestion,
                }))
            }
            None => {
                Err(e)
            }
        }
    }
}

/// Maintains snippets of source text read from the repl.
///
/// Every time the repl is evaluated the full source is rebuilt from these
/// pieces, but is done so in a way that the compiler will do minimal work
/// actually compiling and re-evaluating.
///
/// TODO describe the output
///
/// There are a few rules to follow here:
///
/// - No item name is ever overwritten. Doing so would cause functions to be
///   recompiled and invalidate types that may already exist in the environment.
/// - The expr_fns list only grows, with each calling the next, creating an
///   ever growing call stack during successive evaluations.
/// - Duplicated binding names are overwritten - in the final source they
///   will result in shadowed variables.
#[derive(Clone, Default)]
struct SourceParts {
    items: Map<ItemName, ItemText>,
    item_indexes: Vec<ItemName>,
    expr_fns: Vec<(ItemName, FnText)>,
    binding_names: Vec<String>,
}

type ItemName = String;
type ItemText = String;
type FnText = String;
type ExprText = String;

struct ReplSource {
    text: String,
    this_expr_fn_name: Option<String>,
    next_expr_fn_name: String,
}

impl SourceParts {
    fn add_item(&mut self, name: ItemName, text: ItemText) -> eyre::Result<()> {
        if self.items.contains_key(&name) {
            eyre::bail!("redefining items not yet supported");
        }
        self.item_indexes.push(name.clone());
        self.items.insert(name, text);

        Ok(())
    }

    /// Add an expression and wrap it in an appropriate function definition.
    ///
    /// If `binding_name` is `Some` then this expression produces a new variable
    /// that needs to be propagated into the environment of future repl evaluations.
    fn add_expr_fn(&mut self, expr: ExprText, binding_name: Option<String>) {
        let this_expr_number = self.expr_fns.len();
        let next_expr_number = this_expr_number.checked_add(1).expect("overflow");
        let (this_arg_list, next_arg_list) = self.make_arg_lists(binding_name.clone());
        let this_expr_fn_name = format!("__repl_expr_{}", this_expr_number);
        let next_expr_fn_name = format!("__repl_expr_{}", next_expr_number);
        let this_expr_fn = if binding_name.is_none() {
            format!(
                "async fn {this_expr_fn_name}({this_arg_list}) {{\n\
                     __repl_result =\n\
                     {expr}\n\
                     {next_expr_fn_name}({next_arg_list}).await\n\
                 }}"
            )
        } else {
            format!(
                "async fn {this_expr_fn_name}({this_arg_list}) {{\n\
                     {expr}\n\
                     __repl_result = ()\n\
                     {next_expr_fn_name}({next_arg_list}).await\n\
                 }}"
            )
        };

        self.expr_fns.push((this_expr_fn_name, this_expr_fn));
        if let Some(binding_name) = binding_name {
            self.binding_names.push(binding_name);
        }
    }

    fn make_arg_lists(&self, new_binding: Option<String>) -> (String, String) {
        let this_arg_list = {
            let mut this_arg_list = String::new();
            let arg1 = "__repl_result".to_string();
            this_arg_list.push_str(&arg1);
            for name in &self.binding_names {
                this_arg_list.push_str(", ");
                this_arg_list.push_str(&name);
            }
            this_arg_list
        };

        let next_arg_list = match new_binding {
            Some(new) => {
                let mut next_arg_list = this_arg_list.clone();
                next_arg_list.push_str(", ");
                next_arg_list.push_str(&new);
                next_arg_list
            }
            None => this_arg_list.clone(),
        };

        (this_arg_list, next_arg_list)
    }

    fn create_source(&self) -> ReplSource {
        let mut source = String::new();
        for name in &self.item_indexes {
            let item = self.items.get(name).expect("source");
            source.push_str(&item);
            source.push_str("\n\n");
        }

        for (_, expr_fn) in &self.expr_fns {
            source.push_str(expr_fn);
            source.push_str("\n\n");
        }

        let next_expr_number = self.expr_fns.len();
        let next_expr_fn_name = format!("__repl_expr_{}", next_expr_number);
        let (next_arg_list, _) = self.make_arg_lists(None);
        let next_expr_fn_stub = format!("async fn {next_expr_fn_name}({next_arg_list}) {{ }}",);

        source.push_str(&next_expr_fn_stub);

        let this_expr_fn_name = self.expr_fns.last().map(|(n, _)| n).cloned();

        ReplSource {
            text: source,
            this_expr_fn_name,
            next_expr_fn_name,
        }
    }
}
