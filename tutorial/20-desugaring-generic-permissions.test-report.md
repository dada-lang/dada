Test failed: /Users/dev/dev/dada/tests/tutorial/from_rust/20-desugaring-generic-permissions.dada

[Test file](./20-desugaring-generic-permissions.dada)


# Compiler output

```
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/20-desugaring-generic-permissions.dada:1:23
  [1m[94m|[0m
[1m[94m1 |[0m class Character(name: String)
  [1m[94m|[0m                       [1m[91m^^^^^^[0m [1m[91mhere[0m
  [1m[94m|[0m
[1m[91merror[0m: [1munrecognized field or method `move`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/20-desugaring-generic-permissions.dada:4:20
  [1m[94m|[0m
[1m[94m4 |[0m     character.name.move
  [1m[94m|[0m               [1m[94m----[0m [1m[91m^^^^[0m [1m[91mI could not find a field or method named `move`[0m
  [1m[94m|[0m               [1m[94m|[0m
  [1m[94m|[0m               [1m[94minfo: this has type `SymTy { kind: Perm(SymPerm { kind: Var(SymVariable { [salsa id]: Id(4404), kind: Perm, name: Some(Identifier { text: "P" }), span: Span { start: Offset(43), end: Offset(49), anchor: "..." } }) }, SymTy { kind: Named(Aggregate(SymAggregate { [salsa id]: Id(3401), super_scope: AstModule(AstModule { [salsa id]: Id(3001), name: Identifier { text: "prelude.dada" }, items: SpanVec { span: Span { start: Offset(0), end: Offset(630), anchor: "..." }, values: [Aggregate(AstAggregate { [salsa id]: Id(2001), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) }), Aggregate(AstAggregate { [salsa id]: Id(2002), span: Span { start: Offset(564), end: Offset(593), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(564), end: Offset(570), anchor: "..." }, kind: Export }), kind: Struct, name: Identifier { text: "Pointer" }, name_span: Span { start: Offset(578), end: Offset(585), anchor: "..." }, generics: Some(SpanVec { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, values: [AstGenericDecl { [salsa id]: Id(2401), kind: Type(Span { start: Offset(586), end: Offset(590), anchor: "..." }), name: Some(SpannedIdentifier { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, id: Identifier { text: "T" } }) }] }), inputs: None, where_clauses: None, contents: None }), Function(AstFunction { [salsa id]: Id(2c01), span: Span { start: Offset(595), end: Offset(630), anchor: "..." }, effects: AstFunctionEffects { async_effect: Some(Span { start: Offset(602), end: Offset(607), anchor: "..." }), unsafe_effect: None }, fn_span: Span { start: Offset(608), end: Offset(610), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(595), end: Offset(601), anchor: "..." }, kind: Export }), name: SpannedIdentifier { span: Span { start: Offset(611), end: Offset(616), anchor: "..." }, id: Identifier { text: "print" } }, generics: None, inputs: SpanVec { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, values: [Variable(VariableDecl { [salsa id]: Id(1802), mutable: None, name: SpannedIdentifier { span: Span { start: Offset(617), end: Offset(618), anchor: "..." }, id: Identifier { text: "s" } }, perm: None, base_ty: AstTy { [salsa id]: Id(1405), span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, kind: Named(AstPath { [salsa id]: Id(1007), kind: Identifier(SpannedIdentifier { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, id: Identifier { text: "String" } }) }, None) } })] }, output_ty: None, where_clauses: None, body: Some(DeferredParse { span: Span { start: Offset(628), end: Offset(630), anchor: "..." }, contents: "" }) })] } }), source: AstAggregate { [salsa id]: Id(2001), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) } }), []) }) }`, which doesn't appear to have a field or method `move`[0m
  [1m[94m|[0m
```

# Unexpected diagnostic

```
[1m[91merror[0m: [1mexplicit permission required[0m
 [1m[94m-->[0m tests/tutorial/from_rust/20-desugaring-generic-permissions.dada:1:23
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
[1m[91merror[0m: [1munrecognized field or method `move`[0m
 [1m[94m-->[0m tests/tutorial/from_rust/20-desugaring-generic-permissions.dada:4:20
  [1m[94m|[0m
[1m[94m4 |[0m     character.name.move
  [1m[94m|[0m               [1m[94m----[0m [1m[91m^^^^[0m [1m[91mI could not find a field or method named `move`[0m
  [1m[94m|[0m               [1m[94m|[0m
  [1m[94m|[0m               [1m[94minfo: this has type `SymTy { kind: Perm(SymPerm { kind: Var(SymVariable { [salsa id]: Id(4404), kind: Perm, name: Some(Identifier { text: "P" }), span: Span { start: Offset(43), end: Offset(49), anchor: "..." } }) }, SymTy { kind: Named(Aggregate(SymAggregate { [salsa id]: Id(3401), super_scope: AstModule(AstModule { [salsa id]: Id(3001), name: Identifier { text: "prelude.dada" }, items: SpanVec { span: Span { start: Offset(0), end: Offset(630), anchor: "..." }, values: [Aggregate(AstAggregate { [salsa id]: Id(2001), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) }), Aggregate(AstAggregate { [salsa id]: Id(2002), span: Span { start: Offset(564), end: Offset(593), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(564), end: Offset(570), anchor: "..." }, kind: Export }), kind: Struct, name: Identifier { text: "Pointer" }, name_span: Span { start: Offset(578), end: Offset(585), anchor: "..." }, generics: Some(SpanVec { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, values: [AstGenericDecl { [salsa id]: Id(2401), kind: Type(Span { start: Offset(586), end: Offset(590), anchor: "..." }), name: Some(SpannedIdentifier { span: Span { start: Offset(591), end: Offset(592), anchor: "..." }, id: Identifier { text: "T" } }) }] }), inputs: None, where_clauses: None, contents: None }), Function(AstFunction { [salsa id]: Id(2c01), span: Span { start: Offset(595), end: Offset(630), anchor: "..." }, effects: AstFunctionEffects { async_effect: Some(Span { start: Offset(602), end: Offset(607), anchor: "..." }), unsafe_effect: None }, fn_span: Span { start: Offset(608), end: Offset(610), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(595), end: Offset(601), anchor: "..." }, kind: Export }), name: SpannedIdentifier { span: Span { start: Offset(611), end: Offset(616), anchor: "..." }, id: Identifier { text: "print" } }, generics: None, inputs: SpanVec { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, values: [Variable(VariableDecl { [salsa id]: Id(1802), mutable: None, name: SpannedIdentifier { span: Span { start: Offset(617), end: Offset(618), anchor: "..." }, id: Identifier { text: "s" } }, perm: None, base_ty: AstTy { [salsa id]: Id(1405), span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, kind: Named(AstPath { [salsa id]: Id(1007), kind: Identifier(SpannedIdentifier { span: Span { start: Offset(620), end: Offset(626), anchor: "..." }, id: Identifier { text: "String" } }) }, None) } })] }, output_ty: None, where_clauses: None, body: Some(DeferredParse { span: Span { start: Offset(628), end: Offset(630), anchor: "..." }, contents: "" }) })] } }), source: AstAggregate { [salsa id]: Id(2001), span: Span { start: Offset(0), end: Offset(562), anchor: "..." }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: "..." }, kind: Export }), kind: Class, name: Identifier { text: "String" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: "..." }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: "..." }, contents: "\n    data: Pointer[u8]\n    length: u32\n    capacity: u32\n\n    ## Create a string from a statically allocated byte array.\n    ## Used to create string literals.\n    ##\n    ## # Unsafe\n    ##\n    ## The data must be valid indefinitely.\n    ## The resulting string will not free the data when it is dropped.\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\n        String { data: data, length: length, capacity: 0 }\n    }\n\n    ## Get the length of the string.\n    export fn len(self) -> u32 {\n        self.length\n    }\n" }) } }), []) }) }`, which doesn't appear to have a field or method `move`[0m
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
            108,
        ),
        end: AbsoluteOffset(
            112,
        ),
    },
    message: "unrecognized field or method `move`",
    labels: [
        DiagnosticLabel {
            level: Error,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    108,
                ),
                end: AbsoluteOffset(
                    112,
                ),
            },
            message: "I could not find a field or method named `move`",
        },
        DiagnosticLabel {
            level: Info,
            span: AbsoluteSpan {
                source_file: SourceFile {
                    [salsa id]: Id(800),
                },
                start: AbsoluteOffset(
                    103,
                ),
                end: AbsoluteOffset(
                    107,
                ),
            },
            message: "this has type `SymTy { kind: Perm(SymPerm { kind: Var(SymVariable { [salsa id]: Id(4404), kind: Perm, name: Some(Identifier { text: \"P\" }), span: Span { start: Offset(43), end: Offset(49), anchor: \"...\" } }) }, SymTy { kind: Named(Aggregate(SymAggregate { [salsa id]: Id(3401), super_scope: AstModule(AstModule { [salsa id]: Id(3001), name: Identifier { text: \"prelude.dada\" }, items: SpanVec { span: Span { start: Offset(0), end: Offset(630), anchor: \"...\" }, values: [Aggregate(AstAggregate { [salsa id]: Id(2001), span: Span { start: Offset(0), end: Offset(562), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: \"...\" }, kind: Export }), kind: Class, name: Identifier { text: \"String\" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: \"...\" }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: \"...\" }, contents: \"\\n    data: Pointer[u8]\\n    length: u32\\n    capacity: u32\\n\\n    ## Create a string from a statically allocated byte array.\\n    ## Used to create string literals.\\n    ##\\n    ## # Unsafe\\n    ##\\n    ## The data must be valid indefinitely.\\n    ## The resulting string will not free the data when it is dropped.\\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\\n        String { data: data, length: length, capacity: 0 }\\n    }\\n\\n    ## Get the length of the string.\\n    export fn len(self) -> u32 {\\n        self.length\\n    }\\n\" }) }), Aggregate(AstAggregate { [salsa id]: Id(2002), span: Span { start: Offset(564), end: Offset(593), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(564), end: Offset(570), anchor: \"...\" }, kind: Export }), kind: Struct, name: Identifier { text: \"Pointer\" }, name_span: Span { start: Offset(578), end: Offset(585), anchor: \"...\" }, generics: Some(SpanVec { span: Span { start: Offset(591), end: Offset(592), anchor: \"...\" }, values: [AstGenericDecl { [salsa id]: Id(2401), kind: Type(Span { start: Offset(586), end: Offset(590), anchor: \"...\" }), name: Some(SpannedIdentifier { span: Span { start: Offset(591), end: Offset(592), anchor: \"...\" }, id: Identifier { text: \"T\" } }) }] }), inputs: None, where_clauses: None, contents: None }), Function(AstFunction { [salsa id]: Id(2c01), span: Span { start: Offset(595), end: Offset(630), anchor: \"...\" }, effects: AstFunctionEffects { async_effect: Some(Span { start: Offset(602), end: Offset(607), anchor: \"...\" }), unsafe_effect: None }, fn_span: Span { start: Offset(608), end: Offset(610), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(595), end: Offset(601), anchor: \"...\" }, kind: Export }), name: SpannedIdentifier { span: Span { start: Offset(611), end: Offset(616), anchor: \"...\" }, id: Identifier { text: \"print\" } }, generics: None, inputs: SpanVec { span: Span { start: Offset(620), end: Offset(626), anchor: \"...\" }, values: [Variable(VariableDecl { [salsa id]: Id(1802), mutable: None, name: SpannedIdentifier { span: Span { start: Offset(617), end: Offset(618), anchor: \"...\" }, id: Identifier { text: \"s\" } }, perm: None, base_ty: AstTy { [salsa id]: Id(1405), span: Span { start: Offset(620), end: Offset(626), anchor: \"...\" }, kind: Named(AstPath { [salsa id]: Id(1007), kind: Identifier(SpannedIdentifier { span: Span { start: Offset(620), end: Offset(626), anchor: \"...\" }, id: Identifier { text: \"String\" } }) }, None) } })] }, output_ty: None, where_clauses: None, body: Some(DeferredParse { span: Span { start: Offset(628), end: Offset(630), anchor: \"...\" }, contents: \"\" }) })] } }), source: AstAggregate { [salsa id]: Id(2001), span: Span { start: Offset(0), end: Offset(562), anchor: \"...\" }, visibility: Some(AstVisibility { span: Span { start: Offset(0), end: Offset(6), anchor: \"...\" }, kind: Export }), kind: Class, name: Identifier { text: \"String\" }, name_span: Span { start: Offset(13), end: Offset(19), anchor: \"...\" }, generics: None, inputs: None, where_clauses: None, contents: Some(DeferredParse { span: Span { start: Offset(20), end: Offset(562), anchor: \"...\" }, contents: \"\\n    data: Pointer[u8]\\n    length: u32\\n    capacity: u32\\n\\n    ## Create a string from a statically allocated byte array.\\n    ## Used to create string literals.\\n    ##\\n    ## # Unsafe\\n    ##\\n    ## The data must be valid indefinitely.\\n    ## The resulting string will not free the data when it is dropped.\\n    export unsafe fn literal(data: Pointer[u8], length: u32) -> String {\\n        String { data: data, length: length, capacity: 0 }\\n    }\\n\\n    ## Get the length of the string.\\n    export fn len(self) -> u32 {\\n        self.length\\n    }\\n\" }) } }), []) }) }`, which doesn't appear to have a field or method `move`",
        },
    ],
    children: [],
}
```

