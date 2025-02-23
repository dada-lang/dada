use std::error::Report;

use dada_ir_ast::{
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::{
        classes::SymAggregate,
        indices::InferVarIndex,
        types::{SymGenericTerm, SymTyName},
        variables::SymVariable,
    },
};

pub trait OrElse<'db> {
    fn report(&self, db: &'db dyn Db, because: Because) -> Reported;

    fn or_else(&self, because: Because) -> Diagnostic<'db>;

    fn to_arc(&self) -> Arc<dyn OrElse<'db>>;
}

impl<'db, F> OrElse<'db> for F
where
    F: 'db,
    F: Fn(Because) -> Diagnostic<'db>,
    F: Clone,
{
    fn report(&self, db: &'db dyn Db, because: Because) -> Reported {
        (*self)(because).report(db)
    }

    fn or_else(&self, because: Because) -> Diagnostic<'db> {
        (*self)(because)
    }

    fn to_arc(&self) -> Arc<dyn OrElse<'db>> {
        Arc::new(F::clone(self))
    }
}

pub enum Because<'db> {
    /// `shared[place]` was required
    NotSubOfShared(SymPlace<'db>),

    /// `leased[place]` was required
    NotSubOfLeased(SymPlace<'db>),

    /// Universal variable was not declared to be `predicate` (and it must be)
    VarNotDeclaredToBe(SymVariable<'db>, Predicate),

    /// Universal variable was declared to be `predicate` (but ought not to be)
    VarDeclaredToBe(SymVariable<'db>, Predicate),

    /// The never type is not copy
    NeverIsNotCopy,

    /// The never type is not copy
    NeverIsNotLent,

    /// Classes are not copy
    ClassIsNotCopy(SymTyName<'db>),

    /// `A B` is not lent
    ApplicationNotLent(SymGenericTerm<'db>, SymGenericTerm<'db>),

    /// Struct is not copy because some component of the given struct type is not copy
    StructComponentNotCopy(SymTyName<'db>, SymGenericTerm<'db>, Box<Because<'db>>),

    /// Struct is copy
    StructIsCopy(SymTy<'db>),

    /// Perm is copy
    PermIsCopy(SymPerm<'db>),

    /// Leasing from a copy place yields a copy permission (which is not desired here)
    LeasedFromCopyIsCopy(SymPerm<'db>),
}

impl Because {
    pub fn struct_component_not_copy<'db>(
        self,
        sym_ty_name: SymTyName<'db>,
        term: impl Into<SymGenericTerm<'db>>,
    ) -> Because<'db> {
        Because::StructComponentNotCopy(sym_ty_name, term.into(), Box::new(self))
    }
}

pub(super) fn report_infer_is_contradictory<'db>(
    env: &Env<'db>,
    var_index: InferVarIndex,
    predicate: Predicate,
    predicate_span: Span<'db>,
    isnt_span: Span<'db>,
) -> Reported {
    let db = env.db();
    let (var_span, var_kind) = env
        .runtime()
        .with_inference_var_data(var_index, |data| (data.span(), data.kind()));

    Diagnostic::error(
        db,
        var_span,
        format!("contradictory requirements for `{var_kind}`"),
    )
    .label(
        db,
        Level::Error,
        var_span,
        format!("I could not infer a `{var_kind}` here because it would have to be both `{predicate}` and not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        predicate_span,
        format!("required to be `{predicate}` here"),
    )
    .label(
        db,
        Level::Error,
        isnt_span,
        format!("required not to be `{predicate}` here"),
    )
    .report(db)
}

pub(super) fn report_never_must_be_but_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the never type (`!`) is not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("the never type (`!`) is considered `my` and therefore is not `{predicate}`"),
    )
    .report(db)
}

pub(super) fn report_term_must_not_be_leased_but_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: impl Into<SymGenericTerm<'db>>,
) -> Reported {
    let term: SymGenericTerm<'db> = term.into();
    let kind = match term.kind() {
        Ok(kind) => kind,
        Err(reported) => return reported,
    };
    let db = env.db();
    Diagnostic::error(db, span, format!("the {kind} `{term}` cannot be leased"))
        .label(
            db,
            Level::Error,
            span,
            format!("the {kind} `{term}` is considered leased"),
        )
        .report(db)
}

pub(super) fn report_term_must_be_but_isnt<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
) -> Reported {
    let term: SymGenericTerm<'db> = term.into();
    let kind = match term.kind() {
        Ok(kind) => kind,
        Err(reported) => return reported,
    };
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the {kind} `{term}` is not `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("I expected a `{predicate}` {kind} but I found `{term}`"),
    )
    .report(db)
}

pub(super) fn report_term_must_not_be_but_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    term: impl Into<SymGenericTerm<'db>>,
    predicate: Predicate,
) -> Reported {
    let term: SymGenericTerm<'db> = term.into();
    let kind = match term.kind() {
        Ok(kind) => kind,
        Err(reported) => return reported,
    };
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("the {kind} `{term}` must not be `{predicate}`"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("I did not expect a `{predicate}` {kind} but I found `{term}`"),
    )
    .report(db)
}

pub(super) fn report_var_must_be_but_is_not_declared_to_be<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("variable `{var}` must be `{predicate}` but is not declared to be"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("variable `{var}` is not declared to be `{predicate}`"),
    )
    .report(db)
}

pub(super) fn report_var_must_not_be_declared_but_is<'db>(
    env: &Env<'db>,
    span: Span<'db>,
    var: SymVariable<'db>,
    predicate: Predicate,
) -> Reported {
    let db = env.db();
    Diagnostic::error(
        db,
        span,
        format!("variable `{var}` must not be declared as `{predicate}` (but it is)"),
    )
    .label(
        db,
        Level::Error,
        span,
        format!("variable `{var}` must not be declared to be `{predicate}`"),
    )
    .report(db)
}
