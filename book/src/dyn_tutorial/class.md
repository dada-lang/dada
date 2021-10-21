# Declaring the `Point` class

{{#include caveat.md}}

Let's try something a bit more interesting. We'll define a class `Point` for storing `(x, y)` values. The simplest syntax for declaring a class in Dada is as follows:

```
class Point(var x, var y)
```

This declares a class `Point` with two fields, `x` and `y`. It also defines a constructor function `Point`, so you can create an instance of `Point` by calling that function. Try executing the following code:

```
async fn main() {
    var p := Point(x: 22, y: 44)
    print("The point is {p}").await
    print("The point is ({p.x}, {p.y})").await
    p.x += 1
    print("The point is now ({p.x}, {p.y})").await
}

// prints:
// The point is Point(x: 22, y: 44)
// The point is (22, 44)
// The point is now (23, 44)
```

Some things to note here:

* `var p := ...` declares a local variable (we also saw `var` when declaring a field)
    * More precisely, `var` declares a mutable local variable. We'll see other ways to declare locals later on.
* You create a class by invoking its constructor function `Point(x: 22, y: 44)`
    * Function calls in Dada, like in Swift, support labeled arguments.
    * Labels are mandatory unless there is a single argument.
* There is a default "stringify" for classes that prints out the name of the class and stringifies its fields.
    * This is what you see with `"{p}"`.
    * It can be overridden, but we won't discuss that now.
* You can access and mutate the values of fields in the usual way (e.g., `p.x`, `p.x += 1`).

OK, everything so far probably feels fairly familiar. Let's start learning about the new things!
