# Tutorial: Typed Dada

{{#include caveat.md}}

This tutorial picks up[^orwill] where the [Dynamic Dada](./dyn_tutorial.md) tutorial leaves off. It introduces the static type system used by the Dada compiler to help your code avoid errors related to ownership permissions. Using Typed Dada not only makes your code more secure, it also allows us to compile down to tight and efficient machine code that doesn't require any kind of permission checks.

[^orwill]: Or it will, once it is written, lol.