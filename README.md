# dada


<img src="https://raw.githubusercontent.com/dada-lang/dada-artwork/main/dada.svg" width="300" alt="Tzara"></img>

> I speak only of myself since I do not wish to convince, I have no right to drag others into my river, I oblige no one to follow me and everybody practices his art in his own way.
>
> *Tristan Tzara, "Dada Manifesto 1918”*

## What the heck is Dada?

Dada is a thought experiment. What if we were making a language like Rust, but one that was meant to feel more like Java or JavaScript, and less like C++? One that didn't aspire to being used in kernels or tiny embedded devices and was willing to require a minimal runtime. What might that look like?

## To try it yourself...

As of right now, Dada doesn't really exist, though we have some experimental prototypes. If you'd like to learn more...

* Visit [dada-lang.org](https://dada-lang.org) and read the [tutorial](https://dada-lang.org/docs/dyn_tutorial.html) -- this will give you an idea what we're shooting for.
* On the web, visit https://dada-lang.org/playground to see how things work today.
* On your own computer, clone this repository and do `cargo dada run dada_tests/hello_world.dada` (requires [Rust and Cargo to be installed](https://doc.rust-lang.org/cargo/getting-started/installation.html)).

## Preserved playground demo

This branch preserves the interpreter and borrow-visualization playground from
`5f5cbd92`, immediately before its removal in June 2024.

With rustup and Node.js/npm installed, run `./demo-playground.sh`, then open
http://127.0.0.1:3000/playground. The script uses an April 2024 Rust nightly and
wasm-bindgen 0.2.92, bypassing the old x86-only wasm-pack downloader. It was
verified with Node 24.21.0. Set `PORT` to use another local port.

Try this program, then click within a statement to see the permission graph:

```text
class Point(x, y)

async fn main() {
    let p = Point(22, 44)
    let q = p.lease
    let r = q.lease
    r.x += 1
    print(r.x).await
}
```

It prints `23`. The graph follows the cursor and shows the owning permission
and the two leases. Reload the page if you switch output modes and want to
return to the initial graph view; the historical UI has no dedicated debug radio button.

## License

Licensed under either of [Apache License, Version 2.0][apache] or [MIT license][mit] at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[apache]: LICENSE-APACHE
[mit]: LICENSE-MIT
