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
    p = Point(22, 44)
    print("The point is `{p}`").await
    print("The point is `({p.x}, {p.y})`").await
    p.x := 33
    p.x += 1
    print("The point is now `({p.x}, {p.y})`").await
}

# prints:
# The point is `my Point(x: 22, y: 44)`
# The point is `(22, 44)`
# The point is now `(34, 44)`
```

Some things to note here:

* The `p = ...` statement declares a local variable `p`.
* You create a class by invoking its constructor function `Point(22, 44)`.
* Strings can embed expressions with `{}`, and they get "stringified", so `"The point is {p}"` embeds `p` into the string.
    * The default stringifier prints the values of each field, but also the *ownership mode* (`my`, in this case). We'll talk about ownership next.
* You write `:=` to reassign variables or fields (just `=` is for declaring a new variable).
    * You can also use the `+=`, `-=`, `*=`, `/=`, `%=` operators you may be familiar with from other languages.
* Comments are written `#`, like Python or Ruby, not `//` like JavaScript or Rust.

## Labeled arguments

One other thing worth nothing is that Dada supports *labeled argments*. That means that instead of writing `Point(x, y)` one can also give labels to each argument:

```
class Point(x, y)
async fn main() {
    p = Point(x: 22, y: 44)
    print("The point is `{p}`").await
}

# prints:
# The point is `my Point(x: 22, y: 44)`
```

Adding labels can help make it clearer what is going on. The rules are as follows:

* You must also give the arguments in the order in which they were declared in the function, whether or not labels were provided.
* Once you give a label to a parameter, you must give a label to all the remaining parameters (so you can't do `Point(x: 22, yy)` but you can do `Point(22, y: 44)`.

Dada will also sometimes suggest you use labels if it thinks you might be making a mistake. For example, try this:

```
async fn main() {
    var x = 22
    var y = 44
    var p = Point(y, x)
}
```

See how we reversed the order of `y` and `x`? If we try to compile this, Dada will warn us and suggest that this might've been a mistake. We can disable this warning by giving explicit labels to the arguments, making it clear that we *meant* to switch the order:


```
async fn main() {
    var x = 22
    var y = 44
    var p = Point(x: y, y: x)
}
```

## Summary and key differences

OK, everything so far hopefully feels fairly familiar. Here are the main highlights:

* You can use `"{p}"` in strings to print the values of things.
* Use `=` to declare a new variable and `:=` or `+=` to update an existing one.

There is also this thing called ownership. What's that? (Go to the next chapter to find out!)
