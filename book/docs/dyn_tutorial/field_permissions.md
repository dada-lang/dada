# Permissions on fields

import Caveat from '../caveat.md'

<Caveat/>

Like other variables, class fields have permissions. The `Point` class we've been working with, for example, declares its `x` and `y` fields to have `our` permission:

```
class Point(our x, our y)
#           ~~~    ~~~
```

We could also declare fields to have `my` permission, like this `Pair` class does:

```
class Pair(my a, my b)
```

Because the fields on `Pair` are declared as `my`, they will take ownership of the data stored in them. You can see that creating a `Pair` moves the values into the `Pair` by exploring examples like this one:

```
class Widget()
class Pair(my a, my b)

my w_a = Widget()
my w_b = Widget()
my pair = Pair(w_a, w_b)
print(pair).await                 # Prints `Pair(Widget(), Widget())`
print(w_a).await                  # Error: moved!
```

Once you create a `Pair`, you can also move values out from its fields. Try moving the cursor in this example to just after the `pair.a`:

```
class Widget()
class Pair(my a, my b)

my pair = Pair(Widget(), Widget())
my w_a = pair.a
#              ▲
# ─────────────┘
print(w_a).await                       # Prints `Widget()`

# You see:
# 
# ┌──────┐       ┌──────┐
# │ pair ├──my──►│ Pair │
# │      │       │ ──── │
# │      │       │ a    │       ┌────────┐
# │      │       │ b:   ├──my──►│ Widget │
# │      │       └──────┘       └────────┘
# │      │       ┌────────┐
# │ w_a  ├──my──►│ Widget │
# └──────┘       └────────┘
```

## Inherited `our` permissions

When you access a field, the permission you get is determined not only by the permission declared on the field itself but by the path you take to reach it. In particular, if you have `our` permission to an object, all of its `my` fields also become `our`, as you can see in this next. Assigning to `my w_a` gets an error, because `pair.a` has the wrong permissions; try changing it to `our w_a` and you will see that it works fine:

```
class Widget()
class Pair(my a, my b)

our pair = Pair(Widget(), Widget())
my w_a = pair.a                       # Error: `pair.a` has `our` permission
print(w_a).await
```

This might seem surprising, but think about it: if you have `our` permission, then there can be other variables that have `our` permission as well, and you can't *both* have `my` permission to the fields. Otherwise, in an example like this, both `w_a1` and `w_a2` would have `my` permission to the same `Widget`, and that can't be:

```
class Widget()
class Pair(my a, my b)

our pair1 = Pair(Widget(), Widget())
our pair2 = pair1
my w_a1 = pair1.a                       # Error: `pair.a` has `our` permission
my w_a2 = pair2.a
```

