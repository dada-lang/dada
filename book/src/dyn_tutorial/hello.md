# Hello, Dada!

{{#include ../caveat.md}}

The classic “Hello, World” program in Dada should be quite familiar:

```
async fn main() {
    print(“Hello, Dada”).await
}
```

The main twist that may be surprising is that, like JavaScript, Dada is based exclusively on **async-await**. This means that operations that perform I/O, like `print`, don't execute immediately. Instead, they return a *thunk*, which is basically "code waiting to run" (but not running yet). The thunk doesn't execute until you *await* it by using the `.await` operation. 
