---
sidebar_position: 3
---

# Aside: Labeled arguments

Before we go further with the tutorial, it's worth nothing that Dada supports _labeled arguments_. That means that instead of writing `Point(22, 44)` one can also give labels to each argument, like `Point(x: 22, y: 44)`:

```dada ide
class Point(x, y)

my p = Point(x: 22, y: 44)
print("The point is `{p}`").await
```

Try changing the code above to give the parameters in another orde, such as `Point(y: 44, x: 22)` -- you will see that the output doesn't change.

Adding labels can help make it clearer what is going on. The rules are as follows:

- You must also give the arguments in the order in which they were declared in the function, whether or not labels were provided.
- Once you give a label to a parameter, you must give a label to all the remaining parameters (so you can't do `Point(x: 22, yy)` but you can do `Point(22, y: 44)`.

Dada will also sometimes suggest you use labels if it thinks you might be making a mistake. For example, try this:

```dada ide
class Point(x, y)

async fn print_line(start, end) {
    print("The start is {start}").await
    print("The end is {end}").await
}

start = Point(22, 44)
end = Point(33, 55)
print_line(end, start).await
#          ~~~~~~~~~~ warning: are these parameters in the right order?
```

See the squiggly line? That is Dada telling us that we may have reversed the order of `end` and `start`. We can disable this warning by giving explicit labels to the arguments, making it clear that we _meant_ to switch the order:

```dada ide
class Point(x, y)

async fn print_line(start, end) {
    print("The start is {start}").await
    print("The end is {end}").await
}

start = Point(22, 44)
end = Point(33, 55)
print_line(start: end, end: start).await
```
