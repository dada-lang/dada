Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/06-sharing.dada

[Test file](./06-sharing.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:6
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:8
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:11
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:18
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                  [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:20
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                    [1m[91m^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m read_string(s.share) # prints "Hello, world"
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:8:12
  [1m[94m|[0m
[1m[94m8 |[0m read_string(s.share) # prints "Hello, world"
  [1m[94m|[0m            [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:9:1
  [1m[94m|[0m
[1m[94m9 |[0m read_string(s) # prints "Hello, world"
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:9:12
  [1m[94m|[0m
[1m[94m9 |[0m read_string(s) # prints "Hello, world"
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:10:1
   [1m[94m|[0m
[1m[94m10 |[0m read_string(s.slice_from(3)) # prints "lo, world"
   [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:10:12
   [1m[94m|[0m
[1m[94m10 |[0m read_string(s.slice_from(3)) # prints "lo, world"
   [1m[94m|[0m            [1m[91m^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:12:19
   [1m[94m|[0m
[1m[94m12 |[0m fn read_string(s: shared String) {
   [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected type to come next[0m
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:12:17
   [1m[94m|[0m
[1m[94m12 |[0m fn read_string(s: shared String) {
   [1m[94m|[0m                 [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
   [1m[94m|[0m                 [1m[91m|[0m
   [1m[94m|[0m                 [1m[91mI expected this to be followed by type[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:12:34
   [1m[94m|[0m
[1m[94m12 |[0m   fn read_string(s: shared String) {
   [1m[94m|[0m  [1m[91m__________________________________^[0m
[1m[94m13 |[0m [1m[91m|[0m     print("{s}")
[1m[94m14 |[0m [1m[91m|[0m }
   [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `Array`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:2:11
  [1m[94m|[0m
[1m[94m2 |[0m     data: Array[u8] # <-- primitive
  [1m[94m|[0m           [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `Array`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:2:11
  [1m[94m|[0m
[1m[94m2 |[0m     data: Array[u8] # <-- primitive
  [1m[94m|[0m           [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            25,
        ),
        end: AbsoluteOffset(
            30,
        ),
    },
    message: "could not find anything named `Array`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    25,
                ),
                end: AbsoluteOffset(
                    30,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
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
            88,
        ),
        end: AbsoluteOffset(
            91,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
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
            92,
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


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:6
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
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
            93,
        ),
        end: AbsoluteOffset(
            94,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:8
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
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
            97,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:11
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            98,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:18
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                  [1m[91m^[0m [1m[91mhere[0m
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
            105,
        ),
        end: AbsoluteOffset(
            106,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:7:20
  [1m[94m|[0m
[1m[94m7 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                    [1m[91m^^^^^^^^^^^^^[0m [1m[91mhere[0m
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
            107,
        ),
        end: AbsoluteOffset(
            120,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m read_string(s.share) # prints "Hello, world"
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            122,
        ),
        end: AbsoluteOffset(
            133,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:8:12
  [1m[94m|[0m
[1m[94m8 |[0m read_string(s.share) # prints "Hello, world"
  [1m[94m|[0m            [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
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
            133,
        ),
        end: AbsoluteOffset(
            142,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:9:1
  [1m[94m|[0m
[1m[94m9 |[0m read_string(s) # prints "Hello, world"
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            167,
        ),
        end: AbsoluteOffset(
            178,
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
 [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:9:12
  [1m[94m|[0m
[1m[94m9 |[0m read_string(s) # prints "Hello, world"
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
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
            178,
        ),
        end: AbsoluteOffset(
            181,
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
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:10:1
   [1m[94m|[0m
[1m[94m10 |[0m read_string(s.slice_from(3)) # prints "lo, world"
   [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            206,
        ),
        end: AbsoluteOffset(
            217,
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
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:10:12
   [1m[94m|[0m
[1m[94m10 |[0m read_string(s.slice_from(3)) # prints "lo, world"
   [1m[94m|[0m            [1m[91m^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
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
            217,
        ),
        end: AbsoluteOffset(
            234,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected type to come next[0m
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:12:17
   [1m[94m|[0m
[1m[94m12 |[0m fn read_string(s: shared String) {
   [1m[94m|[0m                 [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
   [1m[94m|[0m                 [1m[91m|[0m
   [1m[94m|[0m                 [1m[91mI expected this to be followed by type[0m
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
            273,
        ),
        end: AbsoluteOffset(
            274,
        ),
    },
    message: "expected type to come next",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    273,
                ),
                end: AbsoluteOffset(
                    274,
                ),
            },
            message: "I expected this to be followed by type",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    275,
                ),
                end: AbsoluteOffset(
                    281,
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
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:12:19
   [1m[94m|[0m
[1m[94m12 |[0m fn read_string(s: shared String) {
   [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            275,
        ),
        end: AbsoluteOffset(
            281,
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
                    275,
                ),
                end: AbsoluteOffset(
                    281,
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
  [1m[94m-->[0m tests/tutorial/from_rust/06-sharing.dada:12:34
   [1m[94m|[0m
[1m[94m12 |[0m   fn read_string(s: shared String) {
   [1m[94m|[0m  [1m[91m__________________________________^[0m
[1m[94m13 |[0m [1m[91m|[0m     print("{s}")
[1m[94m14 |[0m [1m[91m|[0m }
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
            290,
        ),
        end: AbsoluteOffset(
            310,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

