Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada

[Test file](./17-desugaring-shared-string-arguments.dada)


# Compiler output

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada:1:19
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string(s: shared String) {
  [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected type to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada:1:17
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string(s: shared String) {
  [1m[94m|[0m                 [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m                 [1m[91m|[0m
  [1m[94m|[0m                 [1m[91mI expected this to be followed by type[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada:1:34
  [1m[94m|[0m
[1m[94m1 |[0m   fn read_string(s: shared String) {
  [1m[94m|[0m  [1m[91m__________________________________^[0m
[1m[94m2 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected type to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada:1:17
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string(s: shared String) {
  [1m[94m|[0m                 [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m                 [1m[91m|[0m
  [1m[94m|[0m                 [1m[91mI expected this to be followed by type[0m
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
            16,
        ),
        end: AbsoluteOffset(
            17,
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
                    16,
                ),
                end: AbsoluteOffset(
                    17,
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
                    18,
                ),
                end: AbsoluteOffset(
                    24,
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
 [1m[94m-->[0m tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada:1:19
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string(s: shared String) {
  [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            18,
        ),
        end: AbsoluteOffset(
            24,
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
                    18,
                ),
                end: AbsoluteOffset(
                    24,
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
 [1m[94m-->[0m tests/tutorial/from_rust/17-desugaring-shared-string-arguments.dada:1:34
  [1m[94m|[0m
[1m[94m1 |[0m   fn read_string(s: shared String) {
  [1m[94m|[0m  [1m[91m__________________________________^[0m
[1m[94m2 |[0m [1m[91m|[0m }
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
            33,
        ),
        end: AbsoluteOffset(
            36,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

