Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada

[Test file](./12b-owned-vs-copy-types-structs.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m enum Option[T] {
  [1m[94m|[0m [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:6
  [1m[94m|[0m
[1m[94m7 |[0m enum Option[T] {
  [1m[94m|[0m      [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:12
  [1m[94m|[0m
[1m[94m7 |[0m enum Option[T] {
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:16
   [1m[94m|[0m
[1m[94m 7 |[0m   enum Option[T] {
   [1m[94m|[0m  [1m[91m________________^[0m
[1m[94m 8 |[0m [1m[91m|[0m     Some(T),
[1m[94m 9 |[0m [1m[91m|[0m     None
[1m[94m10 |[0m [1m[91m|[0m }
   [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `x`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:3:9
  [1m[94m|[0m
[1m[94m3 |[0m         x + y
  [1m[94m|[0m         [1m[91m^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `y`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:3:13
  [1m[94m|[0m
[1m[94m3 |[0m         x + y
  [1m[94m|[0m             [1m[91m^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `x`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:3:9
  [1m[94m|[0m
[1m[94m3 |[0m         x + y
  [1m[94m|[0m         [1m[91m^[0m [1m[91mI could not find anything with this name :([0m
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
            80,
        ),
        end: AbsoluteOffset(
            81,
        ),
    },
    message: "could not find anything named `x`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    80,
                ),
                end: AbsoluteOffset(
                    81,
                ),
            },
            message: "I could not find anything with this name :(",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `y`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:3:13
  [1m[94m|[0m
[1m[94m3 |[0m         x + y
  [1m[94m|[0m             [1m[91m^[0m [1m[91mI could not find anything with this name :([0m
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
            84,
        ),
        end: AbsoluteOffset(
            85,
        ),
    },
    message: "could not find anything named `y`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    84,
                ),
                end: AbsoluteOffset(
                    85,
                ),
            },
            message: "I could not find anything with this name :(",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m enum Option[T] {
  [1m[94m|[0m [1m[91m^^^^[0m [1m[91mhere[0m
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
            95,
        ),
        end: AbsoluteOffset(
            99,
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
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:6
  [1m[94m|[0m
[1m[94m7 |[0m enum Option[T] {
  [1m[94m|[0m      [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            100,
        ),
        end: AbsoluteOffset(
            106,
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
 [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:12
  [1m[94m|[0m
[1m[94m7 |[0m enum Option[T] {
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
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
            106,
        ),
        end: AbsoluteOffset(
            109,
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
  [1m[94m-->[0m tests/tutorial/from_rust/12b-owned-vs-copy-types-structs.dada:7:16
   [1m[94m|[0m
[1m[94m 7 |[0m   enum Option[T] {
   [1m[94m|[0m  [1m[91m________________^[0m
[1m[94m 8 |[0m [1m[91m|[0m     Some(T),
[1m[94m 9 |[0m [1m[91m|[0m     None
[1m[94m10 |[0m [1m[91m|[0m }
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
            110,
        ),
        end: AbsoluteOffset(
            135,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

