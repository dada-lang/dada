Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/22-other-permissions.dada

[Test file](./22-other-permissions.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:1
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:5
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:7
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m       [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:9
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m         [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:18
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m                  [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:1
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:2
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:3
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m   [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:14
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m              [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:16
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m                [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:17
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m                 [1m[91m^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:25
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m                         [1m[91m^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:1
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:2
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:3
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m   [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:14
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m              [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:1
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:2
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:3
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m   [1m[91m^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:8
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:9
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m         [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:20
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                    [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:22
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                      [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:23
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                       [1m[91m^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:31
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                               [1m[91m^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:1
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:2
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:3
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m   [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:14
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m              [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:1
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:2
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:3
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m   [1m[91m^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:6
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:7
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m       [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:18
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m                  [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:34
   [1m[94m|[0m
[1m[94m10 |[0m     fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m                                  [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected `:` to come next[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:40
   [1m[94m|[0m
[1m[94m10 |[0m       fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m  [1m[94m________________________________________[0m[1m[91m^^^^^^[0m[1m[94m_-[0m
   [1m[94m|[0m [1m[94m|[0m                                        [1m[91m|[0m
   [1m[94m|[0m [1m[94m|[0m                                        [1m[91mI expected this to be followed by `:`[0m
[1m[94m11 |[0m [1m[94m|[0m         self.name.move
[1m[94m12 |[0m [1m[94m|[0m     }
   [1m[94m|[0m [1m[94m|_____-[0m [1m[94minfo: but instead I saw this[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:47
   [1m[94m|[0m
[1m[94m10 |[0m       fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m  [1m[91m_______________________________________________^[0m
[1m[94m11 |[0m [1m[91m|[0m         self.name.move
[1m[94m12 |[0m [1m[91m|[0m     }
   [1m[94m|[0m [1m[91m|_____^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:9:23
  [1m[94m|[0m
[1m[94m9 |[0m class Character(name: String) {
  [1m[94m|[0m                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `moved`[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:28
   [1m[94m|[0m
[1m[94m10 |[0m     fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m                            [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
   [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:9:23
  [1m[94m|[0m
[1m[94m9 |[0m class Character(name: String) {
  [1m[94m|[0m                       [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            164,
        ),
        end: AbsoluteOffset(
            170,
        ),
    },
    message: "explicit permission required",
    labels: [],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mcould not find anything named `moved`[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:28
   [1m[94m|[0m
[1m[94m10 |[0m     fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m                            [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            201,
        ),
        end: AbsoluteOffset(
            206,
        ),
    },
    message: "could not find anything named `moved`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    201,
                ),
                end: AbsoluteOffset(
                    206,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:34
   [1m[94m|[0m
[1m[94m10 |[0m     fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m                                  [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            207,
        ),
        end: AbsoluteOffset(
            211,
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
                    207,
                ),
                end: AbsoluteOffset(
                    211,
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
[1m[91merror[0m: [1mexpected `:` to come next[0m
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:40
   [1m[94m|[0m
[1m[94m10 |[0m       fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m  [1m[94m________________________________________[0m[1m[91m^^^^^^[0m[1m[94m_-[0m
   [1m[94m|[0m [1m[94m|[0m                                        [1m[91m|[0m
   [1m[94m|[0m [1m[94m|[0m                                        [1m[91mI expected this to be followed by `:`[0m
[1m[94m11 |[0m [1m[94m|[0m         self.name.move
[1m[94m12 |[0m [1m[94m|[0m     }
   [1m[94m|[0m [1m[94m|_____-[0m [1m[94minfo: but instead I saw this[0m
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
            213,
        ),
        end: AbsoluteOffset(
            219,
        ),
    },
    message: "expected `:` to come next",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    213,
                ),
                end: AbsoluteOffset(
                    219,
                ),
            },
            message: "I expected this to be followed by `:`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    220,
                ),
                end: AbsoluteOffset(
                    250,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:10:47
   [1m[94m|[0m
[1m[94m10 |[0m       fn moved_name(self) -> moved[self] String {
   [1m[94m|[0m  [1m[91m_______________________________________________^[0m
[1m[94m11 |[0m [1m[91m|[0m         self.name.move
[1m[94m12 |[0m [1m[91m|[0m     }
   [1m[94m|[0m [1m[91m|_____^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            220,
        ),
        end: AbsoluteOffset(
            250,
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
                    220,
                ),
                end: AbsoluteOffset(
                    250,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:1
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
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
            529,
        ),
        end: AbsoluteOffset(
            532,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:5
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
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
            533,
        ),
        end: AbsoluteOffset(
            534,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:7
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
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
            535,
        ),
        end: AbsoluteOffset(
            536,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:9
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m         [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
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
            537,
        ),
        end: AbsoluteOffset(
            546,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:27:18
   [1m[94m|[0m
[1m[94m27 |[0m let c = Character("Hello")
   [1m[94m|[0m                  [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
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
            546,
        ),
        end: AbsoluteOffset(
            555,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:1
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
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
            556,
        ),
        end: AbsoluteOffset(
            557,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:2
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
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
            557,
        ),
        end: AbsoluteOffset(
            558,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:3
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m   [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            558,
        ),
        end: AbsoluteOffset(
            569,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:14
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m              [1m[91m^^[0m [1m[91mhere[0m
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
            569,
        ),
        end: AbsoluteOffset(
            571,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:16
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
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
            571,
        ),
        end: AbsoluteOffset(
            572,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:17
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
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
            572,
        ),
        end: AbsoluteOffset(
            580,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:28:25
   [1m[94m|[0m
[1m[94m28 |[0m c.leased_name().push_str("Hi") # ERROR
   [1m[94m|[0m                         [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            580,
        ),
        end: AbsoluteOffset(
            586,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:1
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
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
            595,
        ),
        end: AbsoluteOffset(
            596,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:2
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
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
            596,
        ),
        end: AbsoluteOffset(
            597,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:3
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m   [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            597,
        ),
        end: AbsoluteOffset(
            608,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:29:14
   [1m[94m|[0m
[1m[94m29 |[0m c.leased_name() # is being called like `c.share.leased_name()` and `leased[shared] = shared` so you get back a shared string
   [1m[94m|[0m              [1m[91m^^[0m [1m[91mhere[0m
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
            608,
        ),
        end: AbsoluteOffset(
            610,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:1
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
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
            753,
        ),
        end: AbsoluteOffset(
            754,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:2
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
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
            754,
        ),
        end: AbsoluteOffset(
            755,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:3
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m   [1m[91m^^^^^[0m [1m[91mhere[0m
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
            755,
        ),
        end: AbsoluteOffset(
            760,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:8
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
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
            760,
        ),
        end: AbsoluteOffset(
            761,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:9
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m         [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            761,
        ),
        end: AbsoluteOffset(
            772,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:20
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                    [1m[91m^^[0m [1m[91mhere[0m
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
            772,
        ),
        end: AbsoluteOffset(
            774,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:22
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                      [1m[91m^[0m [1m[91mhere[0m
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
            774,
        ),
        end: AbsoluteOffset(
            775,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:23
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                       [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            775,
        ),
        end: AbsoluteOffset(
            783,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:32:31
   [1m[94m|[0m
[1m[94m32 |[0m c.lease.leased_name().push_str("Hi")
   [1m[94m|[0m                               [1m[91m^^^^^^[0m [1m[91mhere[0m
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
            783,
        ),
        end: AbsoluteOffset(
            789,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:1
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
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
            791,
        ),
        end: AbsoluteOffset(
            792,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:2
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
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
            792,
        ),
        end: AbsoluteOffset(
            793,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:3
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m   [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            793,
        ),
        end: AbsoluteOffset(
            804,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:34:14
   [1m[94m|[0m
[1m[94m34 |[0m c.copied_name() # shared[c] String
   [1m[94m|[0m              [1m[91m^^[0m [1m[91mhere[0m
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
            804,
        ),
        end: AbsoluteOffset(
            806,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:1
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
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
            857,
        ),
        end: AbsoluteOffset(
            858,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:2
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
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
            858,
        ),
        end: AbsoluteOffset(
            859,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:3
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m   [1m[91m^^^[0m [1m[91mhere[0m
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
            859,
        ),
        end: AbsoluteOffset(
            862,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:6
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
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
            862,
        ),
        end: AbsoluteOffset(
            863,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:7
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m       [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
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
            863,
        ),
        end: AbsoluteOffset(
            874,
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
  [1m[94m-->[0m tests/tutorial/from_rust/22-other-permissions.dada:36:18
   [1m[94m|[0m
[1m[94m36 |[0m c.our.copied_name() # our String
   [1m[94m|[0m                  [1m[91m^^[0m [1m[91mhere[0m
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
            874,
        ),
        end: AbsoluteOffset(
            876,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

