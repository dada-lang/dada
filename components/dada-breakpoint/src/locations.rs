use dada_ir::{code::syntax, filename::Filename, span::LineColumn};

/// Salsa input: the set of breakpoint locations.
///
/// Defaults to empty set if not explicitly set.
///
/// Does this belong here? I can't decide.
#[salsa::memoized(in crate::Jar ref)]
#[allow(clippy::needless_lifetimes)]
pub fn breakpoint_locations(_db: &dyn crate::Db, _filename: Filename) -> Vec<LineColumn> {
    vec![] // default: none
}

/// Returns all the breakpoints set for a given chunk of code.
pub fn breakpoints_in_tree(
    db: &dyn crate::Db,
    filename: Filename,
    tree: syntax::Tree,
) -> Vec<syntax::Expr> {
    let locations = breakpoint_locations(db, filename);
    locations
        .iter()
        .flat_map(|l| crate::breakpoint::find(db, filename, *l))
        .filter(|bp| bp.tree == tree)
        .map(|bp| bp.expr)
        .collect()
}
