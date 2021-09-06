# Dada for Rust programmers



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