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

When you write `q = p`, the default is to get _shared_ permission.

## The shared permission

The answer lies in the `q = p` statement.

You may have noticed

In the previous chapter, we saw keywords like `my` and `our` attached to variables. These are examples of **permissions** -- permissions are a key part of Dada. Like Rust, Dada leverages permissions to avoid the need for a garbage collector, while retaining memory safety.

## Running example

As we explain permissions both here and over the next several chapters, we're going to work with variations on this example program. It builds on syntax that we [introduced previously](./class.md), so if anything is confusing you may want to check out that chapter.

```
class Point(our x, our y)

async fn main() {
    my p = Point(22, 44)
    print("The point is ({p.x}, {p.y})").await
}
```

## `my`, `our`, and other permissions

This example already makes use of two permissions, `my` and `our`. There are four permissions in total, and they can be divided along two dimensions:

|        | Unique     | Shared       |
| ------ | ---------- | ------------ |
| Owned  | [`my`]     | [`our`]      |
| Leased | [`leased`] | [`shleased`] |

The difference between these dimensions:

- _Owned_ permissions are permanent. They cannot be revoked through access to other variables.
  - _Leased_ permissions are temporary -- there is always a lessor (either an owner or another lease), and that lessor can reclaim full access to their object.
- _Unique_ permissions are exclusive. When one variable has unique permission, no other variables can access the object while that variable is in use (without some kind of error occurring).
  - _Shared_ permissions can be copied freely, but they [require that the object is read-only](./sharing_xor_mutation.md). In other words, while you are reading from an object with a shared permission, nobody else can come and change it under your feet (except via an [atomic](./atomic.md) field).

## Overview of what's to come

Over the next few chapters, we're going to look at each of the permissions in detail:

- We'll start with the _owned permissions_, [`my`] and [`our`].
- Then we'll cover the _leased permissions_, [`leased`] and [`shleased`].

[`my`]: ./my
[`our`]: ./our
[`leased`]: ./lease
[`shleased`]: ./shlease
