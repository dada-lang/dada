Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/25-webassembly-inside-compilation.dada

[Test file](./25-webassembly-inside-compilation.dada)


# Compiler output

```
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m @macro(arguments)
  [1m[94m|[0m [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:1:2
  [1m[94m|[0m
[1m[94m1 |[0m @macro(arguments)
  [1m[94m|[0m  [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:1:7
  [1m[94m|[0m
[1m[94m1 |[0m @macro(arguments)
  [1m[94m|[0m       [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:19
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                   [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected `:` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:10
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m          [1m[91m^^^^^^^^^[0m[1m[94m-[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m          [1m[91m|[0m
  [1m[94m|[0m          [1m[91mI expected this to be followed by `:`[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:38
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                                      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:39
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                                       [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:41
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                                         [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:47
  [1m[94m|[0m
[1m[94m4 |[0m   fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m  [1m[91m_______________________________________________^[0m
[1m[94m5 |[0m [1m[91m|[0m     ...
[1m[94m6 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m @macro(arguments)
  [1m[94m|[0m [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
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
            0,
        ),
        end: AbsoluteOffset(
            1,
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
                    0,
                ),
                end: AbsoluteOffset(
                    1,
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
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:1:2
  [1m[94m|[0m
[1m[94m1 |[0m @macro(arguments)
  [1m[94m|[0m  [1m[91m^^^^^[0m [1m[91mhere[0m
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
            1,
        ),
        end: AbsoluteOffset(
            6,
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
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:1:7
  [1m[94m|[0m
[1m[94m1 |[0m @macro(arguments)
  [1m[94m|[0m       [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            6,
        ),
        end: AbsoluteOffset(
            17,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected `:` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:10
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m          [1m[91m^^^^^^^^^[0m[1m[94m-[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m          [1m[91m|[0m
  [1m[94m|[0m          [1m[91mI expected this to be followed by `:`[0m
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
            50,
        ),
    },
    message: "expected `:` to come next",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    41,
                ),
                end: AbsoluteOffset(
                    50,
                ),
            },
            message: "I expected this to be followed by `:`",
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
                    51,
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
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:19
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                   [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            51,
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
                    50,
                ),
                end: AbsoluteOffset(
                    51,
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
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:38
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                                      [1m[91m^[0m [1m[91mhere[0m
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
            69,
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
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:39
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                                       [1m[91m^[0m [1m[91mhere[0m
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
            71,
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
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:41
  [1m[94m|[0m
[1m[94m4 |[0m fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m                                         [1m[91m^^^^^[0m [1m[91mhere[0m
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
            72,
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


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/25-webassembly-inside-compilation.dada:4:47
  [1m[94m|[0m
[1m[94m4 |[0m   fn macro(arguments..., input: FnAst) -> FnAst {
  [1m[94m|[0m  [1m[91m_______________________________________________^[0m
[1m[94m5 |[0m [1m[91m|[0m     ...
[1m[94m6 |[0m [1m[91m|[0m }
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
            78,
        ),
        end: AbsoluteOffset(
            89,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

