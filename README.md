# dada

> I speak only of myself since I do not wish to convince, I have no right to drag others into my river, I oblige no one to follow me and everybody practices his art in his own way.
>
> *Tristan Tzara, "Dada Manifesto 1918”*

## What the heck is Dada?

Dada is a thought experiment. What if we were making a language like Rust, but one that was meant to feel more like Java or JavaScript, and less like C++? One that didn't aspire to being used in kernels or tiny embedded devices and was willing to require a minimal runtime. What might that look like?

### Dada in a nutshell

Dada is an ownership-based language that is in some ways similar to Rust:

* Like Rust, Dada doesn't require a garbage collector.
* Like Rust, Dada guarantees memory safety and data-race freedom.
* Like Rust, Dada data structures can be allocated in the stack and use flat memory layouts.

In other ways, though, Dada is very different:

* Like TypeScript, Dada is a gradually typed language:
  * That means you can start out using Dada in the interpreter, with no type annotations, to get a feel for how it works.
  * Once you've gotten comfortable with it, you can add type annotations and use the compiler for performance comparable to Rust.
* Dada targets WebAssembly first and foremost:
  * You can build native targets with Dada, but its FFI system is based on WebAssembly interface types.
* Dada is object-oriented, though not in a purist way:
  * Dada combines OO with nice features like pattern matching, taking inspiration from languages like Scala.

Dada also has some limitations compared to Rust:

* Dada has a required runtime and does not target "bare metal systems" or kernels.
* Dada does not support inline assembly or arbitrary unsafe code.

## To try it yourself...

* On the web, visit https://dada-lang.org/playground.
* On your own computer, clone this repository and do `cargo dada run dada_tests/hello_world.dada`. *Requires Rust and Cargo which can be installed by following these docs https://doc.rust-lang.org/cargo/getting-started/installation.html*
