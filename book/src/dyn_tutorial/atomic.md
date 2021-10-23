# Atomic storage and blocks

In the [sharing xor mutability][sxm] section, we discussed how `var` fields, when shared, become immutable. We also talked about the dangers of mixing sharing and mutation. The challenge is that *sometimes* you *really do* want to mix sharing and mutation. As a simple example, we might want to have a shared cache that multiple parts of our code can access. How do we do that?

[sxm]: ./sharing_xor_mutability.md

For this reason, Dada supports a more dynamic variant of [sharing xor mutability][sxm] called "atomic data". The idea is that we can declare some storage as being `atomic` instead of `var`. Unlike `var` storage, atomic storage can be mutated when shared, but only within a **transaction**. The transaction ensures that all of our changes to that shared data occur as a single unit, without interference from either other variables or other threads.

Let's see an example. To start, we'll write a shared counter type using `atomic`. It might seem at first that `atomic` behaves just like `var`. This code, for example, will print `counter is 1`:

```
class Counter(atomic value)

async fn main() {
    var c1 = Counter(0)
    c1.value += 1
    print("counter is {c1.value}").await
}
```

But what happens if we make the variable `c1` a shared variable, instead?

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    c1.value += 1
    print("counter is {c1.value}").await
}
```

When you run this, you'll find that you still get an exception:

```
error: access to shared, atomic field outside of atomic block
|    c1.value += 1
        ^^^^^ when shared, atomic fields can only be accessed in an atomic block
```

In fact, even if we comment out the write, we *still* get an exception:

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    // c1.value += 1
    print("counter is {c1.value}").await
}
```

Running that snippet yields:

```
error: access to shared, atomic field outside of atomic block
|    print("counter is {c1.value}").await
                           ^^^^^ when shared, atomic fields can only be accessed in an atomic block
```

## Atomic blocks

The solution to our problem, as the message says, is to add an *atomic block*:

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    atomic {
        c1.value += 1

        // we'll come back to this:
        //
        // print("counter is {c1.value}").await
    }
}
```

Inside of an atomic block, we are allowed to access and modify `atomic` fields "as if" we had unique access. The runtime tracks what data we are using and looks for conflicts. It also ensures that, if there are multiple threads, the threads execute in some sequential order. In other words, if we had 10 threads each running `atomic { c1.value += 1; }` we could be sure that the counter had the value 10 at the end, as expected.

## Transactions and await

You probably noticed that we commented out the call to `print` in the previous example. What happens if we uncomment it?

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    atomic {
        c1.value += 1

        print("counter is {c1.value}").await
    }
}
```

When you run this, you see the following error:

```
error: await cannot be used inside of an atomic block
|    atomic {
     ------ atomic block started here
...
|        print("counter is {c1.value}").await
                                        ^^^^^ await appears here
```

As the message says, it is not permitted to have an `await` inside of an atomic block. The reason for this is that `atomic` blocks are implemented using *software transactional memory*. The idea is that we monitor the behavior of different threads and detect potential conflicts. If one thread interferes with another, we have to rerun one or both of them to get the final result. Rerunning code that has no side-effects is not a problem. But re-running code that does I/O (such as printing on the screen) would be bad, that would mean you see the same message twice. Since all I/O in Dada is asynchronous, we can guarantee that there are no side-effects by forbidding `await` inside of an `atomic`.

You can make the shared counter work by moving the `await` outside of the atomic block. The value of an atomic block is equal to the value of its last expression, so you can (for example) pass the final value of the counter out from the `atomic` block and store it into a variable `v`, like so:

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    v = atomic {
        c1.value += 1
        c1.value
    }
    print("counter is {v}").await
}
```

Printing the variable `v` doesn't require accessing atomic storage, so there are no problems here.

Another cute way to make this work is to make the value of the `atomic` block be the thunk itself:

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    atomic {
        c1.value += 1

        print("counter is {c1.value}")
    }.await
}
```

The reason this works is that the string `s` to be printed is computed inside the `atomic` block. We then create a thunk for `print(s)` that will hold on to that string `s` and return it out from the `atomic` block. The `await` then operates on the thunk after the `atomic` has completed.

## Interference between threads

The fact that we can't read shared, atomic fields outside of an atomic block is telling us something interesting. For a moment, imagine that we had shared `c1` with other threads: since is a shared value, that is something we would be alloweed to do. 

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    // ... imagine we shared c1 with other threads here
    v = atomic {
        c1.value += 1
        c1.value
    }
    print("counter is {v}").await
}
```

If we had done that, then the value that we are going to print is *not necessarily* the current value of the counter. Consider: if there were multiple threads executing with accessing to `c1`, they too could be incrementing the counter. Those transactions could occur in between the end of our `atomic` section and the start of `print`. This is why we have to both increment and read the value together. If we modify the program to have two `atomic` blocks, it is actually doing something quite different:

```
class Counter(atomic value)

async fn main() {
    c1 = Counter(0)
    // ... imagine we shared c1 with other threads here
    atomic { c1.value += 1}
    v = atomic { c1.value }
    print("counter is {v}").await
}
```

In this version, we are separating the increment from the read. This explicitly makes space for other threads to come in and modify `c1.value` during that time. Therefore, the value we read for `v` might be quite different than the value at the end of the first transaction.

## Interference within a thread

As described in the [sharing xor mutability][sxm] discussion, it is possible to have interference within a single thread as well. Consider this example, where we have a `Point` stored in an atomic field (part of a class `Cell`). There are two references to this cell, `cell1` and `cell2`:

```
class Point(var x, var y)
class Cell(atomic value)

async fn main() {
    cell1 = Cell(Point(x: 22, y: 44))
    cell2 = cell1

    x = atomic {
        point1 = cell1.value.lease
        v = point1.x
        cell2.value.x += 1
        point1.x = v + 1
    }

    print("{x}").await
}
```

What do you think this program prints? Try it and see. OK, I admit it, it's a trick question. The final `print` never executes:

```
error: use of invalidated lease
|    point1 = cell1.value.lease
                          ----- lease issued here
...
|    cell2.value.x += 1
                 - lease invalidated here
|    point1.x = v + 1
     ^^^^^^^^ invalidated lease used here
```

If you step through this program, you can see what's going on. Initially, we [see](https://asciiflow.com/#/share/eJyrVspLzE1VslJyyknMUNJRykmsTC0CcqtjlCpilKwsLYx1YpQqgSwjSyMgqyS1ogTIiVF6NGUPMUgBGRCnDo%2BqmJg8kNrk1JwcQ0JqUdC0XSB9zkB9YD2otikE5GfmlSggzEeVRfMBip3YTHs0pQEZKaC624gMd5cl5pSmQvUkluTnZiajq6iwUjAyItoH2GQeTWlSqLRSMDFBmEKz%2BJ2CFkQwRNj1IKFpm6DKCkDRZkhKcGJNTSi2Ee%2FzGKVapVoAhvupNw%3D%3D) that `point1` is leased from `cell1`:

```
class Point(var x, var y)
class Cell(atomic value)

async fn main() {
    cell1 = Cell(Point(x: 22, y: 44))
    cell2 = cell1

    x = atomic {
        point1 = cell1.value.lease
        //                        ▲
        // ───────────────────────┘
        v = point1.x
        cell2.value.x += 1
        point1.x = v + 1
    }

    print("{x}").await
}

// ┌───────────┐             ┌───────┐           ┌───────┐
// │ cell1     ├─our────────►│ Cell  │           │ Point │
// │           │             │ ────  │           │ ───── │
// │ cell2     ├─our────────►│ value ├─atomic───►│ x: 22 │
// │           │             │       │           │ y: 44 │
// ├───────────┤             └───────┘           └───────┘
// │           │                ▲
// │ point1    ├─leased(cell1)──┘
// │           │
// └───────────┘
```

Moving the cursor to right after the write through `cell2.value`, we see that this lease has been invalidated:

```
class Point(var x, var y)
class Cell(atomic value)

async fn main() {
    cell1 = Cell(Point(x: 22, y: 44))
    cell2 = cell1

    x = atomic {
        point1 = cell1.value.lease
        v = point1.x
        cell2.value.x += 1
        //                ▲
        // ───────────────┘
        point1.x = v + 1
    }

    print("{x}").await
}

// ┌───────────┐             ┌───────┐           ┌───────┐
// │ cell1     ├─our────────►│ Cell  │           │ Point │
// │           │             │ ────  │           │ ───── │
// │ cell2     ├─our────────►│ value ├─atomic───►│ x: 22 │
// │           │             │       │           │ y: 44 │
// ├───────────┤             └───────┘           └───────┘
// │           │
// │ point1    │
// │           │
// └───────────┘
```

If you think back to the rules on [leasing](./lease.md), this makes sense: a lease lasts until the lessor ends it by using the value. **In this case, though, the value that was leased had not one lessor (`cell1`) but *two*, because it is jointly owned.** Either of those lessors can end the lease by using the value again.

To see why this is useful, imagine for a moment that you were writing a function that takes two cells as arguments. Just like the [`transfer`] function that we described in the [sharing xor mutability][sxm] chapter, you don't realize that `cell1` and `cell2` refer to the same object. In that case, this code that you wrote above is probably wrong! It is going to read the value of `x`, mutate it, and then mutate it again, ignoring that write in between. This is precisely the bug we showed as a "data race", but occuring within a single thread. Dada's rules detect this problem and eliminate it.

## Leasing versus multiple writes

In the previous example, it was crucial that we created a `lease`. If we didn't create a lease, the code could execute just fine, for better or worse:

```
class Point(var x, var y)
class Cell(atomic value)

async fn main() {
    cell1 = Cell(Point(x: 22, y: 44))
    cell2 = cell1

    x = atomic {
        v = cell1.value.x
        cell2.value.x += 1
        cell1.value.x = v + 1
    }

    print("{x}").await
}
```

Runnig this, we see the output `23` -- even though there were two increments, only one took effect. Is this right? Wrong? Well, that's for you, as the author of the code, to say.

The idea here is this: **when you lease an object, you are saying "so long as I use this lease, I am not expecting interference from other variables"**. You are, in effect, creating a kind of "mini-transaction". If however you write the code *without* a lease, as we did above, then interference is possible. Just as we saw with multiple `atomic` sections, that may sometimes be what you want!