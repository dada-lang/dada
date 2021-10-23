# Declaring the `Point` class

{{#include ../caveat.md}}

The main data structures in Dada are classes. The full class syntax has various bells and whistles, but let's start off with the simplest form. We'll define a class `Point` for storing `(x, y)` values:

```
class Point(var x, var y)
//    ^^^^^ ^^^ ^
//      |    |  |
//      |    | Field name
//      |   Storage mode
//    Class name
```

## Field storage modes

This declares a class `Point` with two fields, `x` and `y`. For each field, besides its name, there is a "storage mode":

* `shared`: Shared fields cannot be reassigned.
    * If you're familiar with Java, shared fields are similar to `final` fields, although with some important differences that we will cover later [when we cover sharing and shared storage](./shared_storage.md).
    * Shared fields are the default if you say nothing (e.g., `class Point(x, y)` would have shared fields).[^good]
* `var`: Variable fields (as we see here) can be reassigned.
    * `var` fields are the most common kind of mutable field, and are kind of analogous to "normal fields" in Java.
* `atomic`: Atomic fields allow you to coordinate mutation for shared data structures, particularly those across multiple threads. 
    * We're going to ignore them for now, but we cover them in the [atomic section](./atomic.md).

## Constructor functions

The `class Point(..)` syntax also creates a constructor function that creates an instance of `Point` when called. Try executing the following code:

```
async fn main() {
    var p = Point(x: 22, y: 44)
    print("The point is {p}").await
    print("The point is ({p.x}, {p.y})").await
    p.x := 33
    p.x += 1
    print("The point is now ({p.x}, {p.y})").await
}

// prints:
// The point is Point(x: 22, y: 44)
// The point is (22, 44)
// The point is now (34, 44)
```

Some things to note here:

* The `var p = ...` statement declares a local variable `p`
    * Just like fields, variables have a storage mode, and `var` indicates a variable whose value can be modified.
    * If we had written `p = Point(x: 22, y: 44)` that would have created a shared local variable that doesn't permit assignment (try modifying it and see what happens!).
* You create a class by invoking its constructor function `Point(x: 22, y: 44)`
    * Function calls in Dada, like in Swift, support labeled arguments.
    * Labels are mandatory unless there is a single argument.
* There is a default "stringify" for classes that prints out the name of the class and stringifies its fields.
    * This is what you see with `"{p}"`.
    * It can be overridden, but we won't discuss that now.
* You write `:=` to reassign variables or fields (just `=` is for declaring a new variable).
    * You can also use the `+=`, `-=`, `*=`, `/=`, `%=` operators you may be familiar with from other languages.

## Summary and key differences

OK, everything so far hopefully feels fairly familiar. Here are the main highlights:

* Fields and variables have a storage mode (`shared`, `var`, or `atomic`). We've only used `var` so far.
* Function calls have labeled arguments.
* You can use `"{p}"` in strings to print the values of things.
* Use `=` to declare a new variable and `:=` or `+=` to update an existing one.

## Footnotes

[^good]: Shared fields might actually be a good choice for points: that would mean that we can't modify the `x` or `y` field independently but must create a new `Point`. However, we use `var` fields here because they let us illustrate more aspects of Dada later in the tutorial.
