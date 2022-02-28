# Declaring the `Point` class

{{#include ../caveat.md}}

The main data structures in Dada are classes. The full class syntax has various bells and whistles, but let's start off with the simplest form. We'll define a class `Point` for storing `(x, y)` values. It will have two fields, `x` and `y`:

```
class Point(x, y)
#     ^^^^^ ^
#       |   |
#       |   Field name
#       |
#     Class name
```

## Constructor functions

The `class Point(..)` syntax also creates a constructor function that creates an instance of `Point` when called. Try executing the following code:

```
class Point(x, y)

async fn main() {
    my p = Point(22, 44)
    print("The point is `{p}`").await
    print("The point is `({p.x}, {p.y})`").await
    p.x := 33
    p.x += 1
    print("The point is now `({p.x}, {p.y})`").await
}

# prints:
# The point is `Point(x: 22, y: 44)`
# The point is `(22, 44)`
# The point is now `(34, 44)`
```

Some things to note here:

* The `my p = ...` statement declares a local variable `p`.  
    * The keyword `my` is called an [ownership permission](./permissions.md), and we'll be diving into those shortly!
* You create a class by invoking its constructor function `Point(22, 44)`.
* Strings can embed expressions with `{}`, and they get "stringified", so `"The point is {p}"` embeds `p` into the string.
    * The default stringifier prints the values of each field.
* You write `:=` to reassign variables or fields (just `=` is for declaring a new variable).
    * You can also use the `+=`, `-=`, `*=`, `/=`, `%=` operators you may be familiar with from other languages.
* Comments are written `#`, like Python or Ruby, not `//` like JavaScript or Rust.

## Summary and key differences

So far, everything hopefully feels fairly familiar, except for the [ownership permission](./permissions.md) `my`, which we'll cover next. Some highlights:

* Declaring a class like `class Point(x, y)` also gives a constructor function `Point` that can be called (e.g., `Point(22, 44)`) to create an instance of the class.
* You can use `"{p}"` in strings to print the values of things.
* Use `=` to declare a new variable and `:=` or `+=` to update an existing one.
* Dada is based on async-await:
    * When you `print` something (or do any I/O), you must `await` the result for it to take effect (`print("Hi").await`).
    * You can only `await` things from inside an `async` fn.
