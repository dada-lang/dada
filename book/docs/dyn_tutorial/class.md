---
sidebar_position: 2
---

# Declaring the `Point` class

import Caveat from '../caveat.md'

<Caveat/>

The main data structures in Dada are classes. The full class syntax has various bells and whistles, but let's start off with the simplest form. We'll define a class `Point` for storing `(x, y)` values. It will have two fields, `x` and `y`:

```
class Point(our x, our y)
#     ^^^^^ ^^^ ^
#       |    |  |
#       |    | Field name
#       |  Permission
#     Class name
```

## The `our` permission

You can see that the fields are declared with an *permission* -- `our` in this case. There are several permissions (`my`, `our`, `leased`, and `shleased`); these permissions are a key part of how Dada works, and we'll be covering them throughout this tutorial. For now, it's enough to say that an `our` field stores an object that may be referenced by other fields or variables as well (it belongs to everyone, hence `our`). For points, we are expecting to store integers like `22` and `44`, so this is a good fit.

## Constructor functions

The `class Point(..)` syntax also creates a constructor function that creates an instance of `Point` when called. Try executing the following code:

```
class Point(our x, our y)

# This function is declared as `async` because it
# awaits the result of print.
async fn print_point(p) {
    print("The point is: {p}").await
}

# This function is not declared as `async` because it
# doesn't await anything. The `->` indicates that it
# returns a value.
fn compute_new_value() -> {
    33
}

my p = Point(22, 44)
print_point(p).await
p.x := compute_new_value()
p.x += 1
print_point(p).await

# prints:
# The point is: Point(x: 22, y: 44)
# The point is: Point(x: 34, y: 44)
```

Some things to note here:

* Comments are written `#`, like Python or Ruby, not `//` like JavaScript or Rust.
* The `my p = ...` statement declares a local variable `p` using another permission, `my`. 
    * `my` declares a variable that has *unique* access to its object: in this case, we just created the `Point`, so nobody else could possibly be referencing it, so that makes sense. If that doesn't make sense yet, don't worry, we're going to talk all about `my` in the next section.
* You create a class by invoking its constructor function `Point(22, 44)`.
* Strings can embed expressions with `{}`, and they get "stringified", so `"The point is {p}"` embeds `p` into the string.
    * The default stringifier prints the values of each field.
* You write `:=` to reassign variables or fields (just `=` is for declaring a new variable).
    * You can also use the `+=`, `-=`, `*=`, `/=`, `%=` operators you may be familiar with from other languages.
* Declaring a function with `async fn` means that it can await thunks, like the one from calling `print`.
* Declaring a function with `->` means that it returns a value; as in Rust, the final expression in the function is its return value.
    * In this case, `compute_new_value()` returns `33`.

## Summary and key differences

So far, everything hopefully feels fairly familiar, except for the [permissions](./permissions.md), which we'll cover next. Some highlights:

* Declaring a class like `class Point(x, y)` also gives a constructor function `Point` that can be called (e.g., `Point(22, 44)`) to create an instance of the class.
* You can use `"{p}"` in strings to print the values of things.
* Use `=` to declare a new variable and `:=` or `+=` to update an existing one.
* Dada is based on async-await:
    * When you `print` something (or do any I/O), you must `await` the result for it to take effect (`print("Hi").await`).
    * You can only `await` things from inside an `async` fn.
