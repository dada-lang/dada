Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/05-rust-comparison.dada

[Test file](./05-rust-comparison.dada)


# Compiler output

```
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:15
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m               [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:14
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m              [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:16
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m                [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:17
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m                 [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:19
   [1m[94m|[0m
[1m[94m 9 |[0m   struct String<'a> {
   [1m[94m|[0m  [1m[91m___________________^[0m
[1m[94m10 |[0m [1m[91m|[0m     data: &'a [char],
[1m[94m11 |[0m [1m[91m|[0m     length: u32,
[1m[94m12 |[0m [1m[91m|[0m     capacity: u32,
[1m[94m13 |[0m [1m[91m|[0m }
   [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:3:14
  [1m[94m|[0m
[1m[94m3 |[0m     data: Box<[char]>,
  [1m[94m|[0m              [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `Box`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:3:11
  [1m[94m|[0m
[1m[94m3 |[0m     data: Box<[char]>,
  [1m[94m|[0m           [1m[91m^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `Box`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:3:11
  [1m[94m|[0m
[1m[94m3 |[0m     data: Box<[char]>,
  [1m[94m|[0m           [1m[91m^^^[0m [1m[91mI could not find anything with this name :([0m
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
            44,
        ),
    },
    message: "could not find anything named `Box`",
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
                    44,
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
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:3:14
  [1m[94m|[0m
[1m[94m3 |[0m     data: Box<[char]>,
  [1m[94m|[0m              [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            45,
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
                    44,
                ),
                end: AbsoluteOffset(
                    45,
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
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:14
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
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
            122,
        ),
        end: AbsoluteOffset(
            123,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1munrecognized characters(s)[0m
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:15
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m               [1m[91m^[0m [1m[91mI don't know how to interpret these characters[0m
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
            123,
        ),
        end: AbsoluteOffset(
            124,
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
                    123,
                ),
                end: AbsoluteOffset(
                    124,
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
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:16
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m                [1m[91m^[0m [1m[91mhere[0m
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
            124,
        ),
        end: AbsoluteOffset(
            125,
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
 [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:17
  [1m[94m|[0m
[1m[94m9 |[0m struct String<'a> {
  [1m[94m|[0m                 [1m[91m^[0m [1m[91mhere[0m
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
            125,
        ),
        end: AbsoluteOffset(
            126,
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
  [1m[94m-->[0m tests/tutorial/from_rust/05-rust-comparison.dada:9:19
   [1m[94m|[0m
[1m[94m 9 |[0m   struct String<'a> {
   [1m[94m|[0m  [1m[91m___________________^[0m
[1m[94m10 |[0m [1m[91m|[0m     data: &'a [char],
[1m[94m11 |[0m [1m[91m|[0m     length: u32,
[1m[94m12 |[0m [1m[91m|[0m     capacity: u32,
[1m[94m13 |[0m [1m[91m|[0m }
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
            127,
        ),
        end: AbsoluteOffset(
            188,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

