Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada

[Test file](./18b-desugaring-leased-string-arguments-v2.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected `Is` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m     leased(P),
  [1m[94m|[0m     [1m[91m^^^^^^[0m[1m[94m---[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m     [1m[91m|[0m
  [1m[94m|[0m     [1m[91mI expected this to be followed by `Is`[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:3:11
  [1m[94m|[0m
[1m[94m3 |[0m     leased(P),
  [1m[94m|[0m           [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:3:14
  [1m[94m|[0m
[1m[94m3 |[0m     leased(P),
  [1m[94m|[0m              [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:4:1
  [1m[94m|[0m
[1m[94m4 |[0m [1m[91m/[0m {
[1m[94m5 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected `Is` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m     leased(P),
  [1m[94m|[0m     [1m[91m^^^^^^[0m[1m[94m---[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m     [1m[91m|[0m
  [1m[94m|[0m     [1m[91mI expected this to be followed by `Is`[0m
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
            44,
        ),
        end: AbsoluteOffset(
            50,
        ),
    },
    message: "expected `Is` to come next",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    44,
                ),
                end: AbsoluteOffset(
                    50,
                ),
            },
            message: "I expected this to be followed by `Is`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    50,
                ),
                end: AbsoluteOffset(
                    53,
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
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:3:11
  [1m[94m|[0m
[1m[94m3 |[0m     leased(P),
  [1m[94m|[0m           [1m[91m^^^[0m [1m[91mhere[0m
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
            50,
        ),
        end: AbsoluteOffset(
            53,
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
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:3:14
  [1m[94m|[0m
[1m[94m3 |[0m     leased(P),
  [1m[94m|[0m              [1m[91m^[0m [1m[91mhere[0m
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
            54,
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
 [1m[94m-->[0m tests/tutorial/from_rust/18b-desugaring-leased-string-arguments-v2.dada:4:1
  [1m[94m|[0m
[1m[94m4 |[0m [1m[91m/[0m {
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
            55,
        ),
        end: AbsoluteOffset(
            58,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

