//! Figures out what to do with a line of repl input.

use dada_ir::class::Class;
use dada_ir::code::syntax::{Expr, ExprData};
use dada_ir::filename::Filename;
use dada_ir::function::Function;
use dada_ir::in_ir_db::InIrDbExt;
use dada_ir::item::Item;
use dada_ir::span::{FileSpan, Span};
use dada_ir::token::Token;
use dada_ir::Db;
use dada_lex::prelude::*;
use dada_parse::prelude::*;
use salsa::DebugWithDb;

/// The result of calling `Reader::step`.
///
/// This instructs the driver on what do next, whether
/// to evaluate some code, or take some control action.
pub enum Step {
    ReadMore,
    EvalExpr(String),
    EvalBindingExpr { name: String, text: String },
    AddItem { name: String, text: String },
    ExecCommand(Command),
}

/// A repl-specific command of the form `:command`
pub enum Command {
    /// Exit the repl.
    Exit,
    /// Reset the repl state, but not the line input history.
    Reset,
    /// Print a help message.
    Help,
    /// Load a file and replay it into the repl.
    Load(String),
    /// Print the current source code for the repl module
    DumpSource,
}

pub struct Reader {
    /// Holds multiline input.
    buffer: String,
}

impl Reader {
    pub fn new() -> Reader {
        Reader {
            buffer: String::new(),
        }
    }

    pub fn doing_multiline(&self) -> bool {
        !self.buffer.is_empty()
    }

    /// Called when the user enters Ctrl-D.
    ///
    /// Breaks out of multiline input.
    pub fn interrupt(&mut self) {
        self.buffer.clear();
    }

    /// Decides what to do with a line of input.
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn step(&mut self, line: String) -> eyre::Result<Step> {
        let mut text = std::mem::replace(&mut self.buffer, String::new());
        if !text.is_empty() {
            text.push_str("\n");
        }
        text.push_str(&line);

        let thing = try_parse_thing(text)?;

        match thing {
            ParsedThing::Whitespace => Ok(Step::ReadMore),
            ParsedThing::Comment(_) => Ok(Step::ReadMore),
            ParsedThing::ReplCommand(command) => Ok(Step::ExecCommand(command)),
            ParsedThing::OpenTokenTree(text) => {
                self.buffer = text;
                Ok(Step::ReadMore)
            }
            ParsedThing::Expr(text) => Ok(Step::EvalExpr(text)),
            ParsedThing::BindingExpr { name, text } => Ok(Step::EvalBindingExpr { name, text }),
            ParsedThing::Item { name, text } => Ok(Step::AddItem { name, text }),
        }
    }
}

enum ParsedThing {
    Whitespace,
    Comment(String),
    ReplCommand(Command),
    OpenTokenTree(String),
    Expr(String),
    BindingExpr { name: String, text: String },
    Item { name: String, text: String },
}

#[tracing::instrument(level = "debug")]
fn try_parse_thing(text: String) -> eyre::Result<ParsedThing> {
    let input_type = determine_input_type(&text);
    match input_type {
        InputType::Whitespace => Ok(ParsedThing::Whitespace),
        InputType::Comment => Ok(ParsedThing::Comment(text)),
        InputType::ReplCommand => parse_repl_command(&text).map(ParsedThing::ReplCommand),
        InputType::OpenTokenTree => Ok(ParsedThing::OpenTokenTree(text)),
        InputType::Expr => Ok(ParsedThing::Expr(text)),
        InputType::BindingExpr(name) => Ok(ParsedThing::BindingExpr { name, text }),
        InputType::Items => parse_item(&text).map(|(name, text)| ParsedThing::Item { name, text }),
        InputType::Unknown => Err(eyre::eyre!("unrecognized input type")),
    }
}

enum InputType {
    Whitespace,
    Comment,
    ReplCommand,
    OpenTokenTree,
    Expr,
    BindingExpr(String),
    Items,
    Unknown,
}

#[tracing::instrument(level = "debug")]
fn determine_input_type(text: &str) -> InputType {
    if is_whitespace(text) {
        InputType::Whitespace
    } else if is_comment(text) {
        InputType::Comment
    } else if is_repl_command(text) {
        InputType::ReplCommand
    } else if is_open_token_tree(text) {
        InputType::OpenTokenTree
    } else if let Some(input_type) = is_expr(text) {
        input_type
    } else if is_items(text) {
        InputType::Items
    } else {
        InputType::Unknown
    }
}

#[tracing::instrument(level = "debug")]
fn is_whitespace(text: &str) -> bool {
    text.chars().all(char::is_whitespace)
}

#[tracing::instrument(level = "debug")]
fn is_comment(text: &str) -> bool {
    text.trim().chars().next() == Some('#')
}

#[tracing::instrument(level = "debug")]
fn is_repl_command(text: &str) -> bool {
    text.trim().starts_with(":")
}

#[tracing::instrument(level = "debug")]
fn is_open_token_tree(text: &str) -> bool {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "<repl-input>");
    db.update_file(filename, text.into());

    let tt = dada_lex::lex_file(&db, filename);

    let mut tokens = tt.tokens(&db).iter();
    while let Some(token) = tokens.next() {
        if let Token::Delimiter(opening) = token {
            loop {
                let next = tokens.next();
                match next {
                    Some(Token::Delimiter(closing)) => {
                        if *closing == dada_lex::closing_delimiter(*opening) {
                            break;
                        }
                    }
                    None => {
                        return true;
                    }
                    _ => {
                        // pass
                    }
                }
            }
        }
    }

    false
}

#[tracing::instrument(level = "debug")]
fn is_expr(text: &str) -> Option<InputType> {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "<repl-input>");
    db.update_file(filename, text.into());

    let tt = dada_lex::lex_file(&db, filename);
    let expr_tree = dada_parse::code_parser::parse_repl_expr(&db, tt);

    if let Some(expr_tree) = expr_tree {
        let expr_tree_data = expr_tree.data(&db);
        let root_expr = expr_tree_data.root_expr;
        let root_expr_data = &expr_tree_data.tables[root_expr];

        match root_expr_data {
            ExprData::Var(decl, rhs) => {
                let local_decl = &expr_tree_data.tables[*decl];
                let name = local_decl.name.data(&db).string.clone();
                Some(InputType::BindingExpr(name))
            }
            _ => Some(InputType::Expr),
        }
    } else {
        None
    }
}

#[tracing::instrument(level = "debug")]
fn is_items(text: &str) -> bool {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "<repl-input>");
    db.update_file(filename, text.into());

    let items = filename.items(&db);

    items.len() > 0
}

fn parse_repl_command(text: &str) -> eyre::Result<Command> {
    let (first, mut rest) = {
        let mut parts = text.split_whitespace();
        let first = parts.next();
        (first, parts)
    };
    match first {
        Some(":help") => Ok(Command::Help),
        Some(":exit") => Ok(Command::Exit),
        Some(":reset") => Ok(Command::Reset),
        Some(":load") => {
            let (path, none) = (rest.next(), rest.next());
            match (path, none) {
                (Some(path), None) => Ok(Command::Load(path.into())),
                _ => Err(eyre::eyre!(":load takes one argument: `path`")),
            }
        }
        Some(":dump-source") => Ok(Command::DumpSource),
        Some(cmd) => Err(eyre::eyre!("unknown repl command `{}`", cmd)),
        None => unreachable!(),
    }
}

#[tracing::instrument(level = "debug")]
fn parse_item(text: &str) -> eyre::Result<(String, String)> {
    let mut db = dada_db::Db::default();
    let filename = Filename::from(&db, "<repl-input>");
    db.update_file(filename, text.into());

    let items = filename.items(&db);
    assert!(items.len() > 0);

    if items.len() > 1 {
        return Err(eyre::eyre!("parsed more than one item"));
    }

    let item = items.last().unwrap();
    let name = item.name(&db);
    let name = name.as_str(&db).to_string();
    let item_text = item.span(&db).snippet(&db).to_string();

    Ok((name, item_text))
}
