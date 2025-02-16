use dada_ir_ast::{
    diagnostic::{Diagnostic, Level, Reported},
    span::Span,
};

use crate::{
    check::{env::Env, predicates::Predicate},
    ir::{indices::InferVarIndex, types::SymGenericTerm, variables::SymVariable},
};

pub(super) fn report_infer_is_contradictory<'db>(
    env: &Env<'db>,
    var_index: InferVarIndex,
    predicate: Predicate,
    predicate_span: Span<'db>,
    inverted_predicate: Predicate,
    inverted_span: Span<'db>,
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
        format!("I could not infer a `{var_kind}` here because it would have to be both `{predicate}` and `{inverted_predicate}`"),
    )
    .label(
        db,
        Level::Error,
        inverted_span,
        format!("required to be `{inverted_predicate}` here"),
    )
    .label(
        db,
        Level::Error,
        predicate_span,
        format!("required to be `{predicate}` here"),
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
