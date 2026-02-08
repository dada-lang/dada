Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/24-fn-trait.dada

[Test file](./24-fn-trait.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m trait Fn[perm P, type Args] {
  [1m[94m|[0m [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m trait Fn[perm P, type Args] {
  [1m[94m|[0m       [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:9
  [1m[94m|[0m
[1m[94m3 |[0m trait Fn[perm P, type Args] {
  [1m[94m|[0m         [1m[91m^^^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:29
  [1m[94m|[0m
[1m[94m3 |[0m   trait Fn[perm P, type Args] {
  [1m[94m|[0m  [1m[91m_____________________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     type Output;
[1m[94m5 |[0m [1m[91m|[0m     fn get(P self, args: Args) -> P Self::Output;
[1m[94m6 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m trait Fn[perm P, type Args] {
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
            27,
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
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:7
  [1m[94m|[0m
[1m[94m3 |[0m trait Fn[perm P, type Args] {
  [1m[94m|[0m       [1m[91m^^[0m [1m[91mhere[0m
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
            33,
        ),
        end: AbsoluteOffset(
            35,
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
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:9
  [1m[94m|[0m
[1m[94m3 |[0m trait Fn[perm P, type Args] {
  [1m[94m|[0m         [1m[91m^^^^^^^^^^^^^^^^^^^[0m [1m[91mhere[0m
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
            35,
        ),
        end: AbsoluteOffset(
            54,
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
 [1m[94m-->[0m tests/tutorial/from_rust/24-fn-trait.dada:3:29
  [1m[94m|[0m
[1m[94m3 |[0m   trait Fn[perm P, type Args] {
  [1m[94m|[0m  [1m[91m_____________________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     type Output;
[1m[94m5 |[0m [1m[91m|[0m     fn get(P self, args: Args) -> P Self::Output;
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
            55,
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

