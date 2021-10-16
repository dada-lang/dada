# Sharing

⚠️ **DADA DOESN'T REALLY EXIST.** ⚠️ See the [main tutorial page](../tutorial.md) for more information. Also, you have to pretend that all the code examples are editable and runnable, with live IDE tooltips and so forth. =)

**Covers:**

- The `shared` ownership mode and its implications.

## Starting point

As our starting point, we'll use the [format string example](./basics.md#format-strings) from the [basics](./basics.md) chapter:

```
async fn main() {
    var name = “world”
    var greeting = “Hello, {name}”
    print(greeting).await
}
```

## Helper function: `greet`

Let's introduce a helper function called `greet`. The role of `greet` is to compose the greeting and print it to the screen:

```
async fn main() {
    var name = “world”
    greet(name).await
}

async fn greet(name: String) {
    var greeting = “Hello, {name}”
    print(greeting).await
}
```

Two important things to note:

* `greet` is an async function. This is because it is calling `print`, which is an async function. Try removing the `async` keyword and see what happens. (Answer: compilation error.)
* Because `greet` is an async function, calling it yields a thunk which we must `await`. Try removing the `await` and running the example and see what happens! (Answer: nothing.)

## Why no ownership mode?

You may have noticed that `name` doesn't have any ownership mode! That's because the only thing we are doing is *reading* from `name`. When you write no ownership mode on a function parameter, that means the function can be called with *any* ownership mode that the caller wants[^generic].

[^generic]: What's actually happening is that `greet` is generic over the permissions of `name`. We'll cover generics in more detail in a later tutorial.

## Sharing mode

