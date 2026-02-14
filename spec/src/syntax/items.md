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
Item ::= ... see list below ...
```

* {spec}`function` A function `Function`.
* {spec}`class` A class `Class`.
* {spec}`struct` A struct `Struct`.
* {spec}`use` A use declaration `UseDeclaration`.
:::

## Visibility

:::{spec}
Items and fields may have a visibility modifier.
Without a modifier, the item is private to the enclosing module.

```ebnf
Visibility ::= ... see list below ... | ε
```

* {spec}`pub` `pub` makes the item visible within the crate.
* {spec}`export` `export` makes the item visible outside the crate.
:::

## Functions

:::{spec}
A function `Function` is declared with the `fn` keyword,
optionally preceded by effect keywords
and followed by a name, optional generic parameters,
parameters, optional return type, optional where clause,
and a body or semicolon:

```ebnf
Function ::= Visibility Effect* `fn` Identifier GenericParameters?
             `(` Parameters `)` (`->` Type)? WhereClause? FunctionBody
FunctionBody ::= Block | ε
```
:::

### Effects

:::{spec}
Effect keywords may appear in any order before `fn`:

```ebnf
Effect ::= ... see list below ...
```

* {spec}`async` `async` declares an asynchronous function.
* {spec}`unsafe` `unsafe` declares an unsafe function.
:::

### Parameters

:::{spec}
Function parameters are enclosed in parentheses and separated by commas:

```ebnf
Parameters ::= SelfParameter? (`,` Parameter)* | Parameter (`,` Parameter)*
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

### Return Type

:::{spec}
A function may declare a return type with `->` followed by a `Type` after the parameters.
:::

### Generics

:::{spec}
A function may declare generic parameters in square brackets after the name:

```ebnf
GenericParameters ::= `[` GenericParameter (`,` GenericParameter)* `]`
GenericParameter ::= ... see list below ...
```

* {spec}`type-parameters` A type parameter `type` followed by a name: `type T`.
* {spec}`permission-parameters` A permission parameter `perm` followed by a name: `perm P`.
:::

### Where Clauses

:::{spec}
A function may have a `where` clause after the return type
that constrains its generic parameters:

```ebnf
WhereClause ::= `where` WhereConstraint (`,` WhereConstraint)*
WhereConstraint ::= Type `is` WhereKind (`+` WhereKind)*
```

```ebnf
WhereKind ::= ... see list below ...
```

The constraint kinds `WhereKind` are:

* {spec}`ref` `ref`
* {spec}`mut` `mut`
* {spec}`shared` `shared`
* {spec}`unique` `unique`
* {spec}`owned` `owned`
* {spec}`lent` `lent`
:::

## Classes

:::{spec}
A class `Class` is declared with the `class` keyword.
Classes have reference semantics.

```ebnf
Class ::= Visibility `class` Identifier GenericParameters?
          ConstructorFields? WhereClause? ClassBody?
```
:::

### Constructor Fields

:::{spec}
A class may declare constructor fields in parentheses after the name:

```ebnf
ConstructorFields ::= `(` Field (`,` Field)* `)`
```
:::

### Members

:::{spec}
A class body enclosed in curly braces may contain field declarations and method definitions:

```ebnf
ClassBody ::= `{` ClassMember* `}`
ClassMember ::= ... see list below ...
```

* {spec}`field-member` A field declaration `Field`.
* {spec}`method-member` A method `Function`.
:::

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

## Structs

:::{spec}
A struct `Struct` is declared with the `struct` keyword.
The syntax is identical to `Class` but structs have value semantics.

```ebnf
Struct ::= Visibility `struct` Identifier GenericParameters?
           ConstructorFields? WhereClause? ClassBody?
```
:::

## Use Declarations

:::{spec}
A `use` declaration `UseDeclaration` imports a name from another crate,
optionally renaming it with `as`:

```ebnf
UseDeclaration ::= `use` Path (`as` Identifier)?
Path ::= Identifier (`.` Identifier)*
```
:::
