Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/02-hello-world-with-interpolation.dada

[Test file](./02-hello-world-with-interpolation.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m     [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:10
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m          [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:12
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m            [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m print("Hello, {name}")
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:2:6
  [1m[94m|[0m
[1m[94m2 |[0m print("Hello, {name}")
  [1m[94m|[0m      [1m[91m^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
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
            3,
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
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m     [1m[91m^^^^[0m [1m[91mhere[0m
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
            4,
        ),
        end: AbsoluteOffset(
            8,
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
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:10
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m          [1m[91m^[0m [1m[91mhere[0m
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
            9,
        ),
        end: AbsoluteOffset(
            10,
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
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:1:12
  [1m[94m|[0m
[1m[94m1 |[0m let name = "world"
  [1m[94m|[0m            [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            11,
        ),
        end: AbsoluteOffset(
            17,
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
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m print("Hello, {name}")
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
            19,
        ),
        end: AbsoluteOffset(
            24,
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
 [1m[94m-->[0m tests/tutorial/from_rust/02-hello-world-with-interpolation.dada:2:6
  [1m[94m|[0m
[1m[94m2 |[0m print("Hello, {name}")
  [1m[94m|[0m      [1m[91m^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
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
            24,
        ),
        end: AbsoluteOffset(
            41,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

