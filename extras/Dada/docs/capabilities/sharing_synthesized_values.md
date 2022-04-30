# Sharing synthesized values

### Motivating example: returning synthesized values

For this post, I'm going to work through the very basics of ownership in Dada. By the end, I'll show you how Dada solves the "returning synthesized values" problem in Rust.

The "returning synthesized values" problem occurs when you are 'locked in' to a function signature that returns an `&T`, but you wish to return synthesized data. For example, imagine I have a trait `GetName` that returns an `&str`:

```rust
trait GetName {
    fn get_name(&self) -> &str;
}
```

Now imagine that I wish to implement `GetName` for a struct like `Code`:

```rust
struct Code { code: usize }

impl GetName for Code {
    fn get_name(&self) -> &str {
        format!("Code({})", self.code) // ERROR
    }
}
```

This code will not compile, and it can be quite frustrating. The type signature of the trait effectively locks me in to returning a reference to some `String` that is owned by `self`, but that's not what I want here.

By the end of this post, we'll see how Dada addresses this problem.

### Show me the code

Everything I describe here is prototyped in the form of a [PLT Redex](https://redex.racket-lang.org/index.html) model. You can find it in the [dada-model] repository. The model doesn't have a real syntax: Dada programs are represented using S-expressions. For each example, I will link to a test case in the repo.

For the purposes of this post, I'm going to use a "Rust-like surface syntax" for Dada. I have had a lot of fun imagining some ways the syntax could be different, too, and I'll probably blog about those in the future, but I think it would distract from this post.

[dada-model]: https://github.com/dada-lang/dada-model/

### I me me mine: the basics of ownership in Dada

In Rust, there are three basic types:

* `String` -- an owned string (mutable, movable)
* `&String` -- a shared reference to a string (immutable, copyable)
* `&mut String` -- a mutable reference to a string (mutable, movable)

In Dada, those same three concepts are expressed via *permissions* that are applied to a type. The names are slightly different, but the concepts are the same:

* `my String` -- an owned string (mutable, movable)
* `shared String` -- a shared string (immutable, copyable)
* `lent String` -- a lent (borrowed) string (mutable, movable)

Local variables in Dada are declared using the `var` keyword:

```
var x: my String = "Hello, World!"
```

As in Rust, owned values can be moved from place to place. As in the MIR representation used internally in the compiler, Dada's formal model makes moves versus copies explicit. A value can be *moved* from one place to another by using the `give` keyword:

```
var x: my String = "Hello, World!"
var y: my String = give x
// Using `x` again here would be an error
```

Copyable values can be copied using the copy keyword. Attempting to copy an owned class like a `my String` would be an error, however:

```
var x: my String = "Hello, World!"
var y: my String = copy x // ERROR: `x` must be moved
```

### Sharing and leases

In Rust, one uses the `&T` operator to create a shared reference to a value. In Dada, one uses the `share` keyword, which creates a **shared copy** of the value:

```
var x: my String = "Hello, World!"
var p: shared(x) String = share x
```

You'll notice an interesting thing: the type of `p` here is `shared(x) String`. That `x` is called a **lease**, and its Dada's equivalent of a lifetime. It indicates that `p` is *shared from* `x` -- in other words, `p` is a shared copy of a value that is owned by `x`.[^polonius]

[^polonius]: This is a similar model to the one we use in Polonius ([explained here](https://www.youtube.com/watch?v=_agDeiWek8w)); in fact, work on Dada suggested some simplifications that were possible in Polonius, which I'll hopefully describe in a future blog post.

It's possible to have `shared` values with multiple leases:

```
var greeting: my String = "Hello, World!"
var farewell: my String = "Goodbye, World!"
p = if say_hello { 
    share greeting
} else {
    share farewell
}
```

Here, the type of `p` is `shared(greeting | farewell)`, indicating that `p` is a shared `String` owned by either `greeting` *or* `farewell`.

There is a subtyping relationship between shared values: you can always add more leases. So `shared(greeting) String <: shared(greeting | farewell) String`.

### Sharing in function signatures

In Rust, if you have a function that returns an `&T`, you indicate which of the inputs this value was borrowed from by using an explicit lifetime parameter, like so:

```rust
fn get_random<'a>(
    &self, 
    v: &'a Vec<String>,
) -> &'a String
```

In Dada, there are lease parameters, but you would typically use the name of parameters instead:

```
fn get_random(
    shared self, 
    v: shared Vec<String>,
) -> shared(v) String
//         ^^^
```

Technically, this is shorthand for a function with two explicit "lease parameters". Lease parameters are a kind of generic parameter that refer to some set of owner variables in the caller(s):

```
fn get_random<lease l1, lease l2>(
    shared(l1) self, 
    v: shared(l2) Vec<String>,
) -> shared(v) String
```

The compiler knows that borrowing something from `v`, when `v` has type `shared(l2)`, means that you will be borrowing something owned by `l2`.

If you wanted to write a function that returned one of two things, you could do that as well:

```
fn pick<type T>(
    a: shared T,
    b: shared T,
) -> shared(a|b) T
```

### Shared copies, not shared references

The representation of shared values is a major point of departure between Dada and Rust: in Rust, an `&T` is a *pointer* to the original value. In Dada, a `shared T` is a *copy* of the original value. To make it clear:

| Type | Language | Representation |
| ---- | --- | -------------- |
| String | Rust | (owned data_pointer, length, capacity) |
| &String | Rust | pointer to `String` |
| my String | Dada | (owned data_pointer, length, capacity) |
| shared String | Dada | (shared data_pointer, length, capacity) |

In effect, the only difference between a `my` and a `shared` value in Dada is whether or not we will run the destructor when it is dropped. For a `my` value, we will, but for a `shared` value, we will not.

Making shared values have the same representation as owned values seems surprising at first, but it has a number of important advantages. For example, in Rust, the types `&Vec<String>` and `&Vec<&String>` are exactly equivalent in terms of what you can *do* with them. In both cases, you can only get `&String` out from that vector. It might be nice if you could interconvert them. But you can't, because they are *very* different in terms of their runtime representation.

In Dada, `shared Vec<String>` and `shared Vec<shared String>` have the same representation and are literally the same type. (It's not always possible to propagate `shared` modifiers inwards, but I won't go into the details in this post.)

### Subtyping between `my` and `shared`

Because `my` and `shared` have the same representation, we can actually create a *subtyping relationship* between `my` and `shared`. That is, any place a `shared` value is expected, a `my` value can be used instead. For example, imagine we have a function `read_file` that reads the contents of a file from the disk:

```
fn read_file(file_name: shared String) -> my String {
    ...
}
```

Now I can call `read_file` like so:

```
var text = read_file("README.md")
```

Note that `"README.md"`, in Dada, has type `my String`, not `shared String`. So this compiles because `my String` is a subtyping of `shared String`. This achieves a more limited version of the `AsRef` pattern we use in Rust.

### Returning synthesized values

We are finally ready to return to the "returning synthesized values" problem that I used to kick off the post. If you recall, the problem is that sometimes you get 'locked in' to a function signature that returns an `&T`, but you wish to return synthesized data, as is the case with the trait `GetName` and the struct `Code` in this Rust example:

```rust
trait GetName {
    fn get_name(&self) -> &str;
}

struct Code { code: usize }

impl GetName for Code {
    fn get_name(&self) -> &str {
        format!("Code({})", self.code) // ERROR
    }
}
```

This code will not compile because `Code::get_name()` is trying to return a `String` but a `&str` is expected. This can be very frustrating! The type signature of the trait effectively locks me in to returning a reference to some `String` that is owned by `self`, but that's not what I want here.

In Dada, thanks to the subtyping relationship I described, this problem doesn't arise. Here is the equivalent, using a Rust-like syntax for traits and structs:

```
trait GetName {
    fn get_name(shared self) -> shared String;
}

struct Code { code: usize }

impl GetName for Code {
    fn get_name(shared self) -> shared String {
        "Code({self.code})" // equivalent to `format!`
    }
}
```

This code compiles just fine because `"Code({self.code})"` has type `my String`, and `my String` is a subtype of `shared String`.

### Downside of subtyping

There are some downsides to Dada's approach: when the compiler sees a variable of type `shared String` in Dada, it doesn't know *a priori* whether the value needs to be freed or not. It could be a `my String` at runtime that does need to be freed. Consider some Dada code like the following:

```
let c = Code { code: 22 }
let s: shared(c) String = c.get_name()
```

Here, the type of `s` (as annotated) is `shared(c) String`. In other words, this is a shared value that may be owned by `c`. But in fact, we know that `s` is really a `my String` that was created and returned by `get_name()`. 

In practice this means that the compiler would have to generate code to check a flag when a `shared` value is dropped, unless it can see via optimizations that the value is *definitely* not owned.

### Conclusion

To recap, there were two primary ideas that I introduced in this post:

* Sharing that is specified in terms of variables, not lifetimes.
* Subtyping between owned and shared values, which:
    * means that semantically equivalent types like `shared Vec<String>` and `shared Vec<shared String>` also have the same representation;
    * permits one to return synthesized data when shared data is expected;
    * but comes at some cost in space and checking whether destructors are needed.
