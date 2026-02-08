Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/28b-interesting-example-rust.dada

[Test file](./28b-interesting-example-rust.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28b-interesting-example-rust.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m trait Foo {
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28b-interesting-example-rust.dada:2:7
  [1m[94m|[0m
[1m[94m2 |[0m trait Foo {
  [1m[94m|[0m       [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28b-interesting-example-rust.dada:2:11
  [1m[94m|[0m
[1m[94m2 |[0m   trait Foo {
  [1m[94m|[0m  [1m[91m___________^[0m
[1m[94m3 |[0m [1m[91m|[0m     fn name(&self) -> &str; # must be a reference
[1m[94m4 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28b-interesting-example-rust.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m trait Foo {
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

```
Diagnostic {
    level: Error,
    span: AbsoluteSpan {
        source_file: SourceFile {
            [salsa id]: Id(800),
        },
        start: AbsoluteOffset(
            7,
        ),
        end: AbsoluteOffset(
            12,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28b-interesting-example-rust.dada:2:7
  [1m[94m|[0m
[1m[94m2 |[0m trait Foo {
  [1m[94m|[0m       [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

```
Diagnostic {
    level: Error,
    span: AbsoluteSpan {
        source_file: SourceFile {
            [salsa id]: Id(800),
        },
        start: AbsoluteOffset(
            13,
        ),
        end: AbsoluteOffset(
            16,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28b-interesting-example-rust.dada:2:11
  [1m[94m|[0m
[1m[94m2 |[0m   trait Foo {
  [1m[94m|[0m  [1m[91m___________^[0m
[1m[94m3 |[0m [1m[91m|[0m     fn name(&self) -> &str; # must be a reference
[1m[94m4 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

```
Diagnostic {
    level: Error,
    span: AbsoluteSpan {
        source_file: SourceFile {
            [salsa id]: Id(800),
        },
        start: AbsoluteOffset(
            17,
        ),
        end: AbsoluteOffset(
            70,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

