Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/08-leasing.dada

[Test file](./08-leasing.dada)


# Compiler output

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:1
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:5
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m     [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:6
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m      [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:8
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m        [1m[91m^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:11
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m           [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:18
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m                  [1m[91m^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:20
  [1m[94m|[0m
[1m[94m1 |[0m let s: my String = "Hello, "
  [1m[94m|[0m                    [1m[91m^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m push_string(s.lease)
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:2:12
  [1m[94m|[0m
[1m[94m2 |[0m push_string(s.lease)
  [1m[94m|[0m            [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s)
  [1m[94m|[0m [1m[91m^^^^^^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s)
  [1m[94m|[0m            [1m[91m^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1minvalid return value[0m
  [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:11:5
   [1m[94m|[0m
[1m[94m11 |[0m     print("{s}")
   [1m[94m|[0m     [1m[94m------------[0m
   [1m[94m|[0m     [1m[94m|[0m
   [1m[94m|[0m     [1m[91mI expected a value of the return type, but this has type `Future[0-tuple]`[0m
   [1m[94m|[0m     [1m[94minfo: the return type is declared to be `0-tuple`[0m
   [1m[94m|[0m
[1m[94minfo[0m: [1m`Future` and `0-tuple` are distinct types[0m
  [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:11:5
   [1m[94m|[0m
[1m[94m11 |[0m     print("{s}")
   [1m[94m|[0m     [1m[94m------------[0m [1m[94minfo: here[0m
   [1m[94m|[0m
[1m[91merror[0m: [1mcould not find anything named `leased`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:5:19
  [1m[94m|[0m
[1m[94m5 |[0m fn push_string(s: leased String) {
  [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI could not find anything with this name :([0m
  [1m[94m|[0m
[1m[91merror[0m: [1munrecognized field or method `lease`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     s.lease.push("world")
  [1m[94m|[0m     [1m[94m-[0m [1m[91m^^^^^[0m [1m[91mI could not find a field or method named `lease`[0m
  [1m[94m|[0m     [1m[94m|[0m
  [1m[94m|[0m     [1m[94minfo: this has type `SymTy { kind: Perm(SymPerm { kind: Error(Reported(AbsoluteSpan { source_file: SourceFile { [salsa id]: Id(800), url: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/dev/dev/dada/tests/tutorial/from_rust/08-leasing.dada", query: None, fragment: None }, contents: Ok("let s: my String = \"Hello, \"\npush_string(s.lease)\nread_string(s)\n\nfn push_string(s: leased String) {\n    s.lease.push(\"world\")\n    read_string(s)\n}\n\nfn read_string(s: String) {\n    print(\"{s}\")\n}") }, start: AbsoluteOffset(84), end: AbsoluteOffset(90) })) }, SymTy { kind: Named(Aggregate(SymAggregate { [salsa id]: Id(4000), super_scope: AstModule(AstModule { [salsa id]: Id(2401), name: Identifier { text: "prelude.dada" }, items: SpanVec { span: Span { start: Offset(0), end: Offset(630), anchor: "..." }, values: [Aggregate(AstAggregate { [salsa id]: Id(3800), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) }), Aggregate(AstAggregate { [salsa id]: Id(3801), span: Span { start: Offset(564), end: Offset(593), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(564), end: Offset(570), anchor: "..." }, kind: Export }), kind: Struct, name: Identifier { text: "Pointer" }, name_span: Span { start: Offset(578), end: Offset(585), anchor: "..." }, generics: Some(SpanVec { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, values: [AstGenericDecl { [salsa id]: Id(3c00), kind: Type(Span { start: Offset(586), end: Offset(590), anchor: "..." }), name: Some(SpannedIdentifier { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, id: Identifier { text: "T" } }) }] }), inputs: None, where_clauses: None, contents: None }), Function(AstFunction { [salsa id]: Id(2002), span: Span { start: Offset(595), end: Offset(630), anchor: "..." }, effects: AstFunctionEffects { async_effect: Some(Span { start: Offset(602), end: Offset(607), anchor: "..." }), unsafe_effect: None }, fn_span: Span { start: Offset(608), end: Offset(610), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(595), end: Offset(601), anchor: "..." }, kind: Export }), name: SpannedIdentifier { span: Span { start: Offset(611), end: Offset(616), anchor: "..." }, id: Identifier { text: "print" } }, generics: None, inputs: SpanVec { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, values: [Variable(VariableDecl { [salsa id]: Id(1c02), mutable: None, name: SpannedIdentifier { span: Span { start: Offset(617), end: Offset(618), anchor: "..." }, id: Identifier { text: "s" } }, perm: None, base_ty: AstTy { [salsa id]: Id(1803), span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, kind: Named(AstPath { [salsa id]: Id(1006), kind: Identifier(SpannedIdentifier { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, id: Identifier { text: "String" } }) }, None) } })] }, output_ty: None, where_clauses: None, body: Some(DeferredParse { span: Span { start: Offset(628), end: Offset(630), anchor: "..." }, contents: "" }) })] } }), source: AstAggregate { [salsa id]: Id(3800), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) } }), []) }) }`, which doesn't appear to have a field or method `lease`[0m
  [1m[94m|[0m
[1m[91merror[0m: [1mnot callable[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     s.lease.push("world")
  [1m[94m|[0m       [1m[91m^^^^^[0m [1m[91mthis is not something you can call[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexpected a module-level item[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:1
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:5
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:6
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:8
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:11
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:18
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:1:20
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:2:1
  [1m[94m|[0m
[1m[94m2 |[0m push_string(s.lease)
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:2:12
  [1m[94m|[0m
[1m[94m2 |[0m push_string(s.lease)
  [1m[94m|[0m            [1m[91m^^^^^^^^^[0m [1m[91mhere[0m
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
            49,
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:3:1
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s)
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
            50,
        ),
        end: AbsoluteOffset(
            61,
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:3:12
  [1m[94m|[0m
[1m[94m3 |[0m read_string(s)
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
            61,
        ),
        end: AbsoluteOffset(
            64,
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
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:5:19
  [1m[94m|[0m
[1m[94m5 |[0m fn push_string(s: leased String) {
  [1m[94m|[0m                   [1m[91m^^^^^^[0m [1m[91mI could not find anything with this name :([0m
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
            84,
        ),
        end: AbsoluteOffset(
            90,
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
                    84,
                ),
                end: AbsoluteOffset(
                    90,
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
[1m[91merror[0m: [1munrecognized field or method `lease`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     s.lease.push("world")
  [1m[94m|[0m     [1m[94m-[0m [1m[91m^^^^^[0m [1m[91mI could not find a field or method named `lease`[0m
  [1m[94m|[0m     [1m[94m|[0m
  [1m[94m|[0m     [1m[94minfo: this has type `SymTy { kind: Perm(SymPerm { kind: Error(Reported(AbsoluteSpan { source_file: SourceFile { [salsa id]: Id(800), url: Url { scheme: "file", cannot_be_a_base: false, username: "", password: None, host: None, port: None, path: "/Users/dev/dev/dada/tests/tutorial/from_rust/08-leasing.dada", query: None, fragment: None }, contents: Ok("let s: my String = \"Hello, \"\npush_string(s.lease)\nread_string(s)\n\nfn push_string(s: leased String) {\n    s.lease.push(\"world\")\n    read_string(s)\n}\n\nfn read_string(s: String) {\n    print(\"{s}\")\n}") }, start: AbsoluteOffset(84), end: AbsoluteOffset(90) })) }, SymTy { kind: Named(Aggregate(SymAggregate { [salsa id]: Id(4000), super_scope: AstModule(AstModule { [salsa id]: Id(2401), name: Identifier { text: "prelude.dada" }, items: SpanVec { span: Span { start: Offset(0), end: Offset(630), anchor: "..." }, values: [Aggregate(AstAggregate { [salsa id]: Id(3800), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) }), Aggregate(AstAggregate { [salsa id]: Id(3801), span: Span { start: Offset(564), end: Offset(593), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(564), end: Offset(570), anchor: "..." }, kind: Export }), kind: Struct, name: Identifier { text: "Pointer" }, name_span: Span { start: Offset(578), end: Offset(585), anchor: "..." }, generics: Some(SpanVec { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, values: [AstGenericDecl { [salsa id]: Id(3c00), kind: Type(Span { start: Offset(586), end: Offset(590), anchor: "..." }), name: Some(SpannedIdentifier { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, id: Identifier { text: "T" } }) }] }), inputs: None, where_clauses: None, contents: None }), Function(AstFunction { [salsa id]: Id(2002), span: Span { start: Offset(595), end: Offset(630), anchor: "..." }, effects: AstFunctionEffects { async_effect: Some(Span { start: Offset(602), end: Offset(607), anchor: "..." }), unsafe_effect: None }, fn_span: Span { start: Offset(608), end: Offset(610), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(595), end: Offset(601), anchor: "..." }, kind: Export }), name: SpannedIdentifier { span: Span { start: Offset(611), end: Offset(616), anchor: "..." }, id: Identifier { text: "print" } }, generics: None, inputs: SpanVec { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, values: [Variable(VariableDecl { [salsa id]: Id(1c02), mutable: None, name: SpannedIdentifier { span: Span { start: Offset(617), end: Offset(618), anchor: "..." }, id: Identifier { text: "s" } }, perm: None, base_ty: AstTy { [salsa id]: Id(1803), span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, kind: Named(AstPath { [salsa id]: Id(1006), kind: Identifier(SpannedIdentifier { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, id: Identifier { text: "String" } }) }, None) } })] }, output_ty: None, where_clauses: None, body: Some(DeferredParse { span: Span { start: Offset(628), end: Offset(630), anchor: "..." }, contents: "" }) })] } }), source: AstAggregate { [salsa id]: Id(3800), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) } }), []) }) }`, which doesn't appear to have a field or method `lease`[0m
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
            107,
        ),
        end: AbsoluteOffset(
            112,
        ),
    },
    message: "unrecognized field or method `lease`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    107,
                ),
                end: AbsoluteOffset(
                    112,
                ),
            },
            message: "I could not find a field or method named `lease`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    105,
                ),
                end: AbsoluteOffset(
                    106,
                ),
            },
            message: "this has type `SymTy { kind: Perm(SymPerm { kind: Error(Reported(AbsoluteSpan { source_file: SourceFile { [salsa id]: Id(800), url: Url { scheme: \"file\", cannot_be_a_base: false, username: \"\", password: None, host: None, port: None, path: \"/Users/dev/dev/dada/tests/tutorial/from_rust/08-leasing.dada\", query: None, fragment: None }, contents: Ok(\"let s: my String = \\\"Hello, \\\"\\npush_string(s.lease)\\nread_string(s)\\n\\nfn push_string(s: leased String) {\\n    s.lease.push(\\\"world\\\")\\n    read_string(s)\\n}\\n\\nfn read_string(s: String) {\\n    print(\\\"{s}\\\")\\n}\") }, start: AbsoluteOffset(84), end: AbsoluteOffset(90) })) }, SymTy { kind: Named(Aggregate(SymAggregate { [salsa id]: Id(4000), super_scope: AstModule(AstModule { [salsa id]: Id(2401), name: Identifier { text: \"prelude.dada\" }, items: SpanVec { span: Span { start: Offset(0), end: Offset(630), anchor: \"...\" }, values: [Aggregate(AstAggregate { [salsa id]: Id(3800), span: Span { start: Offset(0), end: Offset(562), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: \"...\" }, kind: Export }), kind: Class, name: Identifier { text: \"String\" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: \"...\" }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: \"...\" }, contents: \"\\n    data: Pointer[u8]\\n    length: u32\\n    capacity: u32\\n\\n    ## Create a string from a statically allocated byte array.\\n    ## Used to create string literals.\\n    ##\\n    ## # Unsafe\\n    ##\\n    ## The data must be valid indefinitely.\\n    ## The resulting string will not free the data when it is dropped.\\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\\n        String { data: data, length: length, capacity: 0 }\\n    }\\n\\n    ## Get the length of the string.\\n    export fn len(self) -> u32 {\\n        self.length\\n    }\\n\" }) }), Aggregate(AstAggregate { [salsa id]: Id(3801), span: Span { start: Offset(564), end: Offset(593), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(564), end: Offset(570), anchor: \"...\" }, kind: Export }), kind: Struct, name: Identifier { text: \"Pointer\" }, name_span: Span { start: Offset(578), end: Offset(585), anchor: \"...\" }, generics: Some(SpanVec { span: Span { start: Offset(591), end: Offset(592), anchor: \"...\" }, values: [AstGenericDecl { [salsa id]: Id(3c00), kind: Type(Span { start: Offset(586), end: Offset(590), anchor: \"...\" }), name: Some(SpannedIdentifier { span: Span { start: Offset(591), end: Offset(592), anchor: \"...\" }, id: Identifier { text: \"T\" } }) }] }), inputs: None, where_clauses: None, contents: None }), Function(AstFunction { [salsa id]: Id(2002), span: Span { start: Offset(595), end: Offset(630), anchor: \"...\" }, effects: AstFunctionEffects { async_effect: Some(Span { start: Offset(602), end: Offset(607), anchor: \"...\" }), unsafe_effect: None }, fn_span: Span { start: Offset(608), end: Offset(610), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(595), end: Offset(601), anchor: \"...\" }, kind: Export }), name: SpannedIdentifier { span: Span { start: Offset(611), end: Offset(616), anchor: \"...\" }, id: Identifier { text: \"print\" } }, generics: None, inputs: SpanVec { span: Span { start: Offset(620), end: Offset(626), anchor: \"...\" }, values: [Variable(VariableDecl { [salsa id]: Id(1c02), mutable: None, name: SpannedIdentifier { span: Span { start: Offset(617), end: Offset(618), anchor: \"...\" }, id: Identifier { text: \"s\" } }, perm: None, base_ty: AstTy { [salsa id]: Id(1803), span: Span { start: Offset(620), end: Offset(626), anchor: \"...\" }, kind: Named(AstPath { [salsa id]: Id(1006), kind: Identifier(SpannedIdentifier { span: Span { start: Offset(620), end: Offset(626), anchor: \"...\" }, id: Identifier { text: \"String\" } }) }, None) } })] }, output_ty: None, where_clauses: None, body: Some(DeferredParse { span: Span { start: Offset(628), end: Offset(630), anchor: \"...\" }, contents: \"\" }) })] } }), source: AstAggregate { [salsa id]: Id(3800), span: Span { start: Offset(0), end: Offset(562), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: \"...\" }, kind: Export }), kind: Class, name: Identifier { text: \"String\" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: \"...\" }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: \"...\" }, contents: \"\\n    data: Pointer[u8]\\n    length: u32\\n    capacity: u32\\n\\n    ## Create a string from a statically allocated byte array.\\n    ## Used to create string literals.\\n    ##\\n    ## # Unsafe\\n    ##\\n    ## The data must be valid indefinitely.\\n    ## The resulting string will not free the data when it is dropped.\\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\\n        String { data: data, length: length, capacity: 0 }\\n    }\\n\\n    ## Get the length of the string.\\n    export fn len(self) -> u32 {\\n        self.length\\n    }\\n\" }) } }), []) }) }`, which doesn't appear to have a field or method `lease`",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1mnot callable[0m
 [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:6:7
  [1m[94m|[0m
[1m[94m6 |[0m     s.lease.push("world")
  [1m[94m|[0m       [1m[91m^^^^^[0m [1m[91mthis is not something you can call[0m
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
            107,
        ),
        end: AbsoluteOffset(
            112,
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
                    107,
                ),
                end: AbsoluteOffset(
                    112,
                ),
            },
            message: "this is not something you can call",
        },
    ],
    children: [],
}
```


# Unexpected diagnostic

```
[1m[91merror[0m: [1minvalid return value[0m
  [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:11:5
   [1m[94m|[0m
[1m[94m11 |[0m     print("{s}")
   [1m[94m|[0m     [1m[94m------------[0m
   [1m[94m|[0m     [1m[94m|[0m
   [1m[94m|[0m     [1m[91mI expected a value of the return type, but this has type `Future[0-tuple]`[0m
   [1m[94m|[0m     [1m[94minfo: the return type is declared to be `0-tuple`[0m
   [1m[94m|[0m
[1m[94minfo[0m: [1m`Future` and `0-tuple` are distinct types[0m
  [1m[94m-->[0m tests/tutorial/from_rust/08-leasing.dada:11:5
   [1m[94m|[0m
[1m[94m11 |[0m     print("{s}")
   [1m[94m|[0m     [1m[94m------------[0m [1m[94minfo: here[0m
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
            181,
        ),
        end: AbsoluteOffset(
            193,
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
                    181,
                ),
                end: AbsoluteOffset(
                    193,
                ),
            },
            message: "I expected a value of the return type, but this has type `Future[0-tuple]`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    181,
                ),
                end: AbsoluteOffset(
                    193,
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
                    181,
                ),
                end: AbsoluteOffset(
                    193,
                ),
            },
            message: "`Future` and `0-tuple` are distinct types",
            labels: [],
            children: [],
        },
    ],
}
```

