Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/16-permissions-example.dada

[Test file](./16-permissions-example.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:8
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:11
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:18
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m                  [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m                    [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s) # ==> read_string[shared[s]](s)
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:2:12
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s) # ==> read_string[shared[s]](s)
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # ==> read_string[my](s.move)
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # ==> read_string[my](s.move)
  [1m[94m|[0m            [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:4:1
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s.move) # ==> read_string[my](s.move), ERROR
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:4:12
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s.move) # ==> read_string[my](s.move), ERROR
  [1m[94m|[0m            [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:8
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:11
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:18
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m                    [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s) # ==> read_string[shared[s]](s)
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
            29,
        ),
        end: AbsoluteOffset(
            40,
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:2:12
  [1m[94m|[0m
[1m[94m2 |[0m read_string(s) # ==> read_string[shared[s]](s)
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
            40,
        ),
        end: AbsoluteOffset(
            43,
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # ==> read_string[my](s.move)
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
            76,
        ),
        end: AbsoluteOffset(
            87,
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s.move) # ==> read_string[my](s.move)
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
            87,
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:4:1
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s.move) # ==> read_string[my](s.move), ERROR
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
            126,
        ),
        end: AbsoluteOffset(
            137,
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
 [1m[94m-->[0m tests/tutorial/from_rust/16-permissions-example.dada:4:12
  [1m[94m|[0m
[1m[94m4 |[0m read_string(s.move) # ==> read_string[my](s.move), ERROR
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
            137,
        ),
        end: AbsoluteOffset(
            145,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

