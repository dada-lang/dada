# Creating, dropping, and scopes

{{#include ../caveat.md}}

## Running example

For the next several chapters, we're going to work with variations on this example program:

```
class Point(var x, var y)

async fn main() {
    var p = Point(x: 22, y: 44)
    print("The point is ({p.x}, {p.y})").await
}
```

## Permissions and ownership

In Dada, variables always track both an object and a set of *permissions* that they have for using that object. These permissions control what you can do with the object:

* Can you **write** to it or its fields?
* Can you **read** from it?
* Can you **drop** it, freeing its memory?

When you invoke a constructor, you get back a freshly created object with full permissions. We call this being the unique owner -- you're the only one who has access to that object, and you can do anything you want with it. In our code above, that means that `p` is the owner. When `p` goes out of scope, the `Point` is going to be freed. Note that this is not like garbage collection, where the memory *might* get freed -- it is a guarantee. This is important both because it ensures that Dada programs have tight memory usage and because it means that you can [rely on other sorts of resources, like files and sockets, being disposed of promptly][raii-wycats].

[raii-wycats]: https://blog.skylight.io/rust-means-never-having-to-close-a-socket/

### Exploring ownership with the debugger

We can use the interactive debugger to explore permissions and watch how they evolve. To start, run the program below and move the cursor to the start of the `print` line. This will take you to the point right after `var p = Point(..)` has executed. If you look at the state of the program, you will see:

```
┌───┐       ┌───────┐
│ p ├──my──►│ Point │
└───┘       │ ───── │
            │ x: 22 │
            │ y: 44 │
            └───────┘
```

The label `my` on the edge from `p` to the `Point` object is telling you that `p` *owns* the `Point`. If you step forward in the debugger to the `}`, you will see that when we return from `main`, `p` has been freed:

```
┌───┐
│   │
└───┘
```

### Multiple aliases

Let's play wit the concept of ownership and drops a bit more. To start, we have to introduce the idea of a "subblock". In Dada, we can use the `do` function to create a sub-block like so:[^closure]

[^closure]: What's actually happening here is that the `{...}` block is a closure and the `do` function is being called with that closure as argument. `do` is a very simple function that simply invokes its argument.

```
async fn main() {
    do {
        var p = Point(x: 22, y: 44)
        print("The point is ({p.x}, {p.y})").await
    } // <-- p will get freed as we exit this block
}
```

As the comment notes, we are now creating `p` inside a subblock, and when that subblock exits, `p` will get freed. Guaranteed. So what happens, you may ask, if we have another variable that is pointing at the same point:

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

Here we introduce a bit of new syntax: `var q` by itself simply creates a variable without giving it any value. Later on, when we write `q = p`, we are reassigning the value of that variable. (Note that we didn't write `q = p`; the `=` syntax is only for declaring *new* variables, not reassigning existing ones.)

So, what happens when we try to print the value of `q` at the end of `main`? Try it and see! Clicking run yields:

```
error: accessing freed object
  >     var p = Point(x: 22, y: 44)
            - object was freed when `p` went out of scope
  > print("The point q is ({q.x}, {q.y})).await
                            ^^^ q.x refers to a freed object   
```

Now, move the cursor to the start of the line `q = p`. This will take you to the point right before `q = p` executed. If you look at the state of the program, [you will see](https://asciiflow.com/#/share/eJyrVspLzE1VssorzcnRUcpJrEwtUrJSqo5RqohRsrK0MNGJUaoEsozArJLUihIgJ0bp0ZQ9yCgmJg9IKigoINiFCljFcelSQANoSrFoK0Coya3EoxyKpu0CaQrIz8wrQXcZFrvR7cfuAWwaK6wUjIygytHl4YoqrRRMTAgowul9pVqlWgBzC8%2FT):

```
┌───┐
│   │
│ q │
│   │
├───┤
│   │                ┌───────┐
│ p ├──my───────────►│ Point │
│   │                │ ───── │
└───┘                │ x: 22 │
                     │ y: 44 │
                     └───────┘
```

You can see that there are two blocks on the stack. The outer one contains `q` (with no value) and the inner one (the `do { ... }`) contains `p`. `p` is the owner of the `Point` object we just created. Moving the cursor to the next line [shows](https://asciiflow.com/#/share/eJyrVspLzE1VssorzcnRUcpJrEwtUrJSqo5RqohRsrK0MNGJUaoEsozArJLUihIgJ0bp0ZQ9yCgmJg9IKigoINiFCnDZ4ozEotQUjQJNNF14DFHAAuBKkBFWhdPwmkXYCQUINbmV%2BB0NQtN2gTQF5GfmlSAFAC670e0nzl8goQorBSMjqHJs%2FgYrqrRSMDEhoAin95VqlWoB3SX2yQ%3D%3D):

```
┌───┐
│   │
│ q ├──shared(p)─────────┐
│   │                    │
├───┤                    ▼
│   │                ┌───────┐
│ p ├──my───────────►│ Point │
│   │                │ ───── │
└───┘                │ x: 22 │
                     │ y: 44 │
                     └───────┘
```

Now we can see that `p` remains the owner, but `q` is also referring to the same `Point` object. The permissions on `q` are interesting: `shared(p)`. We'll dig into shared permissions in more detail later, but for now it suffices to say that this means "`q` has permission to read from the object owned by `p`".

If we click to the *next* line, right after the `do { ... }` block as executed, we see the following:


```
┌───┐
│   │
│ q │
│   │
└───┘
```

What this indicates is that after `p` went out of scope, it freed its object, and so `q` was left with no value. When we now try to use `q`, we get an error.

