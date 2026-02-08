Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/13-value-type-split.dada

[Test file](./13-value-type-split.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:7
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m       [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:9
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m         [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m x = 33
  [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:2:3
  [1m[94m|[0m
[1m[94m2 |[0m x = 33
  [1m[94m|[0m   [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m x = 33
  [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m update(x)
  [1m[94m|[0m [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m update(x)
  [1m[94m|[0m       [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1minvalid return value[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:6:5
  [1m[94m|[0m
[1m[94m6 |[0m     x += 1
  [1m[94m|[0m     [1m[94m-[0m
  [1m[94m|[0m     [1m[94m|[0m
  [1m[94m|[0m     [1m[91mI expected a value of the return type, but this has type `ref[x] u32`[0m
  [1m[94m|[0m     [1m[94minfo: the return type is declared to be `0-tuple`[0m
  [1m[94m|[0m
[1m[94minfo[0m: [1m`u32` and `0-tuple` are distinct types[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:6:5
  [1m[94m|[0m
[1m[94m6 |[0m     x += 1
  [1m[94m|[0m     [1m[94m-[0m [1m[94minfo: here[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     x += 1
  [1m[94m|[0m       [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
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
            3,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
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
            4,
        ),
        end: AbsoluteOffset(
            5,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:7
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m       [1m[91m^[0m [1m[91mhere[0m
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
            7,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:1:9
  [1m[94m|[0m
[1m[94m1 |[0m let x = 22
  [1m[94m|[0m         [1m[91m^^[0m [1m[91mhere[0m
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
            8,
        ),
        end: AbsoluteOffset(
            10,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m x = 33
  [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
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
            11,
        ),
        end: AbsoluteOffset(
            12,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:2:3
  [1m[94m|[0m
[1m[94m2 |[0m x = 33
  [1m[94m|[0m   [1m[91m^[0m [1m[91mhere[0m
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
            13,
        ),
        end: AbsoluteOffset(
            14,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m x = 33
  [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
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
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m update(x)
  [1m[94m|[0m [1m[91m^^^^^^[0m [1m[91mhere[0m
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
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m update(x)
  [1m[94m|[0m       [1m[91m^^^[0m [1m[91mhere[0m
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
            24,
        ),
        end: AbsoluteOffset(
            27,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1minvalid return value[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:6:5
  [1m[94m|[0m
[1m[94m6 |[0m     x += 1
  [1m[94m|[0m     [1m[94m-[0m
  [1m[94m|[0m     [1m[94m|[0m
  [1m[94m|[0m     [1m[91mI expected a value of the return type, but this has type `ref[x] u32`[0m
  [1m[94m|[0m     [1m[94minfo: the return type is declared to be `0-tuple`[0m
  [1m[94m|[0m
[1m[94minfo[0m: [1m`u32` and `0-tuple` are distinct types[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:6:5
  [1m[94m|[0m
[1m[94m6 |[0m     x += 1
  [1m[94m|[0m     [1m[94m-[0m [1m[94minfo: here[0m
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
    message: "invalid return value",
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
                    54,
                ),
            },
            message: "I expected a value of the return type, but this has type `ref[x] u32`",
        },
        DiagnosticLabel {
            level: Info,
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
            message: "the return type is declared to be `0-tuple`",
        },
    ],
    children: [
        Diagnostic {
            level: Info,
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
            message: "`u32` and `0-tuple` are distinct types",
            labels: [],
            children: [],
        },
    ],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13-value-type-split.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     x += 1
  [1m[94m|[0m       [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            56,
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
                    55,
                ),
                end: AbsoluteOffset(
                    56,
                ),
            },
            message: "I don't know what to do with this, it appears to be extra",
        },
    ],
    children: [],
}
```

