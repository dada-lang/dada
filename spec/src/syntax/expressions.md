# Expressions

This chapter specifies the expression syntax of Dada.

## `Expr` definition

:::{spec}
An expression `Expr` is parsed using precedence climbing.
From lowest to highest precedence:

```ebnf
Expr ::= ...
```

* {spec}`assign-expr-nt` An assignment expression `AssignExpr`.
:::

## `AssignExpr` definition

:::{spec}
The assignment operator `=` assigns a value to a place expression.
It has the lowest precedence among binary operators:

```ebnf
AssignExpr ::= ...
```

* {spec}`or-expr-nt` A logical OR expression `OrExpr` (`=` `OrExpr`)?
:::

## `OrExpr` definition

:::{spec}
The logical OR operator `||` performs short-circuit boolean logic:

```ebnf
OrExpr ::= ...
```

* {spec}`and-expr-nt` An AND expression `AndExpr` (`||` `AndExpr`)*
:::

## `AndExpr` definition

:::{spec}
The logical AND operator `&&` performs short-circuit boolean logic:

```ebnf
AndExpr ::= ...
```

* {spec}`compare-expr-nt` A comparison expression `CompareExpr` (`&&` `CompareExpr`)*
:::

## `CompareExpr` definition

:::{spec}
The comparison operators compare two values and produce a boolean result:

```ebnf
CompareExpr ::= ...
```

* {spec}`add-expr-nt` An additive expression `AddExpr` (`CompareOp` `AddExpr`)*

```ebnf
CompareOp ::= `==` | `<` | `>` | `<=` | `>=`
```
:::

## `AddExpr` definition

:::{spec}
The additive operators perform addition and subtraction:

```ebnf
AddExpr ::= ...
```

* {spec}`mul-expr-nt` A multiplicative expression `MulExpr` ((`+` | `-`) `MulExpr`)*
:::

## `MulExpr` definition

:::{spec}
The multiplicative operators perform multiplication and division:

```ebnf
MulExpr ::= ...
```

* {spec}`unary-expr-nt` A unary expression `UnaryExpr` ((`*` | `/`) `UnaryExpr`)*
:::

## `UnaryExpr` definition

:::{spec}
A unary expression applies a prefix operator to a postfix expression:

```ebnf
UnaryExpr ::= UnaryOp* PostfixExpr
UnaryOp ::= `!` | `-`
```

* {spec}`not` `!` performs logical negation.
* {spec}`negate` `-` performs arithmetic negation.
:::

## Newline Sensitivity

:::{spec}
A binary operator must appear on the same line as its left operand.
An operator on a new line begins a new expression or is interpreted as a prefix operator.
:::

## `PostfixExpr` definition

:::{spec}
A postfix expression applies zero or more postfix operators
to a primary expression:

```ebnf
PostfixExpr ::= PrimaryExpr PostfixOp*
```
:::

### `PostfixOp` definition

:::{spec}
A postfix operator `PostfixOp` is one of the following:

```ebnf
PostfixOp ::= ...
```

* {spec}`field-access-nt` A field access `FieldAccess`.
* {spec}`call-nt` A function or method call `Call`.
* {spec}`await-nt` An await expression `Await`.
* {spec}`permission-op-nt` A permission operation `PermissionOp`.
:::

### `FieldAccess` definition

:::{spec}
A field access `FieldAccess` uses dot notation to access a field or name a method:

```ebnf
FieldAccess ::= `.` Identifier
```
:::

### `Call` definition

:::{spec}
A function or method call `Call` follows an expression with parenthesized arguments
separated by commas.
The opening parenthesis must appear on the same line as the callee:

```ebnf
Call ::= `(` Expr,* `)`
```
:::

### `Await` definition

:::{spec}
The `.await` postfix operator awaits the result of a future:

```ebnf
Await ::= `.` `await`
```
:::

### `PermissionOp` definition

:::{spec}
A permission operation `PermissionOp` requests specific permissions on a value:

```ebnf
PermissionOp ::= ...
```

* {spec}`give` `.` `give` transfers ownership of the value.
* {spec}`share` `.` `share` creates a shared reference.
* {spec}`lease` `.` `mut` creates a mutable lease.
* {spec}`ref` `.` `ref` creates an immutable reference.
:::

## `PrimaryExpr` definition

:::{spec}
A primary expression `PrimaryExpr` is one of the following:

```ebnf
PrimaryExpr ::= ...
```

* {spec}`literal-nt` A literal `Literal`.
* {spec}`identifier` An identifier `Identifier` referring to a variable or item in scope.
* {spec}`self` The keyword `self`, referring to the receiver of the current method.
* {spec}`if-expr-nt` An if expression `IfExpr`.
* {spec}`return-expr-nt` A return expression `ReturnExpr`.
* {spec}`constructor-expr-nt` A constructor expression `ConstructorExpr`.
* {spec}`paren-expr` A parenthesized expression `(` Expr `)`.
* {spec}`block-expr` A block expression `Block`.
:::

### `IfExpr` definition

:::{spec}
An if expression `IfExpr` evaluates a condition and executes a block:

```ebnf
IfExpr ::= `if` Expr Block (`else` `if` Expr Block)* (`else` Block)?
```
:::

:::{spec} else
An `if` expression may have an `else` clause.
:::

:::{spec} else-if
Multiple conditions may be chained with `else if`.
:::

### `ReturnExpr` definition

:::{spec}
A return expression `ReturnExpr` exits the enclosing function,
optionally with a value.
The value, if present, must appear on the same line as `return`:

```ebnf
ReturnExpr ::= `return` Expr?
```
:::

### `ConstructorExpr` definition

:::{spec}
A constructor expression `ConstructorExpr` creates a new instance
of a class or struct.
The opening brace must appear on the same line as the type name:

```ebnf
ConstructorExpr ::= Identifier `{` ConstructorField,* `}`
ConstructorField ::= Identifier `:` Expr
```
:::
