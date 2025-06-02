Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/12-owned-vs-copy-types.dada

[Test file](./12-owned-vs-copy-types.dada)


# Compiler output

```
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m @repr(box) # <-- maybe just infer it? Not decided yet.
  [1m[94m|[0m [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:7:2
  [1m[94m|[0m
[1m[94m7 |[0m @repr(box) # <-- maybe just infer it? Not decided yet.
  [1m[94m|[0m  [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:7:6
  [1m[94m|[0m
[1m[94m7 |[0m @repr(box) # <-- maybe just infer it? Not decided yet.
  [1m[94m|[0m      [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m enum class Expression {
  [1m[94m|[0m [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected `:` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m       drop {
  [1m[94m|[0m  [1m[94m_____[0m[1m[91m^^^^[0m[1m[94m_-[0m
  [1m[94m|[0m [1m[94m|[0m     [1m[91m|[0m
  [1m[94m|[0m [1m[94m|[0m     [1m[91mI expected this to be followed by `:`[0m
[1m[94m3 |[0m [1m[94m|[0m         # owned types can have a destructor
[1m[94m4 |[0m [1m[94m|[0m     }
  [1m[94m|[0m [1m[94m|_____-[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:2:10
  [1m[94m|[0m
[1m[94m2 |[0m       drop {
  [1m[94m|[0m  [1m[91m__________^[0m
[1m[94m3 |[0m [1m[91m|[0m         # owned types can have a destructor
[1m[94m4 |[0m [1m[91m|[0m     }
  [1m[94m|[0m [1m[91m|_____^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `Name`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: Name) {
  [1m[94m|[0m                       [1m[91m^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:9:5
  [1m[94m|[0m
[1m[94m9 |[0m     enum class Place {
  [1m[94m|[0m     [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `Name`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: Name) {
  [1m[94m|[0m                       [1m[91m^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            22,
        ),
        end: AbsoluteOffset(
            26,
        ),
    },
    message: "could not find anything named `Name`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    22,
                ),
                end: AbsoluteOffset(
                    26,
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
[1m[91merror[0m: [1mexpected `:` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m       drop {
  [1m[94m|[0m  [1m[94m_____[0m[1m[91m^^^^[0m[1m[94m_-[0m
  [1m[94m|[0m [1m[94m|[0m     [1m[91m|[0m
  [1m[94m|[0m [1m[94m|[0m     [1m[91mI expected this to be followed by `:`[0m
[1m[94m3 |[0m [1m[94m|[0m         # owned types can have a destructor
[1m[94m4 |[0m [1m[94m|[0m     }
  [1m[94m|[0m [1m[94m|_____-[0m [1m[94minfo: but instead I saw this[0m
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
    message: "expected `:` to come next",
    labels: [
        DiagnosticLabel {
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
            message: "I expected this to be followed by `:`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    39,
                ),
                end: AbsoluteOffset(
                    90,
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
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:2:10
  [1m[94m|[0m
[1m[94m2 |[0m       drop {
  [1m[94m|[0m  [1m[91m__________^[0m
[1m[94m3 |[0m [1m[91m|[0m         # owned types can have a destructor
[1m[94m4 |[0m [1m[91m|[0m     }
  [1m[94m|[0m [1m[91m|_____^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            39,
        ),
        end: AbsoluteOffset(
            90,
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
                    39,
                ),
                end: AbsoluteOffset(
                    90,
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
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m @repr(box) # <-- maybe just infer it? Not decided yet.
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
            94,
        ),
        end: AbsoluteOffset(
            95,
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
                    94,
                ),
                end: AbsoluteOffset(
                    95,
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
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:7:2
  [1m[94m|[0m
[1m[94m7 |[0m @repr(box) # <-- maybe just infer it? Not decided yet.
  [1m[94m|[0m  [1m[91m^^^^[0m [1m[91mhere[0m
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
            95,
        ),
        end: AbsoluteOffset(
            99,
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
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:7:6
  [1m[94m|[0m
[1m[94m7 |[0m @repr(box) # <-- maybe just infer it? Not decided yet.
  [1m[94m|[0m      [1m[91m^^^^^[0m [1m[91mhere[0m
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
            99,
        ),
        end: AbsoluteOffset(
            104,
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
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m enum class Expression {
  [1m[94m|[0m [1m[91m^^^^[0m [1m[91mhere[0m
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
            149,
        ),
        end: AbsoluteOffset(
            153,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/12-owned-vs-copy-types.dada:9:5
  [1m[94m|[0m
[1m[94m9 |[0m     enum class Place {
  [1m[94m|[0m     [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            177,
        ),
        end: AbsoluteOffset(
            181,
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
                    177,
                ),
                end: AbsoluteOffset(
                    181,
                ),
            },
            message: "I don't know what to do with this, it appears to be extra",
        },
    ],
    children: [],
}
```

