---
sidebar_position: 4
---

# Permissions

Dada hopefully feels familiar to you thus far, but if you played a lot with the programs, you may have noticed some errors you didn't expect. Consider this program...what do you expect it to print? Take a guess, and then hit the "Run" button to see what happens...

```dada ide
class Point(x, y)

p = Point(22, 44)
q = p
q.x := 23
print(p).await
```

Surprise! It gets an error! What is going on? The answer lies in the key Dada concept of **permissions**.

## What is a permission?

In Dada, variables don't just store a reference to an object, like they do in Python or Java. Instead, they store a reference to an object _with some permission_. These permissions determine whether you can read or write to the object.

Permissions in Dada can be divided across two axes. We'll cover those two axes separately:

- **Read** vs **write** -- covered now!
- **Owned** vs **leased** -- covered later, in the chapters on ownership

## Read permission is the default

When you write something like `q = p` in Dada, the default is that you get a **leased, read permission**. Leasing will be covered in more detail later, but for now it suffices to say that the permission for `q` is tied to the permission from `p`; when `p` goes out of scope, for example, then `q`'s permission will also be canceled.

As the name suggests, **read permissions** can only be used to read fields. This is why we get an error!

Dada comes equipped with a visual debugger that lets you visualize permissions. To see how it works, try hitting the "Debug" button and then position your cursor write after the line for `q = p`:

```dada ide
class Point(x, y)

p = Point(22, 44)
q = p
#    ▲
# ───┘
# put your cursor here -- you will see a diagram below
# that shows that while `p` and `q` reference the same
# point, `q` has read permissions (indicated with a blue
# line).
```

## Requesting write permission

You can explicitly request write permission by using the `lease` keyword, like `p.lease`. If you use the debugger and position it after `q = p.lease`, you will see that `q` is given write permission this time. As a result, `q.x := 23` succeeds and, when we print the variable `p`, we see the new value.

```dada ide
class Point(x, y)

p = Point(22, 44)
q = p.lease
q.x := 23
print(p).await
```
