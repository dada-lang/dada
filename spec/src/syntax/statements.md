# Statements

This chapter specifies the statement syntax of Dada.

## Blocks

:::{spec}
A block is a sequence of zero or more statements
enclosed in curly braces: `{ statements }`.
:::

:::{spec} value
A block evaluates to the value of its last expression,
if the last statement is an expression without a trailing semicolon.
:::

## Let Statements

:::{spec}
A `let` statement introduces a new variable binding:

```dada
let name = value
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

## Expression Statements

:::{spec}
An expression followed by a newline or end of block
is an expression statement.
:::
