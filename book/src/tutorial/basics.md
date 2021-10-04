# Dada basics

⚠️ **DADA DOESN'T REALLY EXIST.** ⚠️ See the [main tutorial page](../tutorial.md) for more information.

## Hello, World

The classic “Hello, World” program in Dada should be quite familiar:

```
async fn main() {
    print(“Hello, world!”).await
}
```

The main twist that may be surprising is that, like JavaScript, Dada is based exclusively on **async-await**. This means that operations that perform I/O, like `print`, don't execute immediately. Instead, they return a *thunk*, which is basically "code waiting to run" (but not running yet). The thunk doesn't execute until you *await* it by using the `.await` operation. 

## Introducing variables

Let's try creating some local variables. Local variables in Dada are introduced with the `var` keyword. For example, we could create a local variable that stores the `print` thunk like so:

```
async fn main() {
    var thunk = print(“Hello, world!”)
    thunk.await
}
```

Alternatively, we could put the "Hello, world!" string into a variable like so:

```
async fn main() {
    var greeting = “Hello, world!”
    print(greeting).await
}
```

## Type inference and type annotations on variables

When you create local variables like `greeting`, you may have noticed that the IDE displays some grey text next to each variable:

```
async fn main() {
    var greeting: my String = “Hello, world!”
    //          ^^^^^^^^^^^ type hint displayed by the IDE
    print(greeting).await
}
```

This text indicates the type of the variable `greeting`; as you can see, the compiler typically infers the types of local variables, but you can also write the annotations yourself if you prefer.

> Sometimes it's nice to have the compiler's annotations become part of the text itself. This makes them visible in contexts where the IDEs inference is not available, such as `git diff`, but it also ensures that the type doesn't change as the program is edited. If you put your cursor on the line where the annotation occurs, you'll see a suggestion to automatically apply the edit into your program. You can also run `dada fix` from the command line to apply edits en masse.

## Ownership permission

Let's discuss the type `my String` itself! The `String` part is probably fairly familiar to you, but you might be wondering what the word `my` is about. The keyword `my` is called an *ownership permission*; ownership permissions are the way that Dada tracks who has access to a given value, and what kind of access they have. These permissions are the "secret sauce" that allows Dada programs to avoid all kinds of bugs (and the use of a garbage collector).

The `my` permission indicates that `main` is the only function that has access to this particular string. That makes sense, since the string was just created. Dada ultimately has four kinds of permissions (`my`, `our`, `lent`, and `shared`); we'll cover them as we go through the tutorial.

## Format strings

In addition to plain string constants, Dada strings can embed program fragments that will get executed and placed inline. These are called "format strings". For example, we could introduce a variable `name` (set to `"world"`) and then compose `greeting` like so:

```
async fn main() {
    var name = “world”
    var greeting = “Hello, {name}”
    print(greeting).await
}
```

## Building up a string imperatively

Format strings are ultimately compiled into some code that builds up the string bit by bit. Let's do that ourselves. Instead of writing "Hello, {name}", let's construct the string by starting with "Hello, " and then appending the word "world":

```
async fn main() {
    var name = “world”
    var greeting = “Hello, “
    greeting.push_str(name)
    print(greeting).await
}
```

You'll notice that when you type this in the IDE, the IDE inserts some greyed out text into the call to `push_str`:

```
    greeting.lend.push_str(name)
    //      ^^^^^
```

What is going on here? What is this `lend` keyword?

## Lending

As you probably guessed, *lending* is related to the ownership permissions. When you have unique access to a value, you can lend it to others, which gives them temporary access. In this case, `main` owns the string `greeting` and so when it calls `push_str`, it is *lending* access to `greeting`. 

> Typically, the compiler infers when lending is necessary, as you see here. It adds annotations to help you understand what your code is doing. However, if you prefer, you can also write `greeting.lend` yourself (or accept the IDE's edit). You can also use the `dada fix` command that we mentioned earlier to automatically insert these annotations.

Ownership and lending works just the way it does in real life. Imagine that you are the owner of a book. It is on your shelf. If you think your friend would like it, you can lend it to them. Importantly, though, **while your friend is borrowing the book, you no longer have access to it**. It's still your book, you still own it, but it's not on your shelf anymore. Lending values in Dada works the same way! While `greeting` is lent out, it is no longer accessible. 

You can't really see the way that `lent` values become inaccessible in our example because the lent copy of `greeting` is immediately given to `push_str`. If we make a simpler example though where everything occurs in one function, you should be able to see it. See the error in the code below? Try editing it and reordering things. You'll see that *so long as the lent copy is live* (i.e., going to be used later), you can't access `greeting`:

```
var greeting = "One"
var greeting_lent = greeting.lend
print(greeting).await // <-- error
greeting_lent.push_str("Two")
```

## Creating a helper function

To better understanding lending, let's create a helper function of our own. We're going to make a function `append_name` that appends a name to a greeting. The syntax for a function in Dada looks like this:

```
fn append_name(greeting: String) {
    var name = “world”
    greeting.push_str(name)
}
```

The first thing to note is that `append_name` is not an `async fn`, just a `fn`. This is because it doesn't do any I/O. The second thing to note is that we are getting an error! Highlight the "squiggly line" on the call to `push_str` to see the details:

```
error: cannot lend `greeting`
| fn append_name(greeting: String) {
                 ---------------- greeting is not declared with any permissions
|     greeting.push_str(name)
               ^^^^^^^^
               push_str requires a lent string
help: try declaring `greeting` as a `Lent` string
| fn append_name(greeting: lent String) {
                           +++++
```

The problem is that function parameters only have the permissions that we give them. We didn't declare `greeting` with any particular permissions, and so we are not able to `lend` it out (the only thing we can do is read from it, in fact). We can fix this by applying the compiler's suggestion (try `dada fix` from the command line if you prefer):

```
fn append_name(greeting: lent String) {
    var name = “world”
    greeting.push_str(name)
}
```

Ta da! It works. Now we can modify `main` to use it:

```
async fn main() {
    var greeting = “Hello, “
    append_name(greeting) // IDE shows: append_name(greeting.lend)
    print(greeting).await
}

fn append_name(greeting: lent String) {
    var name = “world”
    greeting.push_str(name)
}
```

As you can see, the call to `append_name` has a greyed out annotation, indicating that `greeting` is being lent to `append_name`.
