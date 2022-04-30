# Dada for Rustaceans

If you know Rust, many of the concepts in Dada will be familiar, but there are also some key differences. Let's start by exploring **dynamic Dada** and then we will discuss the type system!

## Dynamic Dada: Ownership

When you create a value in Dada, as in Rust, you get ownership of it:

```
async fn main() {
    var v /* : my Vec */ = [];
    
}
```
