# Goals

These are the things Dada is trying for.

## Easy to learn, but not dumbed down

Dada is meant to be a language that experts want to use. It should be easy to learn, but it doesn't achieve that by hiding important concepts like ownership and borrowing. Rather, it brings them to the fore, but in a packaging that is (hopefully) easy to learn.

## Does the right thing, not everything

Dada doesn't attempt to do *everything*. It's not really meant to run on a tiny sensor node, for example, and nobody is going to to put it in a kernel anytime soon. But it aims to expose the *right things* that most people building complex software really need. In other words, you can't always do what you want, but if you try sometimes, with Dada, you can do what you need.

## Opinionated (in a good way) 

Going along with the previous point, Dada is not afraid to make some choices on your behalf. For example, it has a builtin smart pointer type (the box) that supports both unique ownership and reference counting. It includes builtin support for vector and hashmap types as well.

## Predictable and reasonably efficient

Dada's performance doesn't have to be *micro-efficient*, but it must be predictable. It's ok to call `malloc`, but not to run a garbage collector, which introduces variable latency. It should also be possible to learn the rules for when `malloc` will be called and to control that if you choose.

## Fun, lightweight, and familiar

Coding in Dada should feel pretty similar to coding in JavaScript, Python, Ruby, or Go, at least a lot of the time.

# Design principles

Some key design principles intended to create the above experience.

## Type erasure

Dada's type system is meant to be "fully erasable", similar to the relationship between TypeScript and JavaScript. That is, given some Typed Dada program, you should be able to delete all the type annotations and have it run in exactly the same way in Dynamic Dada. Any type errors should manifest as runtime exceptions at some point. 

(Caveat: we may want to use function parameter types as "hints", in which case the correct erasure would inclue applying those hints. Have to see.)

## Values, not places or pointers

Like Java or Python, Dada encourages programmers just to think about the *values* they are working with. Dada doesn't have pointer types like `*` or `&` nor a "dereference" operator like `*`. Even though Dada doesn't require a GC, it's meant to encourage familiar, GC-like patterns. If you were to remove all of Dada's sharing and ownership annotations, the result should be a program that could run just fine with a GC, with the expected semantics.

## RAII is for releasing resources, not side effects

Dada has destructors, but they are not expected to have side-effects apart from releasing resources. The Dada compiler is always free to drop values even before the variable that owns them has gone out of scope. So, things like freeing memory, closing files, database handles, and sockets? Perfect. Printing data onto the console or sending a message over the wire? Not so good.

## Sharing xor mutability

Dada shares Rust's obsession with exposing *sharing* (aliasing) and *mutability*, but not both at the same time.

# Longer term goals

## Rust interop

We dream of the day that Dada programs interop smoothly with Rust and can (e.g.) live happily in crates.io. However, that's not something that we're currently focusing on, since we're still playing with the basic structure of Dada. Once we figure out what Dada wants to be, we can figure out how to connect that with Rust (which could involve proposing changes to Rust, if we find patterns in Dada that would be a nice fit). Another path that may be worth considering is leaning heavily on [WebAssembly and Interface Types](https://github.com/WebAssembly/interface-types/blob/master/proposals/interface-types/Explainer.md) for interoperability, in which case it would work smoothly across not only Rust but all languages that compile to WebAssembly. (These things are not exclusive.)
