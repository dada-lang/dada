# [Dynamic Dada](./dyn_tutorial.md) outline

Outline of the dynamic dada tutorial and its contents.

## Basic syntax

* `class Point`
* `my`, `our` exist
* `async`, `await`
* `=`, `:=`, `+=`
* labeled arguments
* use of the `->` to return from a function

## Owned modes

Covers `my` and `our` and their interactions

### my: unique ownership

* the fact that `my -> my` gives ownership
    * `my x = y` takes `y`
    * `fn foo(my x)`, `foo(y)` also takes `y`
* the `give` keyword for making this explicit
    * `my x = y.give`
    * `foo(y.give)`

### our: shared ownership

* introducing the `our` mode, and how it copies
    * `our p = Point(...)`, `our q = p` copies
    * `fn foo(our x)`, we saw `class Point(our x)`
* `my -> our` gives ownership
    * `my p = Point(...), our q = p`
* once shared, cannot go back to unique, could be other copies
    * `my q = Point(....).share`
* the `share` keyword for making this explicit
    * `our q = p.share`
* `give` applied to `share` has no effect

### friends don't let friends mutate `our` data

* `our p = Point(...)`, `p.x += 1` → error and why

### inherited modes

```
class Object()
class Pair(my a, my b)
our p = Pair(Object(), Object())
our o = p.a                       # my field in an our context is our
```

### any: dynamic modes

* `any p = Point(...)`
* `any q = p.share` etc

## Leasing

Don't always want to give permanent access. A **lease** lets you create a temporary reference. It can get cancelled by using your original again.

### leased: unique lease

* `my p = ..., leased q = p`
* make explicit with the `lease` keyword
* cancelling a lease
    * by writing to the original
    * by reading from the original
    * by sharing from the original
* *moving* doesn't (necessarily) cancel a lease
* returning leased content

### subleasing

* `leased q = ..., leased r = q`
* cancellation

### shared leases

* `my p = ..., our leased q = p, our leased r = q`
* actually the default
    * `my p = ..., q = p, r = q`
* make explicit with the `shlease` keyword
    * show how you can have multiple co-existing shleases

## Atomic: shared mutation

* atomic keyword, blocks, functions
* await and atomic don't mix
* leasing atomic fields and so forth
