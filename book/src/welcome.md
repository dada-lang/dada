# Welcome

## What the heck is Dada?

Dada is a thought experiment. What if we were making a language like Rust, but one that was meant to feel more like Java or JavaScript, and less like C++? One didn't aspire to being used in kernels or tiny embedded devices and was willing to require a minimal runtime. What might that look like?

## Why work on Dada?

Working on Dada is really fun and, frankly, kind of relaxing for me. It's also a way to explore different language ideas unfettered by constraints of backwards compatibility. It is my hope that some of the ideas in Dada can make their way back to Rust. --nikomatsakis

## The calculus

At the moment, Dada is implemented as a [PLT Redex](https://redex.racket-lang.org/why-redex.html) model. It includes a formal grammar, a type system, and an operational semantics (for single threaded programs only). This model explores an alternative, Rust-like type system. This site has an [explanation of how the calculus works](./calculus.md).

## Tutorial

Currently, there is no "surface syntax" for Dada. But it's fun to think about! So a second part of this site is a kind of exploration of some ideas for how Dada might look if it were taken from being a model into a full-fledged language. This takes the form of a [tutorial](./tutorial.md).

