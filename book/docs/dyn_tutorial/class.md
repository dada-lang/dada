---
sidebar_position: 2
---

# Declaring the `Point` class

import Caveat from '../caveat.md'

<Caveat/>

The main data structures in Dada are classes. The full class syntax has various bells and whistles, but let's start off with the simplest form. We'll define a class `Point` for storing `(x, y)` values. It will have two fields, `x` and `y`:

```
class Point(x, y)
#     ^^^^^ ^  ^
#       |   |  |
#       |   |  Field name
#       |  Field name
#     Class name
```

## Constructor functions

The `class Point(..)` syntax also creates a constructor function that creates an instance of `Point` when called. To get a feel for how classes work in practice, work with the following code. Feel free to make changes and see what happens! You'll also notice that when you move your cursor, the code executes up until the line you selected.

```dada ide
class Point(x, y)

# This function is declared as `async` because it
# awaits the result of print.
async fn print_point(p) {
    # Note that you can use `{...}` to embed an expression
    # into the value of a string (in this case, the
    # variable `p`).
    print("The point is: {p}").await
}

# Declares a function that computes a new value.
# (It doesn't await anything, so the function is not `async`.)
fn compute_new_value() -> {
    #                  ^^ this `->` indicates that
    #                     the function returns a value.
    33
}

# Writing `p = ...` declares a new variable `p`
# and assigns its initial value (`Point(22, 44)`)
p = Point(22, 44)

# Invoke `print_point`; it's an `async` function,
# so await the result
print_point(p).await

# The `:=` operator is used to modify an existing
# field (remember, `=` declares a new variable).
p.x := compute_new_value()

# You can also use `+=` to modify an existing field
# (this time by adding to it). Other operators, like
# `-=`, `*=`, `/=`, also work.
p.x += 1

# Print the new value.
print_point(p).await

# prints:
# The point is: Point(x: 22, y: 44)
# The point is: Point(x: 34, y: 44)
```
