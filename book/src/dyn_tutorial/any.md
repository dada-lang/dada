# The `any` permission

Rather than labeling variables as `my` or `our`, you can also use the `any` keyword. This will permit the variable to store an object with any permission. When using `any`, the `give` and `share` keywords allow you to control the ownership:

```
class Point(our x, our y)

# The point is `my` when first created
any my_p = Point(22, 44)

# You can `give` it to another variable
any my_p_now = my_p.give

# You can `share` it
any our_p = my_p_now.share

# Giving a shared thing is a copy
any also_our_p = our_p.give

# So is sharing
any and_our_p_too = our_p.share
```

## Using `any` to operate on multiple permissions with one function

The `any` permission is useful if you want to have functions that operate over multiple permissions. Consider the function `give_a`:

```
class Pair(my a, my b)

fn give_a(any pair) -> {
    pair.a.give
}
```

If `give_a` is called on a `my` object, it will return a `my` object, as shown here:

```
# class Pair(my a, my b)
# 
# fn give_a(any pair) -> {
#     pair.a.give
# }

class Widget(our name)
my my_pair = Pair(Widget("a"), Widget("b"))
my my_widget = give_a(my_pair)
print(my_widget).await                         # Prints 'Widget("a")'
print(my_pair).await                           # Error, my_pair has been given away
```

But if `give_a` is called on an `our` object, it will return an `our` object:

```
# class Pair(my a, my b)
# 
# fn give_a(any pair) -> {
#     pair.a.give
# }

class Widget(our name)
our our_pair = Pair(Widget("a"), Widget("b"))
our our_widget = give_a(our_pair)
print(our_widget).await                         # Prints 'Widget("a")'
print(our_pair).await                           # Prints 'Pair(Widget("a"), Widget("b"))'
```

## A hint of what's to come: generic functions

In Typed Dada, `any` functions become a shorthand for generic functions.

