# Dada Programming Language Compiler

Dada is an experimental programming language that explores what a Rust-like language would look like if designed to feel more like Java/JavaScript rather than C++. It's a thought experiment that trades systems programming constraints for higher-level language ergonomics, requiring a minimal runtime.

## Key Language Features

- **Async-first**: Functions are async by default - no need to specify `async fn`
- **Permission-based ownership**: Uses ownership annotations (`my`, `our`, `mut`) for memory safety
- **Class and struct support**: Both value types (structs) and reference types (classes) 
- **WASM compilation target**: Generates WebAssembly as the primary output format
- **Familiar syntax**: Rust-inspired but more accessible, with `#` comments like Python

## Architecture Overview

The Dada compiler is built as a multi-stage pipeline using the [Salsa](https://github.com/salsa-rs/salsa) incremental computation framework:

```text
Source Code
    ↓
[dada-parser] → AST
    ↓  
[dada-ir-sym] → Symbolic IR (type checking happens here)
    ↓
[dada-codegen] → WebAssembly
```

### Core Crates

- **[`dada_parser`](../dada_parser)** - Lexing and parsing Dada source into AST
- **[`dada_ir_sym`](../dada_ir_sym)** - Symbolic IR, type checking, and semantic analysis
- **[`dada_check`](../dada_check)** - High-level checking orchestration 
- **[`dada_codegen`](../dada_codegen)** - WebAssembly code generation
- **[`dada_compiler`](../dada_compiler)** - Compilation orchestration and virtual file system
- **[`dada_debug`](../dada_debug)** - Debug server for compiler introspection

## Type System Highlights

Dada's type system combines:

- **Hindley-Milner type inference** adapted for an object-oriented setting
- **Permission inference** for ownership (`my`, `our`, `mut`, `ref`)
- **Subtyping** with structural and nominal aspects
- **Generic programming** with where-clause constraints
- **The "Red" type system** for ownership analysis

## Getting Started

To explore the compiler implementation:

1. **Start with [`dada_ir_sym`](../dada_ir_sym)** - This contains the main type checking logic
2. **Examine the type checking pipeline** - Look at the check module
3. **Study type inference algorithms** - See the inference module  
4. **Understand the permission system** - Explore the predicates module

## Example Dada Code

```text
# Classes use reference semantics
class BankAccount(my name: String, mut balance: Amount) {
    fn transfer_to(mut self, mut target: BankAccount, amount: Amount) {
        self.mut.withdraw(amount)
        target.mut.deposit(amount)
    }
}

# Functions are async by default
async fn main() {
    let mut alice = BankAccount("Alice", 100.0)
    let mut bob = BankAccount("Bob", 50.0)
    
    alice.mut.transfer_to(bob.mut, 25.0).await
    print(f"Alice: {alice.balance}, Bob: {bob.balance}").await
}
```

## Current Status

⚠️ **Early Development**: Dada is actively under development. Many features are implemented but the language is not yet production-ready. Most test files include `#:skip_codegen` as WebAssembly generation is still incomplete.