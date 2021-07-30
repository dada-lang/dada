# Dada tutorial

## Hello, world!

Naturally, every tutorial must start with Hello, World, so let's start there:

```
async fn main() {
    println("Hello, world!").await
}
```

The first thing you can see is that Dada uses the async-await notation for I/O, similar to JavaScript. The `println` function doesn't take effect immediately: it returns a chunk of suspended code that, when awaited, will print to the screen. To make the actual print happen, we have to `await` it by writing `.await`. (Note that, like Rust, Dada puts the `await` keyword at the end of the line.)

Let's make the example a bit more interesting and introduce a variable:

```
async fn main() {
    var greeting = "Hello"
    println("{greeting}, world!").await
}
```

Here we see that we can introduce a local variable `name` and then embed it into the string using `{}`. Any string in Dada can embed expressions in this way; if you want to print a literal `{`, then you should use two of them (`"{{"`). Alternatively, you can write `r"foo"` to get a "raw" string with no special characters (this includes things like `\n`, which are encoded to a newline in a regular string, but get interpreted as a `\` and a `n` in a raw string).

## Desugaring format strings

When you use a format string like `"Hello, {name}!"` that is actually syntactic sugar for building up the string by hand using a series of calls to `push_str`. If we wanted, we could get a similar effect by rewriting our code like so:

```
async fn main() {
    var greeting = "Hello"
    greeting.push_str(", world!")
    println(greeting).await
}
```

## Ownership in Dada

Instead of using a local variable, let's make a helper function:

```
async fn main() {
    var greeting = "Hello";
    greet_world(greeting).await
}

async fn greet_world(greeting: String) {
    println("{greeting}, world!").await
}
```

So, what happens if we try to call `greet_world` twice?

```
async fn main() {
    var greeting = "Hello";
    greet_world(greeting).await
    greet_world(greeting).await // ERROR
}

async fn greet_world(greeting: String) {
    println("{greeting}, world!").await
}
```

We get an error:

```
|    greet_world(greeting).await
                 -------- `greet_world` claimed ownership of `greeting` here
|    greet_world(greeting).await
                 ^^^^^^^^ but you tried to use it again here!
}

```

What is going on here? The answer is that Dada is based around a system of **ownership**. When you have a string, that requires memory to store the characters. Dada doesn't rely on a garbage collector, so it instead tracks who is the *owner* of that string. When you write the name of a class like `String`, that is actually shorthand for a type `my String`:

```
async fn greet_world(greeting: my String) {
    //                         ^^ explicit ownership mode
    println("{greeting}, world!").await
}
```

### Ownership modes

Dada actually has four "ownership modes" that can be used with classes, and they can be categorized in two ways:

|               | Unique     | Joint |
| ------------- | ---------- | ---------- |
| **Owned**     | `my`       | `our`        |
| **Not owned** | `borrowed` | `shared`     |

When the mode is **unique**, it means that the variable you are using is the *only variable* that can access its value at this time. So when we have `who: my String`, then `who` is the only variable that can access the string underneath. This is why we got an error when we try call `greet` again.

We can fix this by adopting a 'non-unique' (joint) mode. Let's change to `our` by changing to a "joint" mode. Let's try `our`, so that we still have an "owned" mode (we'll explain the "not owned" modes later):

```
async fn main() {
    var greeting = "Hello";
    greet_world(greeting).await
    greet_world(greeting).await // ERROR
}

async fn greet_world(greeting: our String) {
    //                         ^^^
    println("{greeting}, world!").await
}
```

Hey, it compiles now! Behind the scenes, what's happening is that `main` and `greet_world` have **joint ownership**. This means that there is one string buffer, but two variables that each have access to it (`greeting` in `main` and `greeting` in `greet_world`).

## Uniqueness and mutation

Earlier we converted from a format string like "{greeting}, world!" into a call to `push_str`. What happens if we try that again now that we have an `our String`? We might write something like this:

```
async fn greet_world(greeting: our String) {
    greeting.push_str(", world!") // Error
    println(greeting).await
}
```

Answer: compilation error.

```
Error: Cannot borrow from a jointly owned String
```

This is a fundamental rule of Dada: **mutation generally requires uniqueness**. There is a very good reason for this: when we have joint access, that means that other variables are also referencing the same underlying string. If we were to mutate it, then those variables would see those mutations too, which might make them incorrect.

After all, look at our `main` function:

```
async fn main() {
    var greeting = "Hello";
    greet_world(greeting).await
    greet_world(greeting).await // ERROR
}
```

Just looking at `main`, you would expect this program to issue the same greeting twice in a row. But if `greet_world` were permitting to mutate `greeting`, as we tried to do, what we would actually get would be:

```
Hello, world!
Hello, world!, world!
```

This is because the first call to `greet_world` would modify the string that the second call then sees. Probably not what we wanted.

Later on, we'll see that Dada does offer ways to mutate jointly accessible data: but you have to be deliberate about it.

## Sharing

In all our examples so far, we always took ownership of the values we were working with -- either unique or joint ownership. But for most functions, ownership is unnecessary. Ownership implies that the function intends to use the data indefinitely (or perhaps to destroy it!). But most functions just need temporary access to the data to read or write while they execute. The final two ownership modes in Dada, **sharing** and **borrowing**, correspond to this kind of *temporary access*:

* **Shared** mode indicates that the function intends to read the data for some temporary time. Just like with joint ownership, when you declare data as shared, it's considered immutable.
* **Borrowed** mode indicates that the function has temporary, unique access to the data. We'll look at this closely later, but it can be a way to delegate mutation to helper functions (`push_str`, for example, takes a borrowed string, which is how it modifies a string owned by its caller).

We can rewrite `greet_world` to use a `shared` String like so:

```
async fn greet_world(greeting: shared String) {
    println("{greeting}, world!").await
}
```

This will allow our main function to compile just fine:

```
async fn main() {
    var greeting = "Hello";
    greet_world(greeting).await
    greet_world(greeting).await // ok
}
```

This is ok because the first call to `greet_world` doesn't take ownership of `greeting`, but instead just *shares* it with `main`. Once `greet_world` returns, though, that sharing has ended.

. Just as before, we cannot invoke `greeting.push_str` 

## Creating classes

```
struct 
```

## 

## Frequently asked questions

### Why not have immutable local variables, like in Rust?

In short: One more thing to explain, little value added. But also: consistency with fields.

### Consistency with fields? Yeah, that reminds me, why *do* fields need `var`?

In Dada, if you have a class:

```
class Foo {
    strings: Vec<String>
    var counter: u32
}
```

then if you want to modifying this field, you need not only unique access, but the field has to be declared var:

```
impl Foo {
    fn foo(borrowed self) {
        self.strings.push("some string") // ERROR
    }
}
```

This has an advantage, because it limits the set of fields that a `borrowed self` might potentially mutate:

```
impl Foo {
    fn count_empty_strings(borrowed self, str: shared String) {
        if str.is_empty() {
            self.counter += 1
        }
    }
    
    fn count_data(borrowed self) {
        for var str in shared self.strings { // XXX
            self.count_empty_strings(str)
        } 
    }
}
```

### Isn't that inconsistent with only having `var` on local variables?

The underlying principle is really: mutation is not bad, but one should be easily able to anticipate who is affected by it. For local variables, all accesses are readily visible. The IDE can easily color local variables by whether they are mutated or not. 

For fields, this is more challenging. Accesses can be spread across modules or even across multiple *crates*, if the field is public.

## Credit where credit is due

The Dada language builds on a lot of ideas, but it is particularly similar to Lark, which was a joint project of wycats, nikomatsakis, and jonathandturner.