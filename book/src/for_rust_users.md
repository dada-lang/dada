# Dada for Rust programmers

This is a brief introduction to the ideas of Dada, aimed at people familiar with Rust. Dada is generally quite close to Rust but it makes a few strategic departures from the way that Rust does things. This section aims to explore those departures and what motivates them.

You're going to see a combination of random syntactic details but also some core, underlying concepts. At the end, I'll try to pare out the core concepts and talk about what might or might not be applicable to Rust.

## Ownership in Dada: my and give

Just like Rust programs, Dada programs do not rely on a garbage collector to manage memory. Instead, they rely on an ownership system that tracks who owns which piece of memory and thus knows when the memory is no longer in use.

In Dada, the type for an owned piece of data is written with the `my` keyword. So `my String` indicates an *owned* string:

```
var data: my String = "Hello, World!"
```

As a kind of "syntactic sugar", Dada permits you to put this ownership modifier on the `var` keyword as well:

```
my data = "Hello, World!"
```

Just as in Rust, you can transfer ownership of a value from place to place. Unlike Rust, Dada uses a `give` keyword to signifiy when ownership of a value is being transferred:

```
my data: String = "Hello, World!"
my data2: String = give data
// The variable `data` is no longer accessible,
// as its value has been moved.
```

If you leave off the `give` keyword, it is assumed that the value is being copied. For uniquely owned things, copies are not allowed, so something like this is an error (we'll see later how to have multiple owners via ref-counting):

```
my data: String = "Hello, World!"
my data2: String = data // Error: cannot copy a `my String`
```

Just as in Rust, uniquely owned data is mutable. Unlike in Rust, all local variables are mutable, there is no need to declare `let mut`:

```
my data = "Hello, "
data.push_str("World")
```

## Joint ownership in Dada

In Rust, if you want to have jointly owned data, you use a reference-counter type like `Rc` or `Arc`. In Dada, referencing counting is built-in. You can use the  `our` ownership mode:

```
our data: String = "Hello, world!"
```

Just as in Rust, shared data is immutable by default. Attempting to mutate an `our String` will give you a compilation error:

```
our data = "Hello, "
data.push_str("World") // ERROR!
```

When you have a jointly owned variable, you can make copies:

```
our data = "Hello, World"
our data2 = data // OK
```

## Sharing tracks where data came from

In Rust, you use `&T` to create temporary references to things. In Dada, we call that `shared`, and you create one with `share`:

```
my data = "Hello, World!"
shared tmp = share data;
```

Unlike Rust, however, with Dada shared types don't have lifetimes. Instead, the type of a `shared` value indicates where the variables that it is borrowed from. That means that the full type of `tmp` is `shared(data) String`, indicating that it is a shared string that is owned by the variable `data`.

## Shares are not pointers

Besides tracking variables, there is another key difference between a Dada type like `shared String` and a `&String` type in Rust. A `shared String` is not a pointer. It has the same memory layout as a `String`, but it is a copy that is known not to own its data. So when it goes out of scope, it doesn't free the underlying data.

## Sharing freezes the original

Just like in Rust, when you share a variable in Dada, that variable's contents are temporarily immutable so long as you are using the shared copy:

```
my data = "Hello, "
shared p = share data
data.push_str("World") // ERROR
print(p)
```

## Sharing tracks the original value

In Rust, when a variable has been borrowed, you can't move from it. In Dada, though, you can move a value while it is shared, and the sharing remains valid. The effect is just to modify the type of the shared value so that it tracks the value into its new home:

```
my data1 = "Hello"
shared(data1) s = share data1
my data2 = give data1
// type of `s` changes to `shared(data2)`
```

In Rust, this wouldn't always work, because shared references in Rust are *pointers* to the original value. When the value moves, that pointer is still pointing at the old location. But in Dada a shared value is a copy of the data, and so it is not connected to the location of the value.

## Sharing



## Borrowing

```

```

## Ownership in Dada


[^class]: Classes in Dada are a lot like structs in Rust.


Dada's ownership system however is based around the idea of a *lease*. 

 Whereas Rust's ownership system is based around *lifetimes*, though, Dada's system is based around *leases*. Simply put, a *lease* tracks *where the data 

## Sharing copies, not pointers

The first 

## Variable fields

## Leases, not lifetimes

## Self-referential classes

## One box to rule them all


## Splitting classes and data


A brief introduction for Rust programmers. Dada has a very similar concept of ownership and borrowing to Rust, but it manifests quite differently:




## Classes

```
class MyClass<type T>(
    f1: String
    f2: [T]
)
```

## Modes vs references

Dada has a similar concept to Rust

## Data vs enums

Rust offers structs and enums, but Dada just has `data` types. These types are like a hybrid of tuple structs and named structs:

```rust
data MyDataType<type T>(
    f1: String
    f2: [T]
)
```

You can extend one data type with another

```rust
data Option<type T>
data None<type T>: Option<T>
data Some<type T>(value: T): Option<T>
```

Finally, there is a shorthand syntax for that which is similar to Rust's enum syntax:

```
data Option<type T> {
    None
    Some(value: T)
}
```

Note that in Dada, unlike Rust, `None` and `Some` are their own types, and they are coercible to the base type `Option`.

Dada also supports enums that share fields:

```
data AstNode(id: int)
data StringNode: AstNode // inherits the `id: int` field

```

## What Dada does that Rust doesn't

* Special syntax for vectors `[T]` and maps `{K: T}`.
    * No concept of arrays.
* Have exactly one kind of smart pointer, the box
    * A box always includes a ref count, it's just often 1
    * Classes and data types can be defined as boxed
* Instead of unsized types, we heap allocation for interface types
* Supports OO-style subclasses and composition, because it is nice
* Use modes instead of references:
    * A `shared String` is identical in capability to an `&String` in Rust, but it is represented not as a pointer but as a struct (i.e., 3 words)
    * As such, modes are always flattened. There is no such thing as a `shared shared String`, that makes no sense.
    * Modes only apply to classes, not data types, so you never have
* Only supports async I/O
    * Furthermore, FFI calls that are not "pure" (defined below) must be async as well
* Use software transactional memory instead of cell, ref-cell, and the atomic classes
    * This is the most wild-and-crazy design choice of Dada by far