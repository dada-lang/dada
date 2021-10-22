# Giving permissions away

{{#include ../caveat.md}}

At the end of the previous tutorial, we were working with this program:

```
async fn main() {
    var q
    do {
        var p = Point(x: 22, y: 44)
        print("The point is ({p.x}, {p.y})").await
        q := p
    }
    print("The point q is ({q.x}, {q.y})).await
}
```

We saw that `q = p` *shared* the permissions of `p` with `q`; since `p` was still the owner, the point was dropped as we exited the `do { .. }` block, and then we got an error as `q` had no value. But what if we wanted `q` to become the new owner? We can do that with the `give` keyword:

```
async fn main() {
    var q
    do {
        var p = Point(x: 22, y: 44)
        print("The point is ({p.x}, {p.y})").await
        q := p.give
    }
    print("The point q is ({q.x}, {q.y})).await
}
```

If you run the program now, you will find it prints

```
The point is (22, 44)
The point q is (22, 44)
```

If you position the cursor to right after the line `q = p.give`, you will see the [following](https://asciiflow.com/#/share/eJyrVspLzE1VssorzcnRUcpJrEwtUrJSqo5RqohRsrK0MNGJUaoEsozArJLUihIgJ0bp0ZQ9yCgmJg9IKigoINiFCnDZ3Eo05cQgZCMVsAC4EmSEVeE0vGYRdkIBDm0KAfmZeSUKBNwKEkJ3JVFuBwlVWCkYGUGVY%2FMbWFGllYKJCQFFOL2oVKtUCwBpCOYz)):

```
┌───┐
│   │
│ q ├──my────────────────┐
│   │                    │
├───┤                    ▼
│   │                ┌───────┐
│ p │                │ Point │
│   │                │ ───── │
└───┘                │ x: 22 │
                     │ y: 44 │
                     └───────┘
```

In other words, `p` gave up all of its permissions and they now belong to `q`. 

### Use after give

Question: What do you think will happen if you try to use `p` after `q = p.give`? Try it and see!

```
async fn main() {
    var q
    do {
        var p = Point(x: 22, y: 44)
        q := p.give
        print("The point is ({p.x}, {p.y})").await
    }
    print("The point q is ({q.x}, {q.y})).await
}
```

As you might expect, running this program results in an error:

```
error: accessing variable with no value
  >     q = p.give
            - `p` gave up its value here
  >     print("The point is ({p.x}, {p.y})").await
                              ^^^ p has no value
```
