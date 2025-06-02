Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/21-methods.dada

[Test file](./21-methods.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:7
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m       [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:9
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m         [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:18
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m                  [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:5
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:8
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:10
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m          [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:11
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m           [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:12
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m            [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:20
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m                    [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:1
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:2
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m  [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:3
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m   [1m[91m^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:8
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:9
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m         [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:17
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                 [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:19
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                   [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:20
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                    [1m[91m^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:24
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                        [1m[91m^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:1
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:5
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:8
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:10
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m          [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:11
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m           [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:12
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m            [1m[91m^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:16
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m                [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:17
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m                 [1m[91m^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:25
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m                         [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:32
  [1m[94m|[0m
[1m[94m2 |[0m     fn get_name(self) -> moved[self] String {
  [1m[94m|[0m                                [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected `:` to come next[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:38
  [1m[94m|[0m
[1m[94m2 |[0m       fn get_name(self) -> moved[self] String {
  [1m[94m|[0m  [1m[94m______________________________________[0m[1m[91m^^^^^^[0m[1m[94m_-[0m
  [1m[94m|[0m [1m[94m|[0m                                      [1m[91m|[0m
  [1m[94m|[0m [1m[94m|[0m                                      [1m[91mI expected this to be followed by `:`[0m
[1m[94m3 |[0m [1m[94m|[0m         self.name.move
[1m[94m4 |[0m [1m[94m|[0m     }
  [1m[94m|[0m [1m[94m|_____-[0m [1m[94minfo: but instead I saw this[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mextra input[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:45
  [1m[94m|[0m
[1m[94m2 |[0m       fn get_name(self) -> moved[self] String {
  [1m[94m|[0m  [1m[91m_____________________________________________^[0m
[1m[94m3 |[0m [1m[91m|[0m         self.name.move
[1m[94m4 |[0m [1m[91m|[0m     }
  [1m[94m|[0m [1m[91m|_____^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: String) {
  [1m[94m|[0m                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `moved`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:26
  [1m[94m|[0m
[1m[94m2 |[0m     fn get_name(self) -> moved[self] String {
  [1m[94m|[0m                          [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: String) {
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
            22,
        ),
        end: AbsoluteOffset(
            28,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:26
  [1m[94m|[0m
[1m[94m2 |[0m     fn get_name(self) -> moved[self] String {
  [1m[94m|[0m                          [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            57,
        ),
        end: AbsoluteOffset(
            62,
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
                    57,
                ),
                end: AbsoluteOffset(
                    62,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:32
  [1m[94m|[0m
[1m[94m2 |[0m     fn get_name(self) -> moved[self] String {
  [1m[94m|[0m                                [1m[91m^^^^[0m [1m[91mI don't know what to do with this, it appears to be extra[0m
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
            63,
        ),
        end: AbsoluteOffset(
            67,
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
                    63,
                ),
                end: AbsoluteOffset(
                    67,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:38
  [1m[94m|[0m
[1m[94m2 |[0m       fn get_name(self) -> moved[self] String {
  [1m[94m|[0m  [1m[94m______________________________________[0m[1m[91m^^^^^^[0m[1m[94m_-[0m
  [1m[94m|[0m [1m[94m|[0m                                      [1m[91m|[0m
  [1m[94m|[0m [1m[94m|[0m                                      [1m[91mI expected this to be followed by `:`[0m
[1m[94m3 |[0m [1m[94m|[0m         self.name.move
[1m[94m4 |[0m [1m[94m|[0m     }
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
            69,
        ),
        end: AbsoluteOffset(
            75,
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
                    69,
                ),
                end: AbsoluteOffset(
                    75,
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
                    76,
                ),
                end: AbsoluteOffset(
                    106,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:2:45
  [1m[94m|[0m
[1m[94m2 |[0m       fn get_name(self) -> moved[self] String {
  [1m[94m|[0m  [1m[91m_____________________________________________^[0m
[1m[94m3 |[0m [1m[91m|[0m         self.name.move
[1m[94m4 |[0m [1m[91m|[0m     }
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
            76,
        ),
        end: AbsoluteOffset(
            106,
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
                    76,
                ),
                end: AbsoluteOffset(
                    106,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
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
            110,
        ),
        end: AbsoluteOffset(
            113,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
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
            114,
        ),
        end: AbsoluteOffset(
            115,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:7
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
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
            116,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:9
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
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
            118,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:7:18
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
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
            127,
        ),
        end: AbsoluteOffset(
            136,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
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
            137,
        ),
        end: AbsoluteOffset(
            140,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:5
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
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
            141,
        ),
        end: AbsoluteOffset(
            143,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:8
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
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
            144,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:10
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m          [1m[91m^[0m [1m[91mhere[0m
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:11
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
  [1m[94m|[0m           [1m[91m^[0m [1m[91mhere[0m
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
            147,
        ),
        end: AbsoluteOffset(
            148,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:12
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
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
            148,
        ),
        end: AbsoluteOffset(
            156,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:8:20
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = c.get_name()       # shared[c] String
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
            156,
        ),
        end: AbsoluteOffset(
            158,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:1
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
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
            184,
        ),
        end: AbsoluteOffset(
            185,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:2
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
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
            185,
        ),
        end: AbsoluteOffset(
            186,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:3
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
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
            186,
        ),
        end: AbsoluteOffset(
            191,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:8
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
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
            191,
        ),
        end: AbsoluteOffset(
            192,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:9
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m         [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            192,
        ),
        end: AbsoluteOffset(
            200,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:17
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                 [1m[91m^^[0m [1m[91mhere[0m
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
            200,
        ),
        end: AbsoluteOffset(
            202,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:19
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                   [1m[91m^[0m [1m[91mhere[0m
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
            202,
        ),
        end: AbsoluteOffset(
            203,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:20
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                    [1m[91m^^^^[0m [1m[91mhere[0m
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
            203,
        ),
        end: AbsoluteOffset(
            207,
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
 [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:9:24
  [1m[94m|[0m
[1m[94m9 |[0m c.lease.get_name().push(" Tzara") # modifies the name in place to be "Tzara Tzara"
  [1m[94m|[0m                        [1m[91m^^^^^^^^^^[0m [1m[91mhere[0m
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:1
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
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
            267,
        ),
        end: AbsoluteOffset(
            270,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:5
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
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
            271,
        ),
        end: AbsoluteOffset(
            273,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:8
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
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
            274,
        ),
        end: AbsoluteOffset(
            275,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:10
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m          [1m[91m^[0m [1m[91mhere[0m
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
            276,
        ),
        end: AbsoluteOffset(
            277,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:11
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m           [1m[91m^[0m [1m[91mhere[0m
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
            277,
        ),
        end: AbsoluteOffset(
            278,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:12
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m            [1m[91m^^^^[0m [1m[91mhere[0m
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
            278,
        ),
        end: AbsoluteOffset(
            282,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:16
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
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
            282,
        ),
        end: AbsoluteOffset(
            283,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:17
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
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
            283,
        ),
        end: AbsoluteOffset(
            291,
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
  [1m[94m-->[0m tests/tutorial/from_rust/21-methods.dada:10:25
   [1m[94m|[0m
[1m[94m10 |[0m let n2 = c.move.get_name()  # my String
   [1m[94m|[0m                         [1m[91m^^[0m [1m[91mhere[0m
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
            291,
        ),
        end: AbsoluteOffset(
            293,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

