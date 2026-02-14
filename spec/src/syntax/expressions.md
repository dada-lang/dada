# Expressions

This chapter specifies the expression syntax of Dada.

## Primary Expressions

### Identifiers

:::{spec}
An identifier used as an expression refers to a variable or item in scope.
:::

### Self

:::{spec}
The keyword `self` refers to the receiver of the current method.
:::

### Parenthesized Expressions

:::{spec}
An expression may be enclosed in parentheses: `(expr)`.
Parentheses control evaluation order without changing the value.
:::

### Block Expressions

:::{spec}
A block `{ statements }` is an expression.
See [Blocks](statements.md#blocks) for details.
:::

### Literal Expressions

:::{spec}
Integer literals, boolean literals, and string literals
are expressions.
See [Literals](literals.md) and [String Literals](string-literals.md) for details.
:::

## Operator Expressions

### Binary Operators

:::{spec}
Binary operators combine two expressions.
All binary operators are left-associative.
:::

:::{spec} precedence
Binary operators have the following precedence, from lowest to highest:

1. Assignment: `=`
2. Logical OR: `||`
3. Logical AND: `&&`
4. Comparison: `==`, `<`, `>`, `<=`, `>=`
5. Addition and subtraction: `+`, `-`
6. Multiplication and division: `*`, `/`
:::

:::{spec} assignment
The `=` operator assigns a value to a place expression.
:::

:::{spec} comparison
The comparison operators `==`, `<`, `>`, `<=`, `>=`
compare two values and produce a boolean result.
:::

:::{spec} arithmetic
The arithmetic operators `+`, `-`, `*`, `/`
perform arithmetic on numeric values.
:::

:::{spec} logical
The logical operators `&&` and `||`
perform short-circuit boolean logic.
:::

### Unary Operators

:::{spec}
The prefix operator `!` performs logical negation.
:::

:::{spec} negate
The prefix operator `-` performs arithmetic negation.
:::

### Newline Sensitivity

:::{spec}
A binary operator must appear on the same line as its left operand.
An operator on a new line begins a new expression or is interpreted as a prefix operator.
:::

## Postfix Expressions

### Field Access

:::{spec}
A field of a value is accessed with dot notation: `expr.field`.
:::

### Method Calls

:::{spec}
A method is called with dot notation followed by arguments:
`expr.method(args)`.
:::

### Function Calls

:::{spec}
A function is called by following an expression with parenthesized arguments:
`expr(args)`.
Arguments are separated by commas.
:::

### Await

:::{spec}
The `.await` postfix operator awaits a future: `expr.await`.
:::

### Permission Operations

:::{spec}
The following postfix operations request specific permissions on a value:

* {spec}`give` `.give` transfers ownership of the value.
* {spec}`share` `.share` creates a shared reference.
* {spec}`lease` `.mut` creates a mutable lease.
* {spec}`ref` `.ref` creates an immutable reference.
:::

## Control Flow

### If Expressions

:::{spec}
An `if` expression evaluates a condition and executes a block:

```dada
if condition {
    body
}
```
:::

:::{spec} else
An `if` expression may have an `else` clause:
`if condition { body } else { body }`.
:::

:::{spec} else-if
Multiple conditions may be chained with `else if`:
`if c1 { } else if c2 { } else { }`.
:::

### Return

:::{spec}
A `return` expression exits the enclosing function.
:::

:::{spec} return-value
`return` may be followed by a value: `return expr`.
:::

## Constructor Expressions

:::{spec}
A constructor expression creates a new instance of a class or struct:

```dada
TypeName(field1: value1, field2: value2)
```
:::
