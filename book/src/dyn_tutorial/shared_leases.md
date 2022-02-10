# Shared leases

{{#include ../caveat.md}}

Earlier, we saw that:

* You can use the [`share`](./share.md) keyword to convert unique ownership into *joint* ownership.
* You can use the [`lease`](./lease.md) keyword to temporarily lend out access to a variable without giving up ownership.

But what if you wanted to give many people access to the same object, but only for a limited time? You might want this, for example, so that you could mutate the object again. The answer is that you can combine sharing and leasing to create a **shared lease**:

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44)
    q = p.lease.share
    r = q
    print("p is ({p.x}, {p.y})").await
    print("q is ({q.x}, {q.y})").await
    print("r is ({r.x}, {r.y})").await
    p.x += 1
}
```

Let's take it step by step. First, position your cursor after `p.lease` (but before `.share`) and you will see:

```
┌───┐
│   │                  ┌───────┐
│ p ├╌my╌╌╌╌╌╌╌╌╌╌╌╌╌╌►│ Point │
│   │                  │ ───── │
│ q ├─leased(p)───────►│ x: 22 │
│   │                  │ y: 44 │
└───┘                  └───────┘
```

Just after the `lease`, we have that `q` is leased from `p`, the owner. Move the cursor after the `.share` and we see that the exclusive lease is now a shared lease, indicated by `our leased(p)`:

```
┌───┐
│   │                  ┌───────┐
│ p ├╌my╌╌╌╌╌╌╌╌╌╌╌╌╌╌►│ Point │
│   │                  │ ───── │
│ q ├─our leased(p)───►│ x: 22 │
│   │                  │ y: 44 │
└───┘                  └───────┘
```

Although the lease is now shared, `p` remains the owner of the point (and the lessor of the lease).

Next go past the `r = q.share` line. As always, sharing a shared thing simply reproduces it:

```
┌───┐
│   │                  
│ p ├╌my╌╌╌╌╌╌╌╌╌╌╌╌╌╌►┌───────┐
│   │                  │ Point │
│ q ├─our leased(p)───►│ ───── │
│   │                  │ x: 22 │
│ r ├─our leased(p)───►│ y: 44 │
│   │                  └───────┘
└───┘                  
```

Finally, if you move your cursor to after `p.x += 1` you will see that the lease has expired, so `q` and `r` have no value:

```
┌───┐
│   │                  ┌───────┐
│ p ├─my──────────────►│ Point │
│   │                  │ ───── │
│ q │                  │ x: 23 │
│   │                  │ y: 44 │
│ r │                  └───────┘
│   │                  
└───┘                  
```

## Leasing a shared value

If `x.lease.share` produces a shared lease, what do you think happens with `x.share.lease`? In other words, what happens if we try to lease a shared value? 

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44).share
    q = q.lease
}
```

The answer is that `p` remains the joint *owner* of the point, but `q` has a shared lease:

```
┌───┐
│   │                  ┌───────┐
│ p ├╌our╌╌╌╌╌╌╌╌╌╌╌╌╌►│ Point │
│   │                  │ ───── │
│ q ├─our leased(p)───►│ x: 22 │
│   │                  │ y: 44 │
└───┘                  └───────┘
```

There is one interesting wrinkle here. Ordinarily, if you [lease an object](./lease.md), then the lease is cancelled when the original object is used again. But if you have a shared lease, it's ok to continue using the original object, since the only thing that both of you can do is to read from it:


```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44).share
    q = q.lease

    # Reading from `p` does not cancel the lease `q`
    print(p).await

    # Reading from `q` still works:
    print(q).await
}

# Prints:
#
# our Point(22, 44)
# our leased Point(22, 44)
```
