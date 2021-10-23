# Shared storage

{{#include ../caveat.md}}

We've seen that [the `share` keyword can be used to share objects](./share.md) and learned about the [sharing xor mutation][sxm] principle. Now, if you think back to when we were [first learning the syntax for classes](./class.md), you may recall that there were actually three access modes, `shared`, `var`, and `atomic`. We've only been using `var` so far. In this section, we're going to discuss the `shared` mode a bit deeper.

[sxm]: ./sharing_xor_mutation.md

## Shared storage makes shared values

When you have a local variable or a field in the `shared` mode, everything that gets stored in that value becomes shared automatically. Because of [sharing xor mutation][sxm], this typically means that those objects are also immutable (modulo [atomic data](./atomic.md), discussed soon).

Here is an example. In this example, we are creating a `Point`, but we are storing it into a local variable declared like `shared p = ...`. The result is that the field `x` cannot be mutated, just the same as [when we did `Point(..).share`][sxm]:

```
class Point(var x, var y)

async fn main() {
    shared p = Point(x: 22, y: 44)
    p.x += 1 // Exception!
}
```

## Shared storage is the default

The `shared` keyword in the previous example is not really necessary, because `shared` storage is the default:

```
class Point(var x, var y)

async fn main() {
    p = Point(x: 22, y: 44)
    p.x += 1 // Exception!
}
```

## Shared locals can be copied

In addition to preventing mutation, the shared mode allows easy copies. In this snippet, both `p` and `q` are joint owners of the same `Point`:

```
class Point(var x, var y)

async fn main() {
    p = Point(x: 22, y: 44)
    q = p
}
```

What do you think happens in this example, where we declared `q` with the `var` mode? Hint: try putting your cursor right after the end of `p = Point(..)`:

```
class Point(var x, var y)

async fn main() {
    p = Point(x: 22, y: 44)
    //                     ▲
    // ────────────────────┘
    var q = p
}

// ┌───┐
// │   │                  ┌───────┐
// │ p ├─our─────────────►│ Point │
// │   │                  │ ───── │
// │ q │                  │ x: 22 │
// │   │                  │ y: 44 │
// └───┘                  └───────┘
```

As you can see, because `p` is shared storage, the `Point` is already held with `our` permission. Therefore, if we move the cursor to after `var q = p`, we see that there are two `our` references to the same `Point`:

```
class Point(var x, var y)

async fn main() {
    p = Point(x: 22, y: 44)
    var q = p
    //       ▲
    // ──────┘
}

// ┌───┐
// │   │                  ┌───────┐
// │ p ├─our─────────────►│ Point │
// │   │                  │ ───── │
// │ q ├─our─────────────►│ x: 22 │
// │   │                  │ y: 44 │
// └───┘                  └───────┘
```

 What do you think happens if we do `q.x += 1`? Try it and see!

## Shared fields

We can make shared fields as well. for example, we might prefer to define `Point` with all shared fields. The result is a `Point` that cannot be mutated:

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44)
    p.x += 1 // Exception!
}
```

This is quite a common pattern. It allows you to ensure that whatever relationship `x` and `y` had when the `Point` was created is not disturbed by later changes.