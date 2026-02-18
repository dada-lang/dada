# Statements

This chapter specifies the statement syntax of Dada.

## `Block` definition

:::{spec}
A block `Block` is a sequence of zero or more statements
enclosed in curly braces:

```ebnf
Block ::= `{` Statement* `}`
```
:::

:::{spec} value
A block evaluates to the value of its last expression,
if the last statement is an expression statement.
:::

## `Statement` definition

:::{spec}
A statement `Statement` is one of the following:

```ebnf
Statement ::= ...
```

* {spec}`let-statement-nt` A let statement `LetStatement`.
* {spec}`expr-statement-nt` An expression statement `ExprStatement`.
:::

## `LetStatement` definition

:::{spec}
A let statement `LetStatement` introduces a new variable binding:

```ebnf
LetStatement ::= `let` `mut`? Identifier (`:` Type)? (`=` Expr)?
```
:::

:::{spec} type-annotation
A `let` statement may include a type annotation: `let name: Type = value`.
:::

:::{spec} mutable
A `let` statement may use `mut` to declare a mutable binding:
`let mut name = value`.
:::

:::{spec} initializer-optional
The initializer (`= value`) is optional.
A variable may be declared without an initial value.
:::

## `ExprStatement` definition

:::{spec}
An expression statement `ExprStatement` is an expression
followed by a newline or end of block:

```ebnf
ExprStatement ::= Expr
```
:::
