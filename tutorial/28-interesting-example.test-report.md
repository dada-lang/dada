Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/28-interesting-example.dada

[Test file](./28-interesting-example.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected type to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:37
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                     [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m                                     [1m[91m|[0m
  [1m[94m|[0m                                     [1m[91mI expected this to be followed by type[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:39
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:45
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                             [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:55
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:62
  [1m[94m|[0m
[1m[94m1 |[0m   fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m  [1m[91m______________________________________________________________^[0m
[1m[94m2 |[0m [1m[91m|[0m     for string in strings {
[1m[94m...[0m [1m[91m|[0m
[1m[94m8 |[0m [1m[91m|[0m     "foo"
[1m[94m9 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `print_u32`[0m
  [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:13:5
   [1m[94m|[0m
[1m[94m13 |[0m     print_u32(i)
   [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mI could not find anything with this name :([0m
   [1m[94m|[0m
[1m[91merror[0m: [1mnot callable[0m
  [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:13:5
   [1m[94m|[0m
[1m[94m13 |[0m     print_u32(i)
   [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mthis is not something you can call[0m
   [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected type to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:37
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                     [1m[91m^[0m [1m[94m------[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m                                     [1m[91m|[0m
  [1m[94m|[0m                                     [1m[91mI expected this to be followed by type[0m
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
            36,
        ),
        end: AbsoluteOffset(
            37,
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
                    36,
                ),
                end: AbsoluteOffset(
                    37,
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
                    38,
                ),
                end: AbsoluteOffset(
                    44,
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
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:39
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                       [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            44,
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
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:45
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                             [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
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
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:55
  [1m[94m|[0m
[1m[94m1 |[0m fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m                                                       [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            54,
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
 [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:1:62
  [1m[94m|[0m
[1m[94m1 |[0m   fn read_data(strings: Vec[String]) -> shared[strings] String {
  [1m[94m|[0m  [1m[91m______________________________________________________________^[0m
[1m[94m2 |[0m [1m[91m|[0m     for string in strings {
[1m[94m...[0m [1m[91m|[0m
[1m[94m8 |[0m [1m[91m|[0m     "foo"
[1m[94m9 |[0m [1m[91m|[0m }
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
            61,
        ),
        end: AbsoluteOffset(
            190,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `print_u32`[0m
  [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:13:5
   [1m[94m|[0m
[1m[94m13 |[0m     print_u32(i)
   [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            228,
        ),
        end: AbsoluteOffset(
            237,
        ),
    },
    message: "could not find anything named `print_u32`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    228,
                ),
                end: AbsoluteOffset(
                    237,
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
[1m[91merror[0m: [1mnot callable[0m
  [1m[94m-->[0m tests/tutorial/from_rust/28-interesting-example.dada:13:5
   [1m[94m|[0m
[1m[94m13 |[0m     print_u32(i)
   [1m[94m|[0m     [1m[91m^^^^^^^^^[0m [1m[91mthis is not something you can call[0m
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
            228,
        ),
        end: AbsoluteOffset(
            237,
        ),
    },
    message: "not callable",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    228,
                ),
                end: AbsoluteOffset(
                    237,
                ),
            },
            message: "this is not something you can call",
        },
    ],
    children: [],
}
```

