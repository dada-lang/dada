Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/27b-some-export-syntax-main.dada

[Test file](./27b-some-export-syntax-main.dada)


# Compiler output

```
[1m[91merror[0m: [1mcould not find anything named `print_u32`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27b-some-export-syntax-main.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m     print_u32(i)
  [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
[1m[91merror[0m: [1mnot callable[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27b-some-export-syntax-main.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m     print_u32(i)
  [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mthis is not something you can call[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `print_u32`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27b-some-export-syntax-main.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m     print_u32(i)
  [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            36,
        ),
        end: AbsoluteOffset(
            45,
        ),
    },
    message: "could not find anything named `print_u32`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    36,
                ),
                end: AbsoluteOffset(
                    45,
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
[1m[91merror[0m: [1mnot callable[0m
 [1m[94m-->[0m tests/tutorial/from_rust/27b-some-export-syntax-main.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m     print_u32(i)
  [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mthis is not something you can call[0m
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
            36,
        ),
        end: AbsoluteOffset(
            45,
        ),
    },
    message: "not callable",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    36,
                ),
                end: AbsoluteOffset(
                    45,
                ),
            },
            message: "this is not something you can call",
        },
    ],
    children: [],
}
```

