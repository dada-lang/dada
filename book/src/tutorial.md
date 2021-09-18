# Tutorial

*This tutorial aims to explain Dada from scratch. Dada as described here doesn't really exist, so this tutorial is purely hypothetical. It's goal is to get a feeling for how it would be to teach Dada and to explore the ideas without going through the trouble of implementing them.*

**Caveat:** *The act of writing this tutorial caused me to rethink a lot of things about Dada, so I hope to post an updated version.*

## Hello, World

The classic “Hello, World” program in Dada should be quite familiar:

```
async fn main() {
    print(“Hello, world!”).await
}
```

The main twist that may be surprising is that, like JavaScript, Dada is based exclusively on **async-await**. This means that 

## Variables and ownership

To make the program a bit more interesting, let’s try introducing a variable:

```
async fn main() {
    my greeting = “Hello, world”
    print(greeting).await
}
```

Now we start to see some differences. The `my` keyword here is one of the ways that Dada has to introduce a variable; as the name `my` probably suggests, Dada is an *ownership-based language*. This means that when you create values in Dada — like a String — those values always have 1 or more *owners*.

For the time being, when we talk about an owner, we are talking about some function that is currently running. So when you see `my greeting = …`, we are saying that the function `main` owns the string called `greeting`. Later on, we’ll extend the concept of owners to also include other data structures (example: a list or map).

## Mutation and ownership

When you are the owner of a value, you are also permitted to mutate it. An alternate way to write the `main` function might be something like:

```
async fn main() {
    my name = “world”
    my greeting = “Hello, “
    greeting.push_str(name)
    print(greeting).await
}
```

Here, the calls to `greeting.push_str(…)` modify the `greeting` string in place, adding new contents.

## Aside: Format strings

As an aside, there is an easier way to write the above program. Dada strings support inline formatting, so we could write the following:

```
async fn main() {
    my name = “world”
    my greeting = “Hello, {name}”
    print(greeting).await
}
```

This would be equivalent to the program which calls `push_str` manually.

## My is *sole* ownership

The `my` keyword specifically indicates *exclusive ownership*. That is, a variable declaration like `my name` means that the function `main` is the **only** owner of the string `”Hello, world”`.  To understand what this means, we need to first introduce a second function. Consider this example, which introduces a helper function `greet`:

```
async fn main() {
    greet(“world”).await
}

async fn greet(my name: String) {
    print(“Hello, {name}”).await
}
```

This code as is compiles fine, but perhaps we can try a slight modification. We will create a variable `my name` and then call `greet` with that variable:

```
async fn main() {
    my name = “world”
    greet(name).await
}

async fn greet(my name: String) {
    print(“Hello, {name}”).await
}
```

If we try this, we are going to find that the code does not compile:

```
error: calling function without giving ownership
|    greet(name)
|         ^^^^ `greet` requires ownership of `name`
help: try using the `give` keyword
|    greet(give name)
|         ^^^^ use the `give` keyword to transfer ownership to `greet`
```

What is going on here? The problem is that `greet` has declared that it needs sole ownership of its parameter, `name`. This means that `main` needs to **give up** its ownership of `name`, since you can’t have two “sole owners”.

To give up ownership of something in Dada, you use the `give` keyword. Something like `give name` gives up ownership of the value in `name` and allows it to be assigned somewhere else. We can therefore write our code like so (as the compiler, in fact, suggested):

```
async fn main() {
    my name = “world”
    greet(give name).await
}

async fn greet(my name: String) {
    print(“Hello, {name}”).await
}
```

Now the code compiles and runs! Yay!

## Implications of giving up ownership

When we give up ownership of a value, it means that we can’t use the value later. For example, if we were to try and call `greet(give name)` twice in a row, we are going to get an error:

```
async fn main() {
    my name = “world”
    greet(give name).await
    greet(give name).await
}
```

Running this program results in:

```
error: value in `name` was already given away
|    greet(give name).await
|          ————- `name` was given away here
|    greet(give name).await
|              ^^^^ `name` is used again here
```

The ability to *give a value away* is in fact the key enabler that distinguishes languages like Rust and Dada from other languages like C or Java. It allows you to model disposing or transferring resources from place to place. This comes up all the time in programming: for example, closing files (after which you ought not to try and read from the file), sending values to other threads (which you then should not read or modify anymore), or even domain specific things like transferring money between bank accounts.

## Joint ownership using `our`

As powerful as giving values away is, it is not always what you want. Sometimes, you would like to have a single value that has multiple owners. A common example of this might be memoization: you wish to compute a value once and then store a cached value of it. When you need that value later, you want to return a shared copy of that cached value (but you also want the value to stay in the cache). Dada supports this kind of pattern, and we call it **joint ownership**.

Joint ownership is indicated by using the keyword `our`. For example, we could rewrite our running example to use `our` like so:

```
async fn main() {
    our name = “world”
    greet(name).await
}

async fn greet(our name: String) {
    print(“Hello, {name}”).await
}
```

This program compiles just fine. Note that we did not have to write `give name` in order to call `greet`:

```
    greet(name).await
```

Because `greet` is now declared with `our name`, it is no longer demanding *sole* ownership of `name`. This means that `main` does not have to `give` ownership away, but can instead just pass `name` as an ordinary parameter. Since `main` is not giving ownership away, It is also possible to call `greet` multiple times:

```
async fn main() {
    our name = “world”
    greet(name).await
    greet(name).await
}
```

## Joint ownership generally makes values immutable

Joint ownership is pretty useful, but it comes with a catch. Values that are jointly owned are typically **immutable** (we’ll see later that there is actually a way to enable mutation, but it requires some extra care). For example, earlier we saw the function `greet`, defined like so:

```
async fn greet(our name: String) {
    print(“Hello, {name}”).await
}
```

Instead of using the `”Hello, {name}”` format string, we might try to use mutation, similar to what we saw before when we invoked `push_str`, but this time using `insert_str` to insert a string in the beginning:


```
async fn greet(our name: String) {
    name.insert_str(“Hello, “)
    print(name).await
}
```

This code, however, will not compile:

```
error: cannot modify a jointly owned value
|    name.insert_str(“Hello, “)
|    ^^^^^^^^ `name` is jointly owned and cannot be modified
```

This idea, that values which are accessible by more than one function at the same time are immutable, is a core design constraint of Dada (inherited from Rust). The reason for this constraint is that mutating jointly owned values is a recipe for buggy programs.

To see why, consider the version of `main` we saw earlier that calls `greet` twice in a row:

```
async fn main() {
    our name = “world”
    greet(name).await
    greet(name).await
}
```

Now, pretend you are reading this code, and ask yourself: what does this program do? You may not be exactly sure what `greet` does, but you could quite reasonably expect that invoking `greet(name).await` twice in a row would print the same thing twice. However, if we permitted `greet` to modify `name`, then in fact this program would print:

```
Hello, world
Hello, Hello, world
```

which is not what we expected.

This principle extends more generally. It turns that a great many of the nastiest bugs — data races, illegal memory accesses due to iterator invalidation, use after free — correspond to cases where one bit of code modifies some value that another bit of code doesn’t expect to change. 

Therefore, in Dada, we generally try to limit mutation to cases where you have **unique access** to the value, so that there is no possibilty of the value changing underfoot. 

## Leasing as an alternative to ownership

So far, we have introduced the concept of ownership, and we have explained **sole and joint ownership**. What sole and joint ownership have in common is that they do not expire: when you call a function and give it sole or joint ownership of some value, you can’t take that back. 

As an example, when `greet` declares `our name: String`, that indicates to its callers that they must give `greet` joint ownership over that string from the point of the call forward. Even once `greet` has returned, the callers have to assume that there are other owners of `name` floating around. This is because `greet` might have, for example, started a thread that uses `name` and which is still running. Or perhaps `greet` stored `name` in a global data structure somewhere that is still accessible. We don’t know.

In practice, though, most function parameters are only used while the function executes. The function just needs some data to read and write and then, once it’s returned, it is finished. Dada supports this pattern of giving temporary access to a value — it’s called **leasing**. When one function **leases** a value to another, it is saying “here, you can use this value, but when you’re done, I want it back; I might lease it to someone else”.

There are two kinds of leases in Dada: borrowed (unique access) and shared (joint access). When you have a borrowed value `v`, it means that you have **unique** access to that value through `v`, but you are not the owner. Once you finish using `v`, the owner can go using the same value. Since you have unique access, though, you are permitted to modify `v`. Those modifications aren’t visible to anyone else until you have finished your borrow. 

In contrast, a **shared lease** means that you have joint access. There could be many active functions (for example, in different threads, or via different data structures) that all share access to the same shared value. As a result, shared values are generally immutable, just like jointly owned values.

Effectively, ownership in Dada results in a table like this:

| | Unique (permits mutation) | Joint (generally immutable) |
| —- | —- | —- |
| Owned (permanent access) | my | our |
| Leased (temporary access) | borrowed | shared |

On the horizontal axis, you have unique vs joint. This indicates whether there are multiple ways to reference the same value at the same time.

On the vertical axis, you have ownership vs leasing. When you own a value, it means that you have permanent access: you can choose to give that value to other functions, or to another thread, and so forth. When you have leased a value, you only have temporary access. Once the owner of the value starts using the value again, you have to stop. Therefore you could not start a new thread that is using the value, because once you returned, that thread might still be executing, but the owner of the value would like to go back to using it now.

## Using a shared lease

Let’s rewrite our running “Hello, world” example using leases. To start, we’ll use a shared lease:

```
async fn main() {
    my name = “world”
    greet(name).await
    greet(name).await // we’ll call it twice, just for fun
}

async fn greet(shared name: String) {
    print(“Hello, {name}”).await
}
```

There are a few interesting details to note in this program. First, `main` declared `my name`, indicating that it *owns* the string `name`. The function `greet` meanwhile was declared with `shared name`. This indicates that `name` is *leased* from its caller. 

So the situation we have here is that `main` owns a string. It leases this value out to `greet`: because this is only a lease, once `greet` returns, `main` still has ownership, and it can go on and keep using `greet`. In this case, it can call `greet` again.

## Shared leases are generally immutable

Just as we saw with joint ownership, `greet` will not be able to modify its `name` parameter. This is because `greet` only has a shared lease, and shared leases do not permit modification:

```
async fn greet(shared name: String) {
    name.insert_str(“Hello, “)
    print(name).await
}
```

gives the compilation error:

```
error: cannot modify a shared value
|    name.insert_str(“Hello, “)
|    ^^^^^^^^ `name` is shared and cannot be modified
```

## Using a borrowed lease

Let’s do one final rewrite of our program. This time, instead of using a shared lease, we are going to use a borrowed lease. Remember that *borrowing* a value means that you get *unique access* to the value, but only temporarily. This is just the same as borrowing a book or a hammer from a friend: while you are using it, your friend no longer has access to it, but once you’re done, they can have it back.

Here is our same example, but this time using a borrowed lease:

```
async fn main() {
    my name = “world”
    greet(lend name).await
    greet(lend name).await // we’ll call it twice, just for fun
}

async fn greet(borrowed name: String) {
    print(“Hello, {name}”).await
}
```

There are a few things to note here:

* As before, `main` remains the owner of the value, and it uses `my name` to indicate that.
* `greet` is declared with `borrowed name: String`, indicating that it needs a temporary, unique access to `name`.
* When `main` calls `greet`, it signals the borrow with `lend name`.

The final point is important: when `main` wants to `lend` a value to another function, that needs to be indicated explicitly, in the same way that the `give` keyword was used to signal that `main` was giving ownership away. In general, keywords like `lend` and `give` are required *whenever you are giving away unique access, either permanently (`give`) or temporarily (`lend`)*. The keyword is a signal that the calling function may not be able to keep using the variable in the same way that it did before, since it is now lent or given away.

## Borrowed leases permit modification

Because a borrowed lease offers unique access, it also permits modification. Therefore, we could rewrite the program above so that `greet` modified its parameter in place:

```
async fn main() {
    my name = “world”
    greet(lend name).await
    greet(lend name).await
}

async fn greet(borrowed name: String) {
    name.insert_str(“Hello, “)
    print(name).await
}
```

As before, this function would print:

```
Hello, world
Hello, Hello, world
```

You might wonder why this is ok. The key observation here is that `greet(lend name)` contains a signal that `name` may be modified: the `lend` keyword. So even if we don’t know what `greet` does, we know that it may modify the string that we give it.

## Subleasing: returning shared values that are derived from other values

Sometimes you would like to have a function that returns values from 

```
```


