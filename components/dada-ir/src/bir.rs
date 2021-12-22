//! The bir (pronounced like "beer" 🍺) is the "base IR" -- it's what we compile the
//! AST down to for the interpreter or compiler.

// /// Stores the ast for a function.
// #[derive(Clone, Debug, PartialEq, Eq)]
// pub struct Bir {
//     /// Interning tables for expressions and the like.
//     pub tables: Tables,
// }

// tables! {
//     pub struct Tables {
//         local_variables: alloc LocalVariable => LocalVariableData,
//     }
// }

// id!(pub struct LocalVariable);

// pub struct LocalVariableData {
//     name:
// }
