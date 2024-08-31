Test failed: tests/parse_error/fn_named_class.dada

# Unexpected diagnostic

```
error: expected a module-level item
 --> tests/parse_error/fn_named_class.dada:1:12
  |
1 |   fn class() {
  |  ____________^
2 | | #! ^^^^^ expected an identifier
3 | | #!      ^ expected an identifier
4 | | #!      ^ expected a module-level item
5 | | #! expected a module-level itemXXX
  | |___________________________________^ here
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
            11,
        ),
        end: AbsoluteOffset(
            152,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Missing expected diagnostic

```
ExpectedDiagnostic {
    span: MustStartWithin(
        AbsoluteSpan {
            source_file: SourceFile {
                [salsa id]: Id(0),
            },
            start: AbsoluteOffset(
                0,
            ),
            end: AbsoluteOffset(
                12,
            ),
        },
    ),
    message: Regex(
        "expected a module\\-level itemXXX",
    ),
}
```
