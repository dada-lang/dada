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

## `my` vs `our`

This example already makes use of two permissions, `my` and `our`. There are four permisions in total:

| Permission | Explanation                                                    |
| ---------- | -------------------------------------------------------------- |
| `my`       | Unique ownership of the object                                 |
| `our`      | [Shares](./share.md) ownership of the object with others       |
| `leased`   | Unique [lease](./lease.md) to the object                       |
| `shleased` | [Shares a lease](./shared_leases.md) to the object with others |

You can also write `any` on a variable to indicate that the variable has any permission.

## Ownership vs sharing

You can also look at the four permissions as being divided along two dimensions:

|        | Unique   | Shared     |
| ------ | -------- | ---------- |
| Owned  | `my`     | `our`      |
| Leased | `leased` | `shleased` |

## Overview of what's to come

Over the next few chapters, we're going to look at each of the permissions in detail:

* We'll start with the *owned permisions*, [`my`] and [`our`].
* Then we'll cover the *leased permissions*, [`leased`] and [`shleased`].

[`my`]: ./my.md
[`our`]: ./our.md
[`leased`]: ./leased.md
[`shleased`]: ./shleased.md