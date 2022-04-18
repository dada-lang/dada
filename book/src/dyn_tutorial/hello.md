# Hello, Dada!

{{#include ../caveat.md}}

The classic “Hello, World” program in Dada should be quite familiar:

```
print("
    I have forced myself to contradict myself
    in order to avoid conforming to my own taste.
      -- Marcel Duchamp
").await
```

When you run this (try it!) it prints:

```
I have forced myself to contradict myself
in order to avoid conforming to my own taste.
  -- Marcel Duchamp
```

There are a few interesting things to note:

* Dada, like JavaScript, is based exclusively on **async-await**. This means that operations that perform I/O, like `print`, don't execute immediately. Instead, they return a *thunk*, which is basically "code waiting to run" (but not running yet). The thunk doesn't execute until you *await* it by using the `.await` operation. 
* Strings in Dada can spread over multiple lines. Leading and trailing whitespace is stripped by default, and we also remove any common indentation from each line.
