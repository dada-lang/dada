# Goals

These are the things Dada is trying for.s

## Easy to learn, but not dumbed down

Dada is meant to a language that experts want to use. It should be easy to learn, but it doesn't achieve that by hiding important concepts like ownership and borrowing. Rather, it brings them to the fore, but in a packaging that is (hopefully) easy to learn.

## Does the right thing, not everything

Dada doesn't attempt to do *everything*. It's not really meant to run on a tiny sensor node, for example, and nobody is going to to put it in a kernel antime soon. But it aims to expose the *right things* that most people building complex software really need. In other words, you can't always do what you want, but if you try sometimes, with Dada, you can do what you need.

## Opinionated (in a good way) 

Going along with the previous point, Dada is not afraid to make some choices on your behalf. For example, it has a builtin smart pointer type (the box) that supports both unique ownership and reference counting. It includes builtin support for vector and hashmap types as well.

## Predictable and reasonably efficient

Dada's performance doesn't have to be *micro-efficient*, but it must be predictable. It's ok to call `malloc`, but not to run a garbage collector, which introduces variable latency.

# Design principles

Some key design principles intended to create the above experience.

## Values, not places or pointers

Like Java or Python, Dada encourages programmers just to think about the *values* they are working with. Dada doesn't have a "dereference" operator. Even though Dada doesn't require a GC, it's meant to encourage familiar, GC-like patterns. If you were to remove all of Dada's sharing and ownership annotations, the result should be a program that could run just fine with a GC, with the expected semantics.

## RAII is for freeing memory, not side effects

Dada has destructors, but they are not expected to have side-effects apart from releasing resources. The Dada compiler is always free to drop values even before the variable that owns them has gone out of scope.

## Share via variables

Dada encourages you to think about sharing in terms of *variables* that you are sharing from. When you write Dada functions that return shared content, you idiomatically specify the *local variables* from which that shared content will be returned:

```
fn pick<T>(x: shared T, y: shared T) -> shared(x|y) T
```

Dada does have an underlying concept, the **lease**, that is analogous to Rust's lifetime: it's a way of naming an unknown set of variables. However, it is unusual to explicitly name leases; that is only required in more advanced scenarios such as creating an iterator.

## Subtyping



# Out of scope goals

## Rust compability (for now, anyway)

Right now, Dada makes no effort to be compatible with Rust in particular. That said, But if Dada is a success, the goal would be to allow Dada and Rust code to smoothly interoperate. This may require changes to both Dada and Rust.

Alternatively, another path that may be worth considering is leaning heavily on [WebAssembly and Interface Types](https://github.com/WebAssembly/interface-types/blob/master/proposals/interface-types/Explainer.md) for interoperability, in which case it would work smoothly across not only Rust but all languages that compile to WebAssembly.
