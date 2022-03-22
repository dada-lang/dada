# Aside: Labeled argments

Before we go further with the tutorial, it's worth nothing that Dada supports *labeled arguments*. That means that instead of writing `Point(22, 44)` one can also give labels to each argument, like `Point(x: 22, y: 44)`:

```
class Point(our x, our y)

async fn main() {
    my p = Point(x: 22, y: 44)
    print("The point is `{p}`").await
}

# prints:
# The point is `Point(x: 22, y: 44)`
```

Adding labels can help make it clearer what is going on. The rules are as follows:

* You must also give the arguments in the order in which they were declared in the function, whether or not labels were provided.
* Once you give a label to a parameter, you must give a label to all the remaining parameters (so you can't do `Point(x: 22, yy)` but you can do `Point(22, y: 44)`.

Dada will also sometimes suggest you use labels if it thinks you might be making a mistake. For example, try this:

```
class Point(our x, our y)

async fn print_line(my start, my end) {
    print(start).await
    print(end).await
}

async fn main() {
    my start = Point(22, 44)
    my end = Point(33, 55)
    print_line(end, start).await
    #          ~~~~~~~~~~ warning: are these parameters in the right order?
}
```

See the squiggly line? That is Dada telling us that we may have reversed the order of `end` and `start`. We can disable this warning by giving explicit labels to the arguments, making it clear that we *meant* to switch the order:

```
class Point(our x, our y)

async fn print_line(my start, my end) {
    print(start).await
    print(end).await
}

async fn main() {
    my start = Point(22, 44)
    my end = Point(33, 55)
    print_line(start: end, end: start).await
}
```