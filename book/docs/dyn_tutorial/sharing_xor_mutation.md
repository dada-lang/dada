# Friends don't let friends mutate shared data

import Caveat from '../caveat.md'

<Caveat/>

You may be wondering why you would ever **NOT** use `our` variables! They probably feel more like what you are used to. Many `our` variables can have equal access to the same object at the same time, but that comes with one key limitation: the fields of that object become immutable![^atomic]

[^atomic]: This is not actually true. In a future section, we'll introduce [atomic fields](./atomic.md), which can be mutated even in shared data, but they are only meant to be used in limited situations.

```
class Point(x, y)

let p: our = Point(x: 22, y: 44)
p.x += 1                        # <-- Error!
```

This is, in fact, a core principle of Dada: **Friends don't let friends mutate shared data**[^xor]. What it means is that we like sharing, and we like mutation, but we don't like having them both at the same time: that way leads to despair. Well, bugs anyway.

[^xor]: A shorter, if less playful, alternative is **sharing xor mutation**.

## Aside: What's wrong with sharing and mutation?

The _friends don't let friends mutate shared data_ principle needs a bit of justification: it's a pretty big shift from how most languages work! To give you a feeling for why this is such a bedrock principle of Dada, let's look at two examples. For these examples, we'll be using Python, but the same kinds of examples can be constructed in basically any language that permits mutation.

### Example 1: Data race

You're probably familiar with the concept of a _data race_: these occur when you have two threads running in parallel and modifying the same piece of data without any kind of synchronization or coordination. The end result is (typically) unpredictable and arbitrary. Here is some Python code that starts 10 threads, each of which increment the same counter 2 million times. At the end, it prints the counter. What do you think it will print? You'd _like_ to think it will print 20 million. But it won't -- or at least, it might not.

```python
import threading, os

counter = [0]

class MyThread(threading.Thread):
    def run(self):
        for i in range(2000000):
            v = counter[0]
            counter[0] = v + 1

    def launch():
        t = MyThread()
        t.start()
        return t

threads = [MyThread.launch() for i in range(10)]
for t in threads:
    t.join()
print(counter)
```

If you run this, you will likely see different values every time. Why? Because it is possible for another thread to "sneak in" in between these two steps:

```python
v = counter[0]
counter[0] = v + 1
```

In other words, there might be two threads, both of which read a value of N, increment to N+1, and then store N+1 back. Now there were two counter increments, but the counter only changed by one.[^gil]

[^gil]: In Python, it's a bit harder to observe this because of the [Global Interpreter Lock](https://wiki.python.org/moin/GlobalInterpreterLock), but as you can see, it's certainly possible.

### Example 2: Iterator invalidation

"Ok", you're thinking, "I know data races are bad. But why should I avoid sharing and mutation in sequential code?" Good question. It turns out that data races are really just a one case of a more general problem. Consider this Python function, which copies all the elements from one list to another:

```python
def transfer(source, target):
    for e in source:
        target.append(e)
    return target
```

Looks reasonable, right? Now, what do you think happens if I do this?

```python
l = [1, 2, 3]
transfer(l, l)
```

Answer: in Python, you get an infinite loop. What about in Java? There, if you're lucky, you get an [exception](https://docs.oracle.com/javase/7/docs/api/java/util/ConcurrentModificationException.html); otherwise, you get undefined results. What about in C++? There, this is called [iterator invalidation](https://wiki.c2.com/?IteratorInvalidationProblem), and it can lead to crashes or even security vulnerabilities.

Fundamentally, the problem here is that `transfer` was expecting to read from `source` and write to `target`; it was **not** expecting that those writes would also change `source`. This turns out to be a very general thing. **Most of the time, when we are writing code that writes to one variable, we don't expect that it will caues _other_ variables to change their state.**

Functional languages respond to this problem by preventing _all_ mutation. That certainly works. Languages like Rust and Dada respond by preventing mutation and sharing from happening at the same time. That works too, and it gives you more flexibility.

## But... what if I _want_ a shared counter?

"OK", you're thinking, "I get that sharing and mutation can be dangerous, but what if I want a shared counter? How do I do _that?_" That's another good question! Dada has a mechanism for doing this called transactions, and they're covered in a [future section of this tutorial](./atomic.md). The short version, though, is that you can declare when you want to have fields that are mutable even when shared and then modify them: but you can only do it inside a transaction. Inside of that transaction, the Dada runtime ensures that there aren't overlapping reads and writes to the same object from two different variables or two different threads. So we still have sharing xor mutation, but it is enforced differently.
