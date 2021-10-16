# Tutorial

## ⚠️ **DADA DOESN'T REALLY EXIST.** ⚠️ 

This "tutorial" is a thought exercise to get a feeling for how it would be to teach Dada and to explore the ideas without going through the trouble of implementing them.

## What is Dada?

Dada is a language for writing reliable, efficient programs. It has a lot in common with Rust, so if you know Rust, you'll likely feel right at home. At the same time, it is meant to feel a lot closer to Java or JavaScript, so if you know those languages, you'll hopefully find it familiar as well. Like Rust, Dada has an ownership-based type system with strong safety guarantees. It doesn't require a garbage collector and it generally compiles to very efficient code. *Unlike Rust,* Dada doesn't aim to support *zero-cost abstractions* to the same extent: it has a required runtime, for example, and it Dada values all follow some similar layout rules to improve ergonomics, which can make them larger than they otherwise would be. Dada also doesn't support inline assembly or features like that; if you want to do that sort of thing, you can call out to Rust code.[^unsafe] 

## WebAssembly first, optional interpreter

Dada's native target is WebAssembly. It is meant to be embedded on the web and it uses [interface types] (spec) for its FFI. 

In addition to the compiler, Dada has an interpreter. The interpreter is generally used for testing and debugging, since it can do more validation and internal testing. You'll find that all examples in this tutorial are editable and runnable. This is done via the Dada interpreter compiled to WebAssembly.

[interface types]: https://hacks.mozilla.org/2019/08/webassembly-interface-types/

[^unsafe]: There is a notion of unsafe code in Dada, but it is much more limited in scope that Rust's unsafe.

## Chapters

The tutorial is broken up into chapters:

- [Dada basics](./tutorial/basics.md): **Start here!** Introduces the basics of Dada, including ownership and lending.
- [Sharing](./tutorial/sharing.md): Introducing sharing.
- [Joint ownership](./tutorial/our.md): Introducing joint ownership.
- [Classes](./tutorial/classes.md): Creating classes.
- [Matching](./tutorial/match.md): Matching and subclassing.
- [Interfaces](./tutorial/interface.md): Matching and subclassing.

