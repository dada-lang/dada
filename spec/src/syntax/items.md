# Items

This chapter specifies the top-level items that can appear in a Dada source file.

## Source Files

:::{spec}
A Dada source file defines a module.
The module name is derived from the file name.
:::

:::{spec} contents
A source file contains zero or more items,
optionally followed by zero or more statements.
:::

:::{spec} implicit-main
If a source file contains top-level statements,
they are wrapped in an implicit `async fn main()` function.
:::

## Visibility

:::{spec}
Items and fields may have a visibility modifier:

* {spec}`pub` `pub` makes the item visible within the crate.
* {spec}`export` `export` makes the item visible outside the crate.
:::

:::{spec} default
Items without a visibility modifier are private to the enclosing module.
:::

## Functions

:::{spec}
A function is declared with the `fn` keyword:

```dada
fn name(parameters) -> ReturnType {
    body
}
```
:::

### Effects

:::{spec}
A function may be preceded by effect keywords:

* {spec}`async` `async` declares an asynchronous function.
* {spec}`unsafe` `unsafe` declares an unsafe function.
:::

:::{spec} effect-combination
Effect keywords may be combined and appear in any order before `fn`.
:::

### Parameters

:::{spec}
Function parameters are enclosed in parentheses and separated by commas.
:::

:::{spec} self
A function may have a `self` parameter,
which makes it a method.
:::

:::{spec} self-permission
The `self` parameter may be preceded by a permission keyword:
`ref self`, `mut self`, `my self`, `our self`, or `given self`.
:::

:::{spec} parameter-syntax
Each non-self parameter has the form `name: Type`
where `Type` may include a permission prefix.
:::

:::{spec} mutable-parameters
A parameter may be preceded by `mut`
to declare a mutable binding: `mut name: Type`.
:::

### Return Type

:::{spec}
A function may declare a return type with `-> Type` after the parameters.
:::

### Generics

:::{spec}
A function may declare generic parameters in square brackets after the name:
`fn name[type T, perm P](...)`.
:::

:::{spec} type-parameters
A type parameter is declared with `type` followed by a name: `type T`.
:::

:::{spec} permission-parameters
A permission parameter is declared with `perm` followed by a name: `perm P`.
:::

### Where Clauses

:::{spec}
A function may have a `where` clause after the return type
that constrains its generic parameters.
:::

:::{spec} where-syntax
A where clause consists of `where` followed by one or more comma-separated constraints
of the form `Subject is Kind`.
:::

:::{spec} where-kinds
The constraint kinds are:

* {spec}`ref` `ref`
* {spec}`mut` `mut`
* {spec}`shared` `shared`
* {spec}`unique` `unique`
* {spec}`owned` `owned`
* {spec}`lent` `lent`
:::

:::{spec} where-combination
Multiple kinds may be combined with `+`: `where T is shared + owned`.
:::

### Body

:::{spec}
A function body is a block enclosed in curly braces.
:::

:::{spec} body-optional
A function may be declared without a body.
:::

## Classes

:::{spec}
A class is declared with the `class` keyword:

```dada
class Name(fields) {
    members
}
```
:::

:::{spec} reference-semantics
Classes have reference semantics.
:::

### Constructor Fields

:::{spec}
A class may declare constructor fields in parentheses after the name.
Each field has the form `[visibility] [mut] name: Type`.
:::

### Members

:::{spec}
A class body enclosed in curly braces may contain field declarations and method definitions.
:::

:::{spec} field-syntax
A field declaration has the form `[visibility] [mut] name: Type`.
:::

:::{spec} method-syntax
A method is a function declaration within the class body.
Methods typically take `self` as their first parameter.
:::

### Generics and Where Clauses

:::{spec}
Classes support generic parameters and where clauses
with the same syntax as functions.
:::

## Structs

:::{spec}
A struct is declared with the `struct` keyword.
The syntax is identical to `class`.
:::

:::{spec} value-semantics
Structs have value semantics.
:::

## Use Declarations

:::{spec}
A `use` declaration imports a name from another crate:

```dada
use crate_name.path
```
:::

:::{spec} renaming
A `use` declaration may rename the imported item with `as`:
`use crate_name.path as new_name`.
:::
