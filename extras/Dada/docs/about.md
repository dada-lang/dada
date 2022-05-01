# Welcome

> I speak only of myself since I do not wish to convince, I have no right to drag others into my river, I oblige no one to follow me and everybody practices their art their own way.
> 
> Tristan Tzara, "Dada Manifesto 1918”[^1]

## What the heck is Dada?

Dada is a thought experiment. What if we were making a language like Rust, but one that was meant to feel more like Java or JavaScript, and less like C++? One that didn't aspire to being used in kernels or tiny embedded devices and was willing to require a minimal runtime. What might that look like?

## What is the state of Dada?

As of right now, Dada doesn't really exist, though we have some experimental prototypes:

* There is an experimental operational semantics implemented in [PLT Redex], which you can find at [dada-lang/dada-model](https://github.com/dada-lang/dada-model/). More details are available (or will be eventually) in the [Design Docs section](/docs/design_docs) of this site.
* The interpreter, written in Rust, is found on the [dada-lang/dada](https://github.com/dada-lang/dada) repository. You can try a WebAssembly-based build on the [Dada playground](https://dada-lang.org/playground). More details are available in the "Development Guide" section of this site.

[PLT Redex]: https://redex.racket-lang.org/why-redex.html

OK, from here on out I'm going to pretend that Dada really exists in its full glory.

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

Check out one of our [tutorials](/docs/tutorials/).

## Footnotes

[^1]: Updated to use modern pronouns.