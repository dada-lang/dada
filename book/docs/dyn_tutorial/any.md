# The `any` permission

import Caveat from '../caveat.md'

<Caveat/>

Rather than labeling variables as `my` or `our`, you can also use the `any` keyword. This will permit the variable to store an object with any permission. When using `any`, the `give` and `share` keywords allow you to control the ownership:

```
class Point(x: our, y: our)

# The point is `my` when first created
let my_p: any = Point(22, 44)

# You can `give` it to another variable
let my_p_now: any = my_p.give

# You can `share` it
let our_p: any = my_p_now.share

# Giving a shared thing is a copy
let also_our_p: any = our_p.give

# So is sharing
let and_our_p_too: any = our_p.share
```

## Using `any` to operate on multiple permissions with one function

The `any` permission is useful if you want to have functions that operate over multiple permissions. Consider the function `give_a`:

```
class Pair(a: my, b: my)

fn give_a(pair: any) -> {
    pair.a.give
}
```

If `give_a` is called on a `my` object, it will return a `my` object, as shown here:

```
# class Pair(a: my, b: my)
#
# fn give_a(pair: any) -> {
#     pair.a.give
# }

class Widget(name: our)
let my_pair: my = Pair(Widget("a"), Widget("b"))
let my_widget: my = give_a(my_pair)
print(my_widget).await                         # Prints 'Widget("a")'
print(my_pair).await                           # Error, my_pair has been given away
```

But if `give_a` is called on an `our` object, it will return an `our` object:

```
# class Pair(a: my, b: my)
#
# fn give_a(pair: any) -> {
#     pair.a.give
# }

class Widget(name: our)
let our_pair: our = Pair(Widget("a"), Widget("b"))
let our_widget: our = give_a(our_pair)
print(our_widget).await                         # Prints 'Widget("a")'
print(our_pair).await                           # Prints 'Pair(Widget("a"), Widget("b"))'
```

## A hint of what's to come: generic functions

In Typed Dada, `any` functions become a shorthand for generic functions.
