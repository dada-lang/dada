# Desired experience

These are the things we are trying for with Dada.

## Easy to learn, but not dumbed down

Dada is meant to a language that experts want to use. It should be easy to learn, but it doesn't achieve that by hiding important concepts like ownership and borrowing. Rather, it brings them to the fore, but in a packaging that is (hopefully) easy to learn.

## Does the right things, not everything

Dada doesn't attempt to do *everything*. It's not really meant to run on a tiny sensor node, for example, and nobody is going to to put it in a kernel antime soon. But it aims to expose the *right things* that most people building complex software really need. In other words, you can't always do what you want, but if you try sometimes, with Dada, you can do what you need.

## Opinionated (in a good way) 

Going along with the previous point, Dada is not afraid to make some choices on your behalf. For example, it has a builtin smart pointer type (the box) that supports both unique ownership and reference counting. It includes builtin support for vector and hashmap types as well.

## Predictable and reasonably efficient

Dada's performance doesn't have to be *micro-efficient*, but it must be predictable. It's ok to call `malloc`, but not to run a garbage collector, which introduces variable latency.

# Out of scope goals

## Rust compability (for now, anyway)

Right now, Dada makes no effort to be compatible with Rust in particular. That said, But if Dada is a success, the goal would be to allow Dada and Rust code to smoothly interoperate. This may require changes to both Dada and Rust.

Alternatively, another path that may be worth considering is leaning heavily on [WebAssembly and Interface Types](https://github.com/WebAssembly/interface-types/blob/master/proposals/interface-types/Explainer.md) for interoperability, in which case it would work smoothly across not only Rust but all languages that compile to WebAssembly.
