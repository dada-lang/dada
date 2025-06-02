Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/13b-value-type-split-counter.dada

[Test file](./13b-value-type-split-counter.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:7
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m       [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:9
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m         [1m[91m^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:16
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m                [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m update(c.lease)
  [1m[94m|[0m [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m update(c.lease)
  [1m[94m|[0m       [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `leased`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:5:14
  [1m[94m|[0m
[1m[94m5 |[0m fn update(c: leased Counter) {
  [1m[94m|[0m              [1m[91m^^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
[1m[91merror[0m: [1minvalid return value[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     c.x += 1
  [1m[94m|[0m       [1m[94m-[0m
  [1m[94m|[0m       [1m[94m|[0m
  [1m[94m|[0m       [1m[91mI expected a value of the return type, but this has type `ref[c.x] <error> u32`[0m
  [1m[94m|[0m       [1m[94minfo: the return type is declared to be `0-tuple`[0m
  [1m[94m|[0m
[1m[94minfo[0m: [1m`u32` and `0-tuple` are distinct types[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     c.x += 1
  [1m[94m|[0m       [1m[94m-[0m [1m[94minfo: here[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:6:9
  [1m[94m|[0m
[1m[94m6 |[0m     c.x += 1
  [1m[94m|[0m         [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
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
            22,
        ),
        end: AbsoluteOffset(
            25,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:5
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
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
            26,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:7
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
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
            28,
        ),
        end: AbsoluteOffset(
            29,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:9
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m         [1m[91m^^^^^^^[0m [1m[91mhere[0m
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
            30,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:2:16
  [1m[94m|[0m
[1m[94m2 |[0m let c = Counter(22)
  [1m[94m|[0m                [1m[91m^^^^[0m [1m[91mhere[0m
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
            37,
        ),
        end: AbsoluteOffset(
            41,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m update(c.lease)
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
            42,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m update(c.lease)
  [1m[94m|[0m       [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
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
            48,
        ),
        end: AbsoluteOffset(
            57,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `leased`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:5:14
  [1m[94m|[0m
[1m[94m5 |[0m fn update(c: leased Counter) {
  [1m[94m|[0m              [1m[91m^^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            78,
        ),
    },
    message: "could not find anything named `leased`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    72,
                ),
                end: AbsoluteOffset(
                    78,
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
[1m[91merror[0m: [1minvalid return value[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     c.x += 1
  [1m[94m|[0m       [1m[94m-[0m
  [1m[94m|[0m       [1m[94m|[0m
  [1m[94m|[0m       [1m[91mI expected a value of the return type, but this has type `ref[c.x] <error> u32`[0m
  [1m[94m|[0m       [1m[94minfo: the return type is declared to be `0-tuple`[0m
  [1m[94m|[0m
[1m[94minfo[0m: [1m`u32` and `0-tuple` are distinct types[0m
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     c.x += 1
  [1m[94m|[0m       [1m[94m-[0m [1m[94minfo: here[0m
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
            96,
        ),
        end: AbsoluteOffset(
            97,
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
                    96,
                ),
                end: AbsoluteOffset(
                    97,
                ),
            },
            message: "I expected a value of the return type, but this has type `ref[c.x] <error> u32`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    96,
                ),
                end: AbsoluteOffset(
                    97,
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
                    96,
                ),
                end: AbsoluteOffset(
                    97,
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
 [1m[94m-->[0m tests/tutorial/from_rust/13b-value-type-split-counter.dada:6:9
  [1m[94m|[0m
[1m[94m6 |[0m     c.x += 1
  [1m[94m|[0m         [1m[91m^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            99,
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
                    98,
                ),
                end: AbsoluteOffset(
                    99,
                ),
            },
            message: "I don't know what to do with this, it appears to be extra",
        },
    ],
    children: [],
}
```

