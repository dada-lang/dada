# Share and share alike with the `our` permission

import Caveat from '../caveat.md'

<Caveat/>

The `our` permission declares **shared ownership** over an object. As the word _shared_ suggests, multiple variables can have shared ownership at the same time. Sharing is really useful, because it lets you copy data easily from one variable to another without losing access to it:

```
class Point(x: our, y: our)

let x: our = Point(22, 44)
let y: our = x
print("Look ma, I can access both `{x}` and `{y}`").await

# prints Look ma, I can access both `Point(22, 44)` and `Point(22, 44)`
```

When you do this, we would say that both `x` and `y` are **shared owners** of the same point. If you position your cursor in the code above, you will [see](https://asciiflow.com/#/share/eJyrVspLzE1VssorzcnRUcpJrEwtUrJSqo5RqohRsrK0MNOJUaoEsowsDYGsktSKEiAnRunRlD3IKCYmD0gqKChASDSAphiLxgKomtxKPGrR0bRdIK0B%2BZl5JQrEuAHdHUiaCvG5kbAbKqwUjIyIc0OllYKJCUIpintICjilWqVaANIL5SU%3D) that depicted graphically:

```
┌───┐
│   │                  ┌───────┐
│ p ├─our─────────────►│ Point │
│   │                  │ ───── │
│ q ├─our─────────────►│ x: 22 │
│   │                  │ y: 44 │
└───┘                  └───────┘
```

## My to our

When you assign from a `my` variable into an `our` variable, the `my` variable is giving up ownership of its object. Consider this program:

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
print("I can access {p}").await    # Prints `Point(22, 44)`

let q: our = p
let r: our = q
print("I can access {q}").await    # Prints `Point(22, 44)`
print("I can access {r}").await    # Prints `Point(22, 44)`
print("I cannot access {p}").await # Error!
```

Try moving your cursor around to see how ownership evolves. If you position the cursor right after the first `print` line, you'll see that the variable `p` has unique ownership of the `Point`.

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
print("I can access {p}").await    # Prints `Point(22, 44)`
#                              ▲
# ─────────────────────────────┘
...

# You see:
#
# ┌───┐       ┌───────┐
# │ p ├──my──►│ Point │
# │   │       │ ───── │
# │ q │       │ x: 22 │
# │   │       │ y: 44 │
# │ r │       └───────┘
# └───┘
#
```

If you move the cursor to after the `our q = p` line, you'll see that ownership has been transferred to `q`:

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
print("I can access {p}").await    # Prints `Point(22, 44)`

let q: our = p
#             ▲
# ────────────┘
...

# You see:
#
# ┌───┐       ┌───────┐
# │ p │       │ Point │
# │   │       │ ───── │
# │ q ├─our──►│ x: 22 │
# │   │       │ y: 44 │
# │ r │       └───────┘
# └───┘
#
```

Try moving the cursor to after `let r: our = q`, what do you see then?

## Our to my

What do you think happens if you try to assign from an `our` variable to a `my` variable? Try it and see:

```
class Point(x: our, y: our)
let o: our = Point(22, 44)
let m: my = o                     # Error!
```

As you can see, you get an error: once you have given up unique access to an object, you can't get it back again. This is because it's always possible that you have copied the `our` value to other places:

```
class Point(x: our, y: our)
let o: our = Point(22, 44)
let o2: our = o                   # <-- e.g., you might have done this
let m: my = o                     # Error!
```

In that case, if we permitted `o` to be copied to `m`, that would have to invalidate `o2` as well -- otherwise `m` couldn't have unique access to the object.

## Why _can't_ we invalidate other `our` values?

Looking at this example, you might be wondering "why _can't_ we invalidate `o2` when we give ownership to `m`?"

```
class Point(x: our, y: our)
let o: our = Point(22, 44)
let o2: our = o                   # <-- e.g., you might have done this
let m: my = o                     # Error!
```

The answer is that `o2` _owns_ the `Point` -- it is shared ownership, but it is still ownership. The distinguish characteristic of owning an object is that "nobody can take your object away from you". That is, there are no actions that _other code_ can take that will invalidate `o2`. The only way for the value in `o2` to go away is if `o2` either goes out of scope or is assigned to a new value. Later on, we'll see [_leases_](./lease.md) and [_shleases_](./shlease.md) are the way to give temporary permissions that can be revoked later.[^gc]

[^gc]: If you want a more operational form of the answer, consider this: It is actually quite hard to determine if there are other variables out there in your program that have access to a given object, particularly in an optimized implementation. If you use a tracing garbage collector, you have to scan every stack of every thread. If you use reference counting, you can check for a reference count of 1, but that assumes you are not using an optimized reference counting implementation like [deferred reference counting](https://dl.acm.org/doi/10.1145/185009.185016).

## The share keyword

We saw that the `give` keyword is a way to make ownership transfer explicit:

```
class Point(x: our, y: our)
let p: my = Point(22, 44)
let q: my = p.give
```

In the same way, the `share` keyword is a way to make conversion into something shared explicit:

```
class Point(x: our, y: our)
let p: my = Point(22, 44)
let q: our = p.ref
#         ~~~~~ sharing from a `my` variable makes it give up ownership
let r: our = q.ref
#         ~~~~~ sharing an `our` object is just a copy
```

## Give applied to a shared value

The `give` keyword always gives _whatever permissions you have_ to someone else. When you apply `give` to an `our` object, that creates another `our` object. Since `our` is shared, applying `give` to an `our` variable doesn't invalidate the original variable. In fact, it's equivalent to `share`:

```
class Point(x: our, y: our)
let p: our = Point(22, 44)
let q: our = p.give
let r: our = q.give
print("I can still use {p}, {q}, and {r}").await
```
