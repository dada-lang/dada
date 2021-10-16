# Welcome

## What the heck is Dada?

Dada is a thought experiment. What if we were making a language like Rust, but one that was meant to feel more like Java or JavaScript, and less like C++? One didn't aspire to being used in kernels or tiny embedded devices and was willing to require a minimal runtime. What might that look like?

## What is the state of Dada?

At the moment, Dada doesn't really exist. Everything you read on this site is imaginary. However, there is an experimental operational semantics implemented in [PLT Redex](https://redex.racket-lang.org/why-redex.html), which you can find at [dada-lang/dada-model](https://github.com/dada-lang/dada-model/). More details are available (or will be eventually) in the [calculus section](./calculus.md) of this site.

OK, from here on out I'm going to start pretend that Dada really exists.

## Dada in a nutshell

Dada is an ownership-based language that is in some ways similar to Rust:

* Like Rust, Dada doesn't require a garbage collector.
* Like Rust, Dada guarantees memory safety and data-race freedom.
* Like Rust, Dada data structures can be allocated in the stack and use flat memory layouts.

In other ways, though, Dada is very different:

* Like TypeScript, Dada is a **gradually typed** language:
    * That means you can **start out using Dada in the interpreter, with no type annotations**, to get a feel for how it works.
    * Once you've gotten comfortable with it, you can **add type annotations and use the compiler for performance comparable to Rust**.
* Dada **targets WebAssembly** first and foremost:
    * You can build native targets with Dada, but its FFI system is based on [WebAssembly interface types](https://hacks.mozilla.org/2019/08/webassembly-interface-types/).
* Dada is **object-oriented**, though not in a purist way:
    * Dada combines OO with nice features like pattern matching, taking inspiration from languages like Scala.

Dada also has some limitations compared to Rust:

* Dada has a required runtime and does not target "bare metal systems" or kernels.
* Dada does not support inline assembly or arbitrary unsafe code.

## Curious to learn more?

Check out one of our tutorials:

* [Dynamic Dada](./dyn_tutorial.md)
* [Dada for Rustaceans](./rustaceans.md)
