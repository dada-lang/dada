Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada

[Test file](./17d-desugaring-shared-string-arguments-rust-like.dada)


# Compiler output

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:16
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                [1m[91m^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:22
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                      [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:29
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                             [1m[91m^^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected type to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:27
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                           [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m                           [1m[91m|[0m
  [1m[94m|[0m                           [1m[91mI expected this to be followed by type[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m [1m[91m/[0m {
[1m[94m4 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:16
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                [1m[91m^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            57,
        ),
        end: AbsoluteOffset(
            62,
        ),
    },
    message: "extra input",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    57,
                ),
                end: AbsoluteOffset(
                    62,
                ),
            },
            message: "I don't know what to do with this, it appears to be extra",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:22
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                      [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
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
            63,
        ),
        end: AbsoluteOffset(
            64,
        ),
    },
    message: "unrecognized characters(s)",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    63,
                ),
                end: AbsoluteOffset(
                    64,
                ),
            },
            message: "I don't know how to interpret these characters",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected type to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:27
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                           [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m                           [1m[91m|[0m
  [1m[94m|[0m                           [1m[91mI expected this to be followed by type[0m
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
            68,
        ),
        end: AbsoluteOffset(
            69,
        ),
    },
    message: "expected type to come next",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    68,
                ),
                end: AbsoluteOffset(
                    69,
                ),
            },
            message: "I expected this to be followed by type",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    70,
                ),
                end: AbsoluteOffset(
                    76,
                ),
            },
            message: "but instead I saw this",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:2:29
  [1m[94m|[0m
[1m[94m2 |[0m fn read_string[place 'P](s: shared['P] String)
  [1m[94m|[0m                             [1m[91m^^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            70,
        ),
        end: AbsoluteOffset(
            76,
        ),
    },
    message: "extra input",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    70,
                ),
                end: AbsoluteOffset(
                    76,
                ),
            },
            message: "I don't know what to do with this, it appears to be extra",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17d-desugaring-shared-string-arguments-rust-like.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m [1m[91m/[0m {
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
            89,
        ),
        end: AbsoluteOffset(
            92,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

