use dada_ir::{code::syntax, input_file::InputFile};

/// Returns all the breakpoints set for a given chunk of code.
pub fn breakpoints_in_tree(
    db: &dyn crate::Db,
    input_file: InputFile,
    tree: syntax::Tree,
) -> Vec<syntax::Expr> {
    let locations = input_file.breakpoint_locations(db);
    locations
        .iter()
        .flat_map(|l| crate::breakpoint::find(db, input_file, *l))
        .filter(|bp| bp.tree == tree)
        .map(|bp| bp.expr)
        .collect()
}
