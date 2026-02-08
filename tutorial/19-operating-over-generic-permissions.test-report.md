Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/19-operating-over-generic-permissions.dada

[Test file](./19-operating-over-generic-permissions.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:3:55
  [1m[94m|[0m
[1m[94m3 |[0m fn get_name(character: Character) -> moved[character] String {
  [1m[94m|[0m                                                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:3:62
  [1m[94m|[0m
[1m[94m3 |[0m   fn get_name(character: Character) -> moved[character] String {
  [1m[94m|[0m  [1m[91m______________________________________________________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     character.name.move
[1m[94m5 |[0m [1m[91m|[0m }
  [1m[94m|[0m [1m[91m|_^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:1
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:5
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:7
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m       [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:9
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m         [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:18
  [1m[94m|[0m
[1m[94m7 |[0m let c = Character("Tzara")
  [1m[94m|[0m                  [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:5
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:8
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:10
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m          [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:18
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m                  [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:1
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:5
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
  [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:8
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
  [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:10
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
  [1m[94m|[0m          [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:18
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
  [1m[94m|[0m                  [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:1
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:5
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m     [1m[91m^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:8
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m        [1m[91m^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:10
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m          [1m[91m^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:18
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m                  [1m[91m^^^^^^^^[0m [1m[91mhere[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: String)
  [1m[94m|[0m                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `moved`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:3:38
  [1m[94m|[0m
[1m[94m3 |[0m fn get_name(character: Character) -> moved[character] String {
  [1m[94m|[0m                                      [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: String)
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:3:38
  [1m[94m|[0m
[1m[94m3 |[0m fn get_name(character: Character) -> moved[character] String {
  [1m[94m|[0m                                      [1m[91m^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            68,
        ),
        end: AbsoluteOffset(
            73,
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
                    68,
                ),
                end: AbsoluteOffset(
                    73,
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
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:3:55
  [1m[94m|[0m
[1m[94m3 |[0m fn get_name(character: Character) -> moved[character] String {
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
            85,
        ),
        end: AbsoluteOffset(
            91,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:3:62
  [1m[94m|[0m
[1m[94m3 |[0m   fn get_name(character: Character) -> moved[character] String {
  [1m[94m|[0m  [1m[91m______________________________________________________________^[0m
[1m[94m4 |[0m [1m[91m|[0m     character.name.move
[1m[94m5 |[0m [1m[91m|[0m }
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
            92,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:1
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
            121,
        ),
        end: AbsoluteOffset(
            124,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:5
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:7
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
            127,
        ),
        end: AbsoluteOffset(
            128,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:9
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
            129,
        ),
        end: AbsoluteOffset(
            138,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:7:18
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
            138,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:1
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
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
            148,
        ),
        end: AbsoluteOffset(
            151,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:5
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
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
            152,
        ),
        end: AbsoluteOffset(
            154,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:8
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
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
            155,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:10
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m          [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            157,
        ),
        end: AbsoluteOffset(
            165,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:8:18
  [1m[94m|[0m
[1m[94m8 |[0m let n1 = get_name(c)      # get_name[shared[c]](c) -> shared[c] String
  [1m[94m|[0m                  [1m[91m^^^[0m [1m[91mhere[0m
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
            165,
        ),
        end: AbsoluteOffset(
            168,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:1
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
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
            219,
        ),
        end: AbsoluteOffset(
            222,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:5
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
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
            223,
        ),
        end: AbsoluteOffset(
            225,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:8
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
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
            226,
        ),
        end: AbsoluteOffset(
            227,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:10
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
  [1m[94m|[0m          [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            236,
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
 [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:9:18
  [1m[94m|[0m
[1m[94m9 |[0m let n2 = get_name(c.lease)# get_name[leased[c]](c) -> leased[c] String
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
            236,
        ),
        end: AbsoluteOffset(
            245,
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
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:1
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
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
            290,
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


# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:5
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
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
            294,
        ),
        end: AbsoluteOffset(
            296,
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
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:8
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
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
            297,
        ),
        end: AbsoluteOffset(
            298,
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
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:10
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m          [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            299,
        ),
        end: AbsoluteOffset(
            307,
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
  [1m[94m-->[0m tests/tutorial/from_rust/19-operating-over-generic-permissions.dada:10:18
   [1m[94m|[0m
[1m[94m10 |[0m let n3 = get_name(c.move) # get_name[my](c.move) -> my String
   [1m[94m|[0m                  [1m[91m^^^^^^^^[0m [1m[91mhere[0m
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
            307,
        ),
        end: AbsoluteOffset(
            315,
        ),
    },
    message: "expected a module-level item",
    labels: [],
    children: [],
}
```

