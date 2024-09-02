Test failed: tests/parse_error/fn_named_class.dada

# Unexpected diagnostic

```
error: expected an identifier
 --> tests/parse_error/fn_named_class.dada:1:4
  |
1 | fn class() {
  |    ^^^^^ here
  |
```

```
Diagnostic {
    level: Error,
    span: AbsoluteSpan {
        source_file: SourceFile {
            [salsa id]: Id(0),
        },
        start: AbsoluteOffset(
            3,
        ),
        end: AbsoluteOffset(
            8,
        ),
    },
    message: "expected an identifier",
    labels: [],
    children: [],
}
```


# Missing expected diagnostic

```
ExpectedDiagnostic {
    span: MustEqual(
        AbsoluteSpan {
            source_file: SourceFile {
                [salsa id]: Id(0),
            },
            start: AbsoluteOffset(
                3,
            ),
            end: AbsoluteOffset(
                7,
            ),
        },
    ),
    message: Regex(
        "expected an identifier",
    ),
}
```
