Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/18-desugaring-leased-string-arguments.dada

[Test file](./18-desugaring-leased-string-arguments.dada)


# Compiler output

```
[1m[91merror[0m: [1mcould not find anything named `leased`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18-desugaring-leased-string-arguments.dada:1:19
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string(s: leased String) {
  [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `leased`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18-desugaring-leased-string-arguments.dada:1:19
  [1m[94m|[0m
[1m[94m1 |[0m fn read_string(s: leased String) {
  [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
    message: "could not find anything named `leased`",
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
            message: "I could not find anything with this name :(",
        },
    ],
    children: [],
}
```

