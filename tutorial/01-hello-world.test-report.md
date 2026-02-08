Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/01-hello-world.dada

[Test file](./01-hello-world.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/01-hello-world.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m print("Hello, world")
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/01-hello-world.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m print("Hello, world")
  [1m[94m|[0m      [1m[91m^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/01-hello-world.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m print("Hello, world")
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
            0,
        ),
        end: AbsoluteOffset(
            5,
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
 [1m[94m-->[0m tests/tutorial/from_rust/01-hello-world.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m print("Hello, world")
  [1m[94m|[0m      [1m[91m^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
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
            5,
        ),
        end: AbsoluteOffset(
            21,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

