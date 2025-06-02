Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada

[Test file](./14-owned-vs-copy-types-and-permissions.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:8
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:11
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:18
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                  [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, world"
  [1m[94m|[0m                    [1m[91m^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:6
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:8
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:11
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:17
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                 [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:26
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                          [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:28
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                            [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:32
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                                [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:6
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:8
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m        [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:14
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m              [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:18
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                  [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:24
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                        [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:33
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                                 [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:35
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                                   [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:1
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:5
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:6
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:8
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:11
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:18
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:1:20
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
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
            34,
        ),
        end: AbsoluteOffset(
            37,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
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
            38,
        ),
        end: AbsoluteOffset(
            39,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:6
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
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
            39,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:8
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
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
            41,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:11
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
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
            44,
        ),
        end: AbsoluteOffset(
            50,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:17
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                 [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            58,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:26
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                          [1m[91m^[0m [1m[91mhere[0m
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
            60,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:28
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                            [1m[91m^^^^[0m [1m[91mhere[0m
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
            61,
        ),
        end: AbsoluteOffset(
            65,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:2:32
  [1m[94m|[0m
[1m[94m2 |[0m let m: my Option[String] = Some(s.give) # my Option[String] = Option[my String]
  [1m[94m|[0m                                [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            65,
        ),
        end: AbsoluteOffset(
            73,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
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
            114,
        ),
        end: AbsoluteOffset(
            117,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:5
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
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
            118,
        ),
        end: AbsoluteOffset(
            119,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:6
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
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
            119,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:8
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m        [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            121,
        ),
        end: AbsoluteOffset(
            127,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:14
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m              [1m[91m^^^[0m [1m[91mhere[0m
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
            127,
        ),
        end: AbsoluteOffset(
            130,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:18
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                  [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            131,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:24
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                        [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:33
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                                 [1m[91m^[0m [1m[91mhere[0m
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
            146,
        ),
        end: AbsoluteOffset(
            147,
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
 [1m[94m-->[0m tests/tutorial/from_rust/14-owned-vs-copy-types-and-permissions.dada:3:35
  [1m[94m|[0m
[1m[94m3 |[0m let p: shared[m] Option[String] = m     # shared[m] Option[String] = Option[shared[m] String]
  [1m[94m|[0m                                   [1m[91m^[0m [1m[91mhere[0m
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
            148,
        ),
        end: AbsoluteOffset(
            149,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

