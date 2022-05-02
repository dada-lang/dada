---
sidebar_position: 4
---

# Permissions

In the previous chapter, we saw keywords like `my` and `our` attached to variables. These are examples of **permissions** -- permissions are a key part of Dada. Like Rust, Dada leverages permissions to avoid the need for a garbage collector, while retaining memory safety.

## Running example

As we explain permisions both here and over the next several chapters, we're going to work with variations on this example program. It builds on syntax that we [introduced previously](./class.md), so if anything is confusing you may want to check out that chapter.

```
class Point(our x, our y)

async fn main() {
    my p = Point(22, 44)
    print("The point is ({p.x}, {p.y})").await
}
```

## `my`, `our`, and other permissions

This example already makes use of two permissions, `my` and `our`. There are four permisions in total, and they can be divided along two dimensions:

|        | Unique     | Shared       |
| ------ | ---------- | ------------ |
| Owned  | [`my`]     | [`our`]      |
| Leased | [`leased`] | [`shleased`] |

The difference between these dimensions:

* *Owned* permissions are permanent. They cannot be revoked through access to other variables.
    * *Leased* permissions are temporary -- there is always a lessor (either an owner or another lease), and that lessor can reclaim full access to their object.
* *Unique* permissions are exclusive. When one variable has unique permission, no other variables can access the object while that variable is in use (without some kind of error occurring).
    * *Shared* permissions can be copied freely, but they [require that the object is read-only](./sharing_xor_mutation.md). In other words, while you are reading from an object with a shared permission, nobody else can come and change it under your feet (except via an [atomic](./atomic.md) field).

## Overview of what's to come

Over the next few chapters, we're going to look at each of the permissions in detail:

* We'll start with the *owned permisions*, [`my`] and [`our`].
* Then we'll cover the *leased permissions*, [`leased`] and [`shleased`].

[`my`]: ./my
[`our`]: ./our
[`leased`]: ./lease
[`shleased`]: ./shlease