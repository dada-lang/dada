# Why not have an explicit end for lease?

For a while I attempted to have an **explicit end** for every lease
rather than adopting the "stacked borrows"-like approach in which
leases are ended by their owner taking contrary action.

In many ways, explicit ends of leases is a cleaner approach.
The difference is where the error occurs:

```
class Counter(counter)

let x = Counter(0)

// Create a lease of x
let y = x.lease

// In the "explicit end" version, this is an error.
// In the "permissive" variant, it cancels the lease.
x.counter = 1

// In the "permissive" variant, error occurs here.
print(y.counter)
```

It's hard to say which of these statements is wrong.
Deferring the error until something has definitively gone wrong
is more permissive and hence the required approach for Rust's unsafe code
(which wants to be as backwards compatible with common practice as possible).
Dada's untyped mode is analogous to unsafe code, so that's plausibly a good choice here,
but I wanted to try the alternative.

## Where it goes wrong

To make things usable, you don't want to _EXPLICITLY_ end leases,
so we want to have some kind of _drop_ that is auto-inserted.
I imagined we would do this based on liveness information.
But that has a flaw: when you compute liveness, you see direct
uses of a variable, but not indirect. Consider this (correct) code:

```
class Counter(counter)
let x = Counter(0)
let y = x.lease // <-- last (direct) use of `x`
print(y.counter)
```

Liveness-based analysis would drop `x` immediately after the lease,
since it has no more direct uses. But that's not what we want.

We could in principle say that when something is dropped,
it remains in scope until the leases end,
but that's basically GC.

## Doing better requires type data

We could do better if we took types into account, but gradual typing implies they may not be available.
Besides, that's getting pretty complex.

## Implications for Rust

I would like to have liveness-based drops for Rust, but this reveals the (obvious in hindsight) flaw:
we never know when raw pointers are out there. So unless we know that, or declare it UB in some way,
we can't promote drops earlier.
