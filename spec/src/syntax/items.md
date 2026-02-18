# Items

This chapter specifies the top-level items that can appear in a Dada source file.

## Source Files

:::{spec}
A Dada source file defines a module.
The module name is derived from the file name.
A source file contains zero or more items,
optionally followed by zero or more statements:

```ebnf
SourceFile ::= Item* Statement*
```
:::

:::{spec} implicit-main
If a source file contains top-level statements,
they are wrapped in an implicit `async fn main()` function.
:::

:::{spec} kinds
An item `Item` is one of the following:

```ebnf
Item ::= ...
```

* {spec}`function-nt` A function `Function`.
* {spec}`class-nt` A class `Class`.
* {spec}`struct-nt` A struct `Struct`.
* {spec}`use-declaration-nt` A use declaration `UseDeclaration`.
:::

## `Visibility` definition

:::{spec}
Items and fields may have a visibility modifier.
Without a modifier, the item is private to the enclosing module.

```ebnf
Visibility ::= ... | ε
```

* {spec}`pub` `pub` makes the item visible within the crate.
* {spec}`export` `export` makes the item visible outside the crate.
:::

## `Function` definition

:::{spec}
A function `Function` is declared with the `fn` keyword,
optionally preceded by effect keywords
and followed by a name, optional generic parameters,
parameters, optional return type, optional where clause,
and a body or semicolon:

```ebnf
Function ::= Visibility Effect* `fn` Identifier GenericParameters?
             `(` Parameters `)` ReturnType? WhereClause? FunctionBody
```
:::

### `Effect` definition

:::{spec}
Effect keywords may appear in any order before `fn`:

```ebnf
Effect ::= ...
```

* {spec}`async` `async` declares an asynchronous function.
* {spec}`unsafe` `unsafe` declares an unsafe function.
:::

### `Parameters` definition

:::{spec}
Function parameters are enclosed in parentheses and separated by commas:

```ebnf
Parameters ::= FunctionInput,*
FunctionInput ::= SelfParameter | Parameter
```
:::

:::{spec} self
A function may have a `self` parameter as its first parameter,
optionally preceded by a permission keyword,
which makes it a method:

```ebnf
SelfParameter ::= PermissionKeyword? `self`
```
:::

:::{spec} parameter-syntax
Each non-self parameter has the form `name: Type`.
A parameter may be preceded by `mut`
to declare a mutable binding:

```ebnf
Parameter ::= `mut`? Identifier `:` Type
```
:::

### `FunctionBody` definition

:::{spec}
A function may have a body, which is a block enclosed in curly braces.
If no body is present, the function has no definition.

```ebnf
FunctionBody ::= Block | ε
```
:::

### `ReturnType` definition

:::{spec}
A function may declare a return type with `->` followed by a `Type` after the parameters.

ReturnType ::= `->` Type
:::

### `GenericParameters` definition

:::{spec}
A function may declare generic parameters in square brackets after the name:

```ebnf
GenericParameters ::= `[` GenericParameter,* `]`
GenericParameter ::= `type` Identifier | `perm` Identifier
```

* {spec}`type-parameters` A type parameter `type` followed by a name: `type T`.
* {spec}`permission-parameters` A permission parameter `perm` followed by a name: `perm P`.
:::

### `WhereClause` definition

:::{spec}
A function may have a `where` clause after the return type
that constrains its generic parameters:

```ebnf
WhereClause ::= `where` WhereConstraint,+
WhereConstraint ::= Type `is` WhereKind (`+` WhereKind)*
WhereKind ::= ...
```

* {spec}`ref` `ref`
* {spec}`mut` `mut`
* {spec}`shared` `shared`
* {spec}`unique` `unique`
* {spec}`owned` `owned`
* {spec}`lent` `lent`
:::

## `Class` definition

:::{spec}
A class `Class` is declared with the `class` keyword.
Classes have reference semantics.

```ebnf
Class ::= Visibility `class` Identifier GenericParameters?
          ConstructorFields? WhereClause? ClassBody?
```
:::

### `ConstructorFields` definition

:::{spec}
A class may declare constructor fields in parentheses after the name:

```ebnf
ConstructorFields ::= `(` Field,* `)`
```
:::

### `ClassBody` definition

:::{spec}
A class body enclosed in curly braces may contain field declarations and method definitions:

```ebnf
ClassBody ::= `{` ClassMember* `}`
ClassMember ::= ...
```

* {spec}`field-nt` A field declaration `Field`.
* {spec}`method-nt` A method `Method`.
:::

### `Method` definition

:::{spec}
A method `Method` is a function declared inside a class or struct body:

```ebnf
Method ::= Function
```
:::

### `Field` definition

:::{spec} field-syntax
A field declaration `Field` has the form:

```ebnf
Field ::= Visibility `mut`? Identifier `:` Type
```
:::

### Generics and Where Clauses

:::{spec}
Classes support generic parameters and where clauses
with the same syntax as functions.
:::

## `Struct` definition

:::{spec}
A struct `Struct` is declared with the `struct` keyword.
The syntax is identical to `Class` but structs have value semantics.

```ebnf
Struct ::= Visibility `struct` Identifier GenericParameters?
           ConstructorFields? WhereClause? ClassBody?
```
:::

## `UseDeclaration` definition

:::{spec}
A `use` declaration `UseDeclaration` imports a name from another crate,
optionally renaming it with `as`:

```ebnf
UseDeclaration ::= `use` Path (`as` Identifier)?
Path ::= Identifier (`.` Identifier)*
```
:::
