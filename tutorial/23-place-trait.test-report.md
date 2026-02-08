Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/23-place-trait.dada

[Test file](./23-place-trait.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m trait Place[perm P] {
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m trait Place[perm P] {
  [1m[94m|[0m       [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m trait Place[perm P] {
  [1m[94m|[0m            [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:21
  [1m[94m|[0m
[1m[94m3 |[0m   trait Place[perm P] {
  [1m[94m|[0m  [1m[91m_____________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     type Output;
[1m[94m5 |[0m [1m[91m|[0m     fn get(P self) -> P Self::Output;
[1m[94m6 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m trait Place[perm P] {
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
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
            100,
        ),
        end: AbsoluteOffset(
            105,
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
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m trait Place[perm P] {
  [1m[94m|[0m       [1m[91m^^^^^[0m [1m[91mhere[0m
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
            106,
        ),
        end: AbsoluteOffset(
            111,
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
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m trait Place[perm P] {
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
            111,
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
 [1m[94m-->[0m tests/tutorial/from_rust/23-place-trait.dada:3:21
  [1m[94m|[0m
[1m[94m3 |[0m   trait Place[perm P] {
  [1m[94m|[0m  [1m[91m_____________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     type Output;
[1m[94m5 |[0m [1m[91m|[0m     fn get(P self) -> P Self::Output;
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
            120,
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

