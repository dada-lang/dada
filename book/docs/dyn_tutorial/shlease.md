# Shleases: Shared leases

We've nearly completed our tour of Dada's permissions. It's time to visit the last square in our table:

|            | Unique                 | Shared               |
| ---------- | ---------------------- | -------------------- |
| Owned      | [`my`](./my.md)        | [`our`](./our.md)    |
| **Leased** | [`leased`](./lease.md) | ⭐ **`shleased`** ⭐ |

A shlease[^pronounced] is a _shared lease_ and it combines attributes of a `leased` value and an `our` value:

-   Like a `leased` permission, `shleased` permissions are _temporary_. The lessor can terminate the shlease and reclaim their full permission.
-   Like an `our` permission, `shleased` permissions are _shared_. They can be copied freely, and hence -- because [friends don't let friends mutate shared data](./sharing_xor_mutation.md) -- shleased objects are read-only so long as the shlease lasts.

[^pronounced]: Pronounced "shlease".

## Example

Let's see an example of shleases in action. We are going to create a pair (owned by `m`) and then shlease it out to a bunch of objects. All of them will be able to freely read from the pair:

```
class Pair(a: our, b: our)

# `m` owns the `Pair`
let m: my = Pair(22, 44)

# `s1` shleases the pair from `m`
let s1: shleased = m

# `s2` copies the shlease
let s2: shleased = m

# we can now read from `m`, `s1`, and `s2` interchangeably
print(m.a).await
print(s1.a).await
print(s2.a).await
```

## The `shlease` keyword

You can make an explicit shlease by using the `shlease` keyword:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let s1: any = m.shlease
```

If you position your cursor after `s1`, you will see that it has `shleased` permissions.

## Canceling a shlease

When you have a shlease to an object, you know that the object will not be mutated so long as your shlease remains valid. Once a lessor writes to the object, your shlease is canceled. Let's see that in action:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let s: shleased = m

# When `m` writes to `a`, that cancels the shlease
m.a += 1

# Accessing `s` is an error now
print(s.a).await    # Error!
```

## Giving, sharing, and leasing a shleased value

You can also apply the `give`, `share`, and `lease` keywords to shleased values. In all cases, they simply reproduce the `shlease` value:

```
class Pair(a: our, b: our)
let m: my = Pair(22, 44)
let s1: any = m.shlease
let s2: any = s1.give    # s2 is just a copy of s1
let s3: any = s1.ref   # s3 is also just a copy of s1
let s4: any = s1.lease   # s4 is ALSO just a copy of s1
```
