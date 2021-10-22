# Hello, Dada!

{{#include ../caveat.md}}

## Gradual ownership

Dada is a **gradual, ownership-based** language. Let's unpack those two things:

* **Ownership-based:** Dada leverages the concept of *ownership* to ensure that (a) your memory is freed at the right times, without any garbage collection and (b) your parallel programs are data-race free.
    * If you've used Rust, Dada's ownership system will be familiar, but keep in mind that there are some key differences between them. (If you've not used Rust, don't worry, we don't assume any prior knowledge in this tutorial.)
* **Gradual:** Dada lets you smoothly transition from an interpreted, dynamic language (similar to Python or JavaScript) over to a statically typed, fully optimized one (similar to Rust). You can even mix code written in the two styles.

In this tutorial, we're going to start out with the more dynamic flavor of Dada and use that to introduce the concepts of ownership and the like. Once you've gotten familiar to that, we'll start to introduce Dada's type system and show how you can use it to check that your Dada code is free of errors.

## Hello, World

The classic “Hello, World” program in Dada should be quite familiar:

```
async fn main() {
    print(“Hello, Dada”).await
}
```

The main twist that may be surprising is that, like JavaScript, Dada is based exclusively on **async-await**. This means that operations that perform I/O, like `print`, don't execute immediately. Instead, they return a *thunk*, which is basically "code waiting to run" (but not running yet). The thunk doesn't execute until you *await* it by using the `.await` operation. 
