# Leasing permissions

{{#include caveat.md}}

In the previous chapter, we talked about *giving permissions away*. But sometimes we would just like to give temporary access; this is where the `lease` keyword comes in. Consider the following program:

```
class Point(var x, var y)

async fn main() {
    var p := Point(x: 22, y: 44)
    var q := p.lease
    q.x += 1
    print("The point is ({p.x}, {p.y})").await
}
```

Here, we added the line `var q := p.lease`. What that does is to create a *leased* copy of the `Point`. When you lease an object, you are temporarily getting permission to access it. The variable you are getting the permissions *from* (here, `p`) is called the *lessor*. In this case, the lessor is also the owner of the object, but that's not always true: you can have subleases.

## Unique leases

The default lease is an **unique** lease. That means that the new variable has exclusive access to the object. So long as the lease is active, all reads and writes to that object have to go through the leased variable (`q`) or some sublease of `q`. In the next chapter, we'll talk about shared leases, which can be accessed from many variables (we actually saw a shared lease earlier, in the [chapter on creating and dropping objects](create.md)).

Because `q` has a unique lease to the `Point`, it is able to modify the fields of the `Point`. Let's explore this in the debugger. Position your cursor right before `q.x += 1` and you will [see](https://asciiflow.com/#/share/eJyrVspLzE1VssorzcnRUcpJrEwtUrJSqo5RqohRsrK0MNKJUaoEsowsLIGsktSKEiAnRunRlD3IKCYmD0gqKChASDSAphiLxgKomtxKPGrR0bRdIK0B%2BZl5JQrEuAHdHUiaCqFyOamJxakpGgWa%2BGyssFIwMiLOxkorBRMThFIU20kKJqVapVoAp%2FrUlQ%3D%3D):

```
┌───┐
│   │                  ┌───────┐
│ p ├─my──────────────►│ Point │
│   │                  │ ───── │
│ q ├─leased(p)───────►│ x: 22 │
│   │                  │ y: 44 │
└───┘                  └───────┘
```

The `leased(p)` permission here says that `q` is *leased from* `p` (this implies a unique lease). If we then go to the next line, we see that the value of `x` changes:

```
  ┌───┐
  │   │                  ┌───────┐
  │ p ├─my──────────────►│ Point │
  │   │                  │ ───── │
  │ q ├─leased(p)───────►│ x: 23 │
  │   │                  │ y: 44 │
  └───┘                  └───────┘
```

## Ending a lease

Leases last until the lessor chooses to end them. Lessors end a lease by taking some action that violates the terms of the lease: here, since `q` has an exclusive lease, `p` can end the lease by reading or writing from the point, as that implies that `q` no longer has exclusive access. Since the `print` statement reads from `p` (and not `q`), we can [see](https://asciiflow.com/#/share/eJyrVspLzE1VssorzcnRUcpJrEwtUrJSqo5RqohRsrK0MNOJUaoEsowsDYGsktSKEiAnRunRlD3IKCYmD0gqKChASDSAphiLxgKomtxKPGrR0bRdIK0B%2BZl5JQoE3YDhCiQthbi0VFgpGBkRY3allYKJCUIhij0kBYdSrVItAJuouas%3D) that lease ends as we step through that line in the debugger:

```
┌───┐
│   │                  ┌───────┐
│ p ├─my──────────────►│ Point │
│   │                  │ ───── │
│ q │                  │ x: 22 │
│   │                  │ y: 44 │
└───┘                  └───────┘
```

What do you think will happen if we try to use `q` again? Try it and see!