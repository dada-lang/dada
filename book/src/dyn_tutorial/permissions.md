# Ownership permissions

This chapter explains **ownership permissions**, which are what allows Dada (like Rust!) to avoid the need for a garbage collector while retaining memory safety. This chapter also gives a "Dada in a nutshell"-style survey over the key ideas in Dada. Subsequent chapters will dive into those ideas in more detail.

## Running example

As we explain permisions both here and over the next several chapters, we're going to work with variations on this example program. It builds on syntax that we [introduced previously](./class.md), so if anything is confusing you may want to check out that chapter.

```
class Point(x, y)

async fn main() {
    my p = Point(22, 44)
    print("The point is ({p.x}, {p.y})").await
}
```

## What does `my` mean?

In our example, the variable `p` is declared by writing `my p = Point(22, 44)`. The `my` keyword here is an example of an **ownership permision**. It indicates that `p` has *unique ownership* over the object that is stored insode of it:

* *Unique* means that no other variable can access that same object, at least not while `p` is using it.
* *Ownership* means that no other variable can take away the permission to access that object. It also means that, so long as `p` is in scope, the memory for the `Point` will not be freed.

## Other ownership permissions

There are other kinds of ownership permissions, too:

| Permission   | Explanation                                              |
| ------------ | -------------------------------------------------------- |
| `my`         | Unique ownership of the object                           |
| `our`        | [Shares](./share.md) ownership of the object with others |
| `leased`     | Unique [lease](./lease.md) to the object                 |
| `our leased` | [Shared lease](./shared_leases.md)                       |
| `any`        | any of the above                                         |

As the table above suggests, ownership permissions are not fixed. They can change over the course of execution. For example, `p` can [give](./give.md) its object away to a new owner, in which case `p` no longer has any permisions at all. Alternatively, `p` can [lease](./lease.md) access to another variable, which allows that variable to access the object, but only until `p` wants to use it again.

## Other ownership permissions

There are several other ownership permisions besides `my`. We'll be covering them in detail throughout the tutorial but here is a brief summary:

| Permission   | Name                               | Explanation                                                                                                                                                                                                                  |
| ------------ | ---------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `my`         | Unique ownership                   | Nobody else has access to this object unless we grant it to them.                                                                                                                                                            |
| `our`        | [Shared ownership](./share.md)     | Many variables can share access at once, but [none of them can write to the object](./sharing_xor_mutation.md).                                                                                                              |
| `leased`     | [Unique lease](./lease.md)         | the object is [leased](./lease.md) from another variable. This variable has unique access, but it is only temporary. The owner can cancel the lease and start using the object again.                                        |
| `our leased` | [Shared lease](./shared_leases.md) | a [shared lease](./shared_leases.md) means that multiple variables have ([read-only](./sharing_xor_mutation.md)) access, but that access is only temporary. The owner can cancel the lease and start using the object again. |
| `any`        |                                    | the variable can store values with any of the above permisions                                                                                                                                                               |

## Shared ownership

For example, instead of writing `my p`, we could've written `our p` to declare a variable with shared ownership:

```
class Point(x, y)

async fn main() {
    our p = Point(22, 44)
}
```


## Enforcing uniqueness and ownership

When you declare a variable as `my p`, that means that it has *unique ownership* of the object(s) stored inside of it. 

```
async fn main() {
    our p = Point(22, 44)
    my q = p                                         # Error
}
```



## Permissions and mutation

The most obvious effect 
## Default permissions

If you don't specify any ownership permision, the default is `our leased`. This means that the variable cannot be used to mutate the fields of the object. Example:

```
class Point(x, y)

async fn main() {
    p = Point(22, 44)      # declares `p` in `our leased` mode
    p.x += 1               # Error!
}
```

## Ownership permissions are enforced

We saw that `my p` indicates that the variable `p` has unique ownership of its contents. But what happens if we try to store a value in `p` that cannot be owned? As an example, integers like `22` and `44` are inherently shared. If we try to claim ownership of those, we get an error. As an example, try running this:

```
async fn main() {
    my x = 22               # Error!
}
```

Now try changing the program to use `our x = 22`: you'll find that it works. You can also write `x = 22`, which is short for `our leased x = 22`. All of these work fine. This is because integers like `22` are shared: you cannot assert unique access to them. 

**Exercise:** What do you think will happen with `leased x = 22` or `any x = 22`? Give it a try and see!

## Ownership permissions on fields

Ownership permissions also apply to fields and function arguments. The declaration of `Point` we have been using actually declares its fields with the default permissions, `our leased`:

```
class Point(x, y)

# is short for
# class Point(our leased x, our leased y)
```

Just as with local variables, declaring a field to be `my` means that the class can only store values that it has unique ownership of:

```
class Point(my x, my y)

async fn main() {
    my p = Point(22, 44)      # Error!
}
```
