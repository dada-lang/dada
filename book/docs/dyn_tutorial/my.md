---
sidebar_position: 5
---

# I me, me, mine: The my permission

import Caveat from '../caveat.md'

<Caveat/>

The `my` permission is in many ways the most basic. It declares that a variable has **unique ownership** of an object. Just like owning a house, having ownership means that the variable has full, irrevokable access to that object.

## Ownership vs leasing

In many ways, owned permissions ought to be familiar to you, because they are most like other languages. In Java or JavaScript, for example, if you have access to an object, you have a kind of "ownership" over it -- you can keep using it as long as you like, or copy it to other variables. In Dada, that's not a given. Owned permissions are permanent, but the mutable permissions we'll see later can be canceled.

## Unique: me and nobody else

But we said that `my` represents **unique** ownership -- what does it mean that the `my` permission is **unique**? It means there are no other variables that can access the object. So, you might wonder, what happens if we copy the object into another "unique" variable? Well, let's try it and see!

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
let q: my = p # <--- added this line
print("The point is {p}").await
```

If you run it, you will find that it gets an error:

<!-- FIXME: no error is emitted for the code above -->

```
error: `p` has no value
  > my q = p # <-- Added this line!
        - value in `p` was given to `q` here
  > print("The point is {p}").await
                        ^^^ `p` has no value
```

When you assign to the `q: my` variable, you are actually **giving** ownership from `p` to `q`. You can't have two unique owners, so that means that `p` is empty.

## Visualizing permissions with the debugger

Dada comes equipped with a visual debugger that can help you to understand how permissions work. Let's try it! Position the cursor at the end of the first line:

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
#                       ▲
# ──────────────────────┘
let q: my = p
print("The point is {p}").await

# You see:
#
# ┌───┐       ┌───────┐
# │ p ├──my──►│ Point │
# │   │       │ ───── │
# │ q │       │ x: 22 │
# └───┘       │ y: 44 │
#             └───────┘
```

Now position the cursor at the end of the next line and see how the state changes:

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
let q: my = p
#            ▲
# ───────────┘
print("The point is {p}").await

# You see:
#
# ┌───┐       ┌───────┐
# │ p │       │ Point │
# │   │       │ ───── │
# │ q ├──my──►│ x: 22 │
# └───┘       │ y: 44 │
#             └───────┘
```

The `Point` is now owned by `q`!

Try changing the `print` to print from `q` instead of `p`...you will find the program works as expected.

## Calling a function can give ownership, too

What do you think happens when we run this code?

```
class Point(x: our, y: our)

fn take_point(point: my) { }

let p: my = Point(22, 44)
take_point(p)
print(p).await
```

If you guessed "error", you were right! Check it out:

<!-- FIXME: no error is emitted for the code above -->

```
error: `p` has no value
  > take_point(p)
               - value in `p` was given away here
  > print(p).await
          ^ `p` has no value
```

What this example shows is that calling a function whose parameters are declared as `my` transfers ownership to those parameters in just the same way as declaring a `my` local variable. The same holds when calling a class constructor:

```
class Point(x: our, y: our)
class Line(start: my, end: my)

let start: my = Point(22, 44)
let end: my = Point(33, 55)
let line1: my = Line(start, end)
let line2: my = Line(start, end) # Error
```

## Making this explicit: the `give` keyword

If you prefer, you can make the move from `p` to `q` explicit by using the `give` keyword:

```
class Point(x: our, y: our)

let p: my = Point(22, 44)
let q: my = p.give
#             ~~~~ this is new
print("The point is ({p.x}, {p.y})").await
```

## Give can give more than ownership

Earlier, we said that the `give` keywords gives _all the permissions_ from one place to another. That is true no matter how many or how few permissions you have. Right now, we're working with things we own, so `give` transfers ownership. As the tutorial proceeds, we're going to see ways that we can create variables with fewer permissions; using `give` on those variables will then give those fewer permissions.
