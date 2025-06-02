Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/17b-desugaring-shared-string-arguments-v2.dada

[Test file](./17b-desugaring-shared-string-arguments-v2.dada)


# Compiler output

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17b-desugaring-shared-string-arguments-v2.dada:1:16
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string[copy perm P](s: P String) {
  [1m[94m|[0m                [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `P`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17b-desugaring-shared-string-arguments-v2.dada:1:32
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string[copy perm P](s: P String) {
  [1m[94m|[0m                                [1m[91m^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17b-desugaring-shared-string-arguments-v2.dada:1:16
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string[copy perm P](s: P String) {
  [1m[94m|[0m                [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            15,
        ),
        end: AbsoluteOffset(
            19,
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
                    15,
                ),
                end: AbsoluteOffset(
                    19,
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
[1m[91merror[0m: [1mcould not find anything named `P`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/17b-desugaring-shared-string-arguments-v2.dada:1:32
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string[copy perm P](s: P String) {
  [1m[94m|[0m                                [1m[91m^[0m [1m[91mI could not find anything with this name :([0m
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
            31,
        ),
        end: AbsoluteOffset(
            32,
        ),
    },
    message: "could not find anything named `P`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    31,
                ),
                end: AbsoluteOffset(
                    32,
                ),
            },
            message: "I could not find anything with this name :(",
        },
    ],
    children: [],
}
```

