use dada_ir::{code::syntax, input_file::InputFile, span::LineColumn};

/// Salsa input: the set of breakpoint locations.
///
/// Defaults to empty set if not explicitly set.
///
/// Does this belong here? I can't decide.
#[salsa::tracked(return_ref)]
#[allow(clippy::needless_lifetimes)]
pub fn breakpoint_locations(_db: &dyn crate::Db, _input_file: InputFile) -> Vec<LineColumn> {
    vec![] // default: none
}

/// Returns all the breakpoints set for a given chunk of code.
pub fn breakpoints_in_tree(
    db: &dyn crate::Db,
    input_file: InputFile,
    tree: syntax::Tree,
) -> Vec<syntax::Expr> {
    let locations = breakpoint_locations(db, input_file);
    locations
        .iter()
        .flat_map(|l| crate::breakpoint::find(db, input_file, *l))
        .filter(|bp| bp.tree == tree)
        .map(|bp| bp.expr)
        .collect()
}
