# Creating, dropping, and scopes

{{#include ../caveat.md}}

## Running example

For the next several chapters, we're going to work with variations on this example program:

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44)
    print("The point is ({p.x}, {p.y})").await
}
```

## Permissions and ownership

In Dada, variables always track both an object and a set of *permissions* that they have for using that object. These permissions control what you can do with the object:

* Can you **write** to it or its fields?
* Can you **read** from it?
* Can you **drop** it, freeing its memory?

When you invoke a constructor, you get back a freshly created object with full permissions. We call this being the unique owner -- you're the only one who has access to that object, and you can do anything you want with it. In our code above, that means that `p` is the owner. Once we have finished using `p`, the variable is going to be dropped and the `Point` is going to be freed. Note that this is not like garbage collection, where the memory *might* get freed -- it is a guarantee. This is important both because it ensures that Dada programs have tight memory usage and because it means that you can [rely on other sorts of resources, like files and sockets, being disposed of promptly][raii-wycats].

[raii-wycats]: https://blog.skylight.io/rust-means-never-having-to-close-a-socket/

### Exploring ownership with the debugger

We can use the interactive debugger to explore permissions and watch how they evolve. To start, run the program below and move the cursor to the start of the `print` line. This will take you to the point right after `p = Point(..)` has executed. If you look at the state of the program, you will see:

```
┌───┐       ┌───────┐
│ p ├──my──►│ Point │
└───┘       │ ───── │
            │ x: 22 │
            │ y: 44 │
            └───────┘
```

The label `my` on the edge from `p` to the `Point` object is telling you that `p` *owns* the `Point`. 

### The Point is freed

If you step forward in the debugger past the `print`, you will see that `p` has been freed:

```
class Point(x, y)

async fn main() {
    p = Point(x: 22, y: 44)
    print("The point is ({p.x}, {p.y})").await
    //                                        ▲
    // ───────────────────────────────────────┘
}

┌───┐
│   │
└───┘
```

What is going on? The answer is that once you get to the point where there `p` won't be used again, the variable `p` is destroyed. If `p` owns its contents, then its contents are also destroyed (and this proceeds recursively; so e.g. if `p` stored a vector of owned objects, they would be destroyed too).[^precise]

## Footnotes

[^precise]: The *precise* time when `p` gets destroyed depends on the compiler's analysis, but it will always occur some time after the last use of `p` and some time before `p` goes out of scope.
