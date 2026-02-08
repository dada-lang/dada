Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/26-webassembly-components.dada

[Test file](./26-webassembly-components.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/26-webassembly-components.dada:5:1
  [1m[94m|[0m
[1m[94m5 |[0m fetch("https://example.org")
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/26-webassembly-components.dada:5:6
  [1m[94m|[0m
[1m[94m5 |[0m fetch("https://example.org")
  [1m[94m|[0m      [1m[91m^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `http`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/26-webassembly-components.dada:3:10
  [1m[94m|[0m
[1m[94m3 |[0m use wasi.http.fetch
  [1m[94m|[0m          [1m[91m^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `http`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/26-webassembly-components.dada:3:10
  [1m[94m|[0m
[1m[94m3 |[0m use wasi.http.fetch
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
            53,
        ),
        end: AbsoluteOffset(
            57,
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
                    53,
                ),
                end: AbsoluteOffset(
                    57,
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
 [1m[94m-->[0m tests/tutorial/from_rust/26-webassembly-components.dada:5:1
  [1m[94m|[0m
[1m[94m5 |[0m fetch("https://example.org")
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
            65,
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


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/26-webassembly-components.dada:5:6
  [1m[94m|[0m
[1m[94m5 |[0m fetch("https://example.org")
  [1m[94m|[0m      [1m[91m^^^^^^^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
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
            93,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

