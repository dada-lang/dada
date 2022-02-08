# Giving permissions away

{{#include ../caveat.md}}

At the end of the previous tutorial, we were working with this program:

```
async fn main() {
    p = Point(x: 22, y: 44)
    print("The point is ({p.x}, {p.y})").await
}
```

We observed that `p` owned the `Point` which was created and that the `Point` was automatically freed after `p` finished with it.

## Assignments

Next, let's take a look at this program:

```
async fn main() {
    p = Point(x: 22, y: 44)
    q = p // <-- Added this line!
    print("The point is ({p.x}, {p.y})").await
}
```

If you run it, you will find that it gets an error:

```
error: `p` has no value
  > q = p // <-- Added this line!
            - value in `p` was given to `q` here
  > print("The point is ({p.x}, {q.y})).await
                          ^^^ `p` has no value
```

When you have an assignment like `q = p`, the default in Dada is that you are **giving** whatever permissions `p` has over to `q`. In this case, since `p` was the exclusive owner of the value, `q` becomes the exclusive owner of the value. You can't have two exclusive owners, so that means that `p` is empty. If you run the debugger, you can see this in action. Position the cursor right before the `p` in `q = p` line:

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44)
    q = p
    //     ▲
    // ────┘
    print("The point is ({p.x}, {p.y})").await
}
```

If you look at the state of the program, you will see:

```
┌───┐       ┌───────┐
│ p ├──my──►│ Point │
│   │       │ ───── │
│ q │       │ x: 22 │
└───┘       │ y: 44 │
            └───────┘
```

Now position the cursor at the end of the line and see how the state changes:

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44)
    q = p
    //       ▲
    // ──────┘
    print("The point is ({p.x}, {p.y})").await
}


┌───┐       ┌───────┐
│ p │       │ Point │
│   │       │ ───── │
│ q ├──my──►│ x: 22 │
└───┘       │ y: 44 │
            └───────┘
```

The `Point` is now owned by `q`!

Try changing the `print` to print from `q` instead of `p`...you will find the program works as expected.

## Making this explicit: the `give` keyword

If you prefer, you can make the move from `p` to `q` explicit by using the `give` keyword:

```
async fn main() {
    p = Point(x: 22, y: 44)
    q = p.give
    print("The point is ({q.x}, {q.y})").await
}
```

## Give can give more than ownership

Earlier, we said that the `give` keywords gives *all the permissions* from one place to another. That is true no matter how many or how few permissions you have. Right now, we're working with things we own, so `give` transfers ownership. As the tutorial proceeds, we're going to see ways that we can create variables with fewer permissions; using `give` on those variables will then give those fewer permissions.