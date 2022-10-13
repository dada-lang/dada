# Permissions on fields

import Caveat from '../caveat.md'

<Caveat/>

Like other variables, class fields have permissions. The `Point` class we've been working with, for example, declares its `x` and `y` fields to have `our` permission:

```
class Point(x: our, y: our)
#              ~~~     ~~~
```

We could also declare fields to have `my` permission, like this `Pair` class does:

```
class Pair(a: my, b: my)
```

Because the fields on `Pair` are declared as `my`, they will take ownership of the data stored in them. You can see that creating a `Pair` moves the values into the `Pair` by exploring examples like this one:

```
class Widget()
class Pair(a: my, b: my)

let w_a: my = Widget()
let w_b: my = Widget()
let pair: my = Pair(w_a, w_b)
print(pair).await                 # Prints `Pair(Widget(), Widget())`
print(w_a).await                  # Error: moved!
```

Once you create a `Pair`, you can also move values out from its fields. Try moving the cursor in this example to just after the `pair.a`:

```
class Widget()
class Pair(a: my, b: my)

let pair: my = Pair(Widget(), Widget())
let w_a: my = pair.a
#                   ▲
# ──────────────────┘
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

When you access a field, the permission you get is determined not only by the permission declared on the field itself but by the path you take to reach it. In particular, if you have `our` permission to an object, all of its `my` fields also become `our`, as you can see in this next. Assigning to `w_a: my` gets an error, because `pair.a` has the wrong permissions; try changing it to `w_a: our` and you will see that it works fine:

```
class Widget()
class Pair(a: my, b: my)

let pair: our = Pair(Widget(), Widget())
let w_a: my = pair.a                       # Error: `pair.a` has `our` permission
print(w_a).await
```

This might seem surprising, but think about it: if you have `our` permission, then there can be other variables that have `our` permission as well, and you can't _both_ have `my` permission to the fields. Otherwise, in an example like this, both `w_a1` and `w_a2` would have `my` permission to the same `Widget`, and that can't be:

```
class Widget()
class Pair(a: my, b: my)

let pair1: our = Pair(Widget(), Widget())
let pair2: our = pair1
let w_a1: my = pair1.a                       # Error: `pair.a` has `our` permission
let w_a2: my = pair2.a
```
