# Shleases vs sharing a lease

Given that you can

-   shlease an object `m` via `m.shlease`,
-   lease an object `m` via `m.lease`,
-   and share an object `m` via `m.ref`

you may be wondering what would happen if were to _share_ a _leased object_. I.e., what would happen if you did `m.mut.ref`? The answer is that you get back a shleased reference, but there is a subtle difference between `m.shlease` and `m.mut.ref`. It has to do with the way that `m` can terminate the shlease. Let's explore!

## Sharing a lease

Let's start by looking at `m.mut.ref` in detail. Consider this example:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let l: mutable = m.lease
let s: shleased = l.ref
```

As you can see, sharing the lease `l` results in a `shleased` variable `s`. Try putting your cursor on that final line!

```
class Pair(our a, our b)
let m: my = Pair(22, 44)
let l: mutable = m.lease
let s: shleased = l.ref
#                   ▲
# ──────────────────┘

# You see:
#
# ┌───┐            ┌───────┐
# │ m ├╌my╌╌╌╌╌╌╌╌►│ Pair  │
# │   │            │ ───── │
# │ l ├╌mutable╌╌╌╌►│ a: 22 │
# │   │            │ b: 44 │
# │ s ├─shleased──►│       │
# └───┘            └───────┘
```

As you can see, the object `m` is considered mutable via an ordinary, exclusive lease to `l`. `l` is then in turn the lessor on a [sublease](./sublease.md), or rather, a sub*sh*lease, to `s`. This is interesting because leases and shleases are canceled in different ways:

-   A _lease_ is canceled if the lessor accesses the object **in any way**.
-   A _shlease_ is canceled if the lessor **writes to the object**.

This means that if `m` reads from the object, that will cancel `l`, which will in turn cancel `s`:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let l: mutable = m.lease
let s: shleased = l.ref

print(m.a).await           # Reads from `m`, canceling `l` and `s`

print(s.a).await           # Error! `s` is canceled.
```

## Shleasing

Let's compare then to what would happen if we directly created a shlease on `m`. Go ahead and put your cursor at the end of this example:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let s: shleased = m.shlease
#                     ▲
# ────────────────────┘

# You see:
#
# ┌───┐            ┌───────┐
# │ m ├╌my╌╌╌╌╌╌╌╌►│ Pair  │
# │   │            │ ───── │
# │ s ├─shleased──►│ a: 22 │
# │   │            │ b: 44 │
# └───┘            └───────┘
```

As you can see, `s` is directly shleased from `m`. This means that we can continue to read from `m` without violating the shlease:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let s: shleased = m.shlease

print(m.a).await           # Prints `22`, nothing is canceled
print(s.a).await           # Also prints `22`
```

The owner `m` can still cancel the shlease by mutating the object:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let s: shleased = m.shlease

m.a += 1                   # Cancels the shlease
print(s.a).await           # Error!
```

## The crux of it: shleases permit lessor to continue reading, leases don't

As you can see, there is a subtle difference between having a _shared sublease of a lease_ and having a _shlease_:

-   In the first case, the person who mutable the object was still promised unique access, but they have then chosen to share that unique access with others. This doesn't give access to the original owner though!
-   In the _second_ case, the owner granted a shlease, which only promises that the object will not be mutated while the shlease is active. This means that the owner can continue to read from the object without canceling the shlease. It's only when the owner goes to mutate the object that the shlease is canceled.
