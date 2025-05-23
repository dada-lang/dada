//! # Dada Programming Language
//!
//! **👉 For complete documentation and APIs, see the [`dada_lang`](../dada_lang) crate.**
//!
//! This crate provides the main binary entry point for the Dada programming language compiler.
//! The actual compiler implementation, APIs, and comprehensive documentation are located in
//! the [`dada_lang`](../dada_lang) crate.
//!
//! ## Quick Start
//!
//! If you're looking to:
//!
//! - **Use the Dada compiler** - You're in the right place! Install with `cargo install dada`
//! - **Understand the compiler architecture** - See [`dada_lang`](../dada_lang) for the complete overview
//! - **Explore the type system** - Start with [`dada_ir_sym`](../dada_ir_sym) documentation
//! - **Contribute to development** - Check out the [`dada_lang`](../dada_lang) module documentation
//!
//! ## Example Usage
//!
//! ```bash
//! # Compile a Dada source file
//! dada compile my_program.dada
//!
//! # Run a Dada program
//! dada run my_program.dada
//!
//! # Run tests
//! dada test tests/
//! ```
//!
//! ## Architecture
//!
//! The Dada compiler is organized as a workspace with several components:
//!
//! - [`dada_lang`](../dada_lang) - Main compiler APIs and CLI (start here!)
//! - [`dada_parser`](../dada_parser) - Lexing and parsing
//! - [`dada_ir_sym`](../dada_ir_sym) - Symbolic IR and type checking
//! - [`dada_check`](../dada_check) - Type checking orchestration
//! - [`dada_codegen`](../dada_codegen) - WebAssembly code generation
//! - [`dada_compiler`](../dada_compiler) - Compilation orchestration
//!
//! For the complete documentation, visit [`dada_lang`](../dada_lang).

// Re-export the main API from dada-lang for convenience
pub use dada_lang::*;
