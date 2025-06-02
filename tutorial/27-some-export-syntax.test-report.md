Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/27-some-export-syntax.dada

[Test file](./27-some-export-syntax.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m use wasi.http.fetch;
  [1m[94m|[0m                    [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected `(` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:4
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m    [1m[91m^^^^[0m[1m[94m-[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m    [1m[91m|[0m
  [1m[94m|[0m    [1m[91mI expected this to be followed by `(`[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:8
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:9
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m         [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m            [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:13
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m             [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:17
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m                 [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:20
  [1m[94m|[0m
[1m[94m3 |[0m   fn wasi.cli.main() {
  [1m[94m|[0m  [1m[91m____________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     fetch("https://example.org")
[1m[94m5 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `http`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:1:10
  [1m[94m|[0m
[1m[94m1 |[0m use wasi.http.fetch;
  [1m[94m|[0m          [1m[91m^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `http`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:1:10
  [1m[94m|[0m
[1m[94m1 |[0m use wasi.http.fetch;
  [1m[94m|[0m          [1m[91m^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            13,
        ),
    },
    message: "could not find anything named `http`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    9,
                ),
                end: AbsoluteOffset(
                    13,
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
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m use wasi.http.fetch;
  [1m[94m|[0m                    [1m[91m^[0m [1m[91mhere[0m
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
            20,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected `(` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:4
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m    [1m[91m^^^^[0m[1m[94m-[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m    [1m[91m|[0m
  [1m[94m|[0m    [1m[91mI expected this to be followed by `(`[0m
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
            25,
        ),
        end: AbsoluteOffset(
            29,
        ),
    },
    message: "expected `(` to come next",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    25,
                ),
                end: AbsoluteOffset(
                    29,
                ),
            },
            message: "I expected this to be followed by `(`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    29,
                ),
                end: AbsoluteOffset(
                    30,
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
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:8
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
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
            29,
        ),
        end: AbsoluteOffset(
            30,
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
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:9
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m         [1m[91m^^^[0m [1m[91mhere[0m
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
            30,
        ),
        end: AbsoluteOffset(
            33,
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
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m            [1m[91m^[0m [1m[91mhere[0m
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
            34,
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
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:13
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m             [1m[91m^^^^[0m [1m[91mhere[0m
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
            34,
        ),
        end: AbsoluteOffset(
            38,
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
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:17
  [1m[94m|[0m
[1m[94m3 |[0m fn wasi.cli.main() {
  [1m[94m|[0m                 [1m[91m^^[0m [1m[91mhere[0m
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
            38,
        ),
        end: AbsoluteOffset(
            40,
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
 [1m[94m-->[0m tests/tutorial/from_rust/27-some-export-syntax.dada:3:20
  [1m[94m|[0m
[1m[94m3 |[0m   fn wasi.cli.main() {
  [1m[94m|[0m  [1m[91m____________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     fetch("https://example.org")
[1m[94m5 |[0m [1m[91m|[0m }
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
            41,
        ),
        end: AbsoluteOffset(
            77,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

