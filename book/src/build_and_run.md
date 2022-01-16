# Building Dada and running tests

If you're interested in contributing to Dada development, the first thing you will want to do is build and run tests. Here's a quick guide to how it works. Note that Dada is implemented in Rust, so you have to [install Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html) first.

## Build and run Dada

Building Dada is easy. Simply clone the repository and type:

```
> cargo build
```

Once it is built, you can run it by doing 

```
> cargo dada --help
```

`dada` is a [cargo alias] for `cargo run`; it turns off some of cargo's output about building things and so forth. If you prefer, you can do

[cargo alias]: https://doc.rust-lang.org/cargo/reference/config.html#alias

```
> cargo run -- --help
```

## Running tests

Like any cargo package, Dada's test suite can be run with `cargo test`. You may also find it convenient to run the Dada test runner alone via...

```
> cargo dada test
```

...as this allows you to pass more options. Read the test runner documentation for more details.

## Checking a particular file for compilation errors

You can check a particular file for compilation errors by using

```
> cargo dada check path/to/file.dada
```

There are other options too, e.g. for dumping out the IR in various stages, check out

```
> cargo dada check --help
```

to see the list.

## Logs and debugging

If you are debugging Dada, you will probably want to see the logs. You can configure them using the `--log` parameter. Dada uses [tracing] so it takes the usual configuration options.

[tracing]: https://docs.rs/tracing/latest/tracing/

For example, this will dump out *all* debug logs:

```
> cargo dada --log debug check dada_tests/hello_world.dada
```

Or you can dump the logs from a particular crate or module:

```
> cargo dada --log dada_brew check dada_tests/hello_world.dada
```

