Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/07-default-accepts-either.dada

[Test file](./07-default-accepts-either.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:8
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:11
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:18
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                  [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                    [1m[91m^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s)      # OK
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:2:12
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s)      # OK
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # OK
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # OK
  [1m[94m|[0m            [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:4:1
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s)      # ERROR
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:4:12
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s)      # ERROR
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1minvalid return value[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m     print("{s}")
  [1m[94m|[0m     [1m[94m------------[0m
  [1m[94m|[0m     [1m[94m|[0m
  [1m[94m|[0m     [1m[91mI expected a value of the return type, but this has type `Future[0-tuple]`[0m
  [1m[94m|[0m     [1m[94minfo: the return type is declared to be `0-tuple`[0m
  [1m[94m|[0m
[1m[94minfo[0m: [1m`Future` and `0-tuple` are distinct types[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m     print("{s}")
  [1m[94m|[0m     [1m[94m------------[0m [1m[94minfo: here[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
            5,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:8
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
            7,
        ),
        end: AbsoluteOffset(
            9,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:11
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
            10,
        ),
        end: AbsoluteOffset(
            16,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:18
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
            17,
        ),
        end: AbsoluteOffset(
            18,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
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
            19,
        ),
        end: AbsoluteOffset(
            32,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s)      # OK
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
            34,
        ),
        end: AbsoluteOffset(
            45,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:2:12
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s)      # OK
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
            45,
        ),
        end: AbsoluteOffset(
            48,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # OK
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
            59,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # OK
  [1m[94m|[0m            [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            78,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:4:1
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s)      # ERROR
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
            84,
        ),
        end: AbsoluteOffset(
            95,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:4:12
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s)      # ERROR
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
            95,
        ),
        end: AbsoluteOffset(
            98,
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
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m     print("{s}")
  [1m[94m|[0m     [1m[94m------------[0m
  [1m[94m|[0m     [1m[94m|[0m
  [1m[94m|[0m     [1m[91mI expected a value of the return type, but this has type `Future[0-tuple]`[0m
  [1m[94m|[0m     [1m[94minfo: the return type is declared to be `0-tuple`[0m
  [1m[94m|[0m
[1m[94minfo[0m: [1m`Future` and `0-tuple` are distinct types[0m
 [1m[94m-->[0m tests/tutorial/from_rust/07-default-accepts-either.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m     print("{s}")
  [1m[94m|[0m     [1m[94m------------[0m [1m[94minfo: here[0m
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
            145,
        ),
        end: AbsoluteOffset(
            157,
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
                    145,
                ),
                end: AbsoluteOffset(
                    157,
                ),
            },
            message: "I expected a value of the return type, but this has type `Future[0-tuple]`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    145,
                ),
                end: AbsoluteOffset(
                    157,
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
                    145,
                ),
                end: AbsoluteOffset(
                    157,
                ),
            },
            message: "`Future` and `0-tuple` are distinct types",
            labels: [],
            children: [],
        },
    ],
}
```

