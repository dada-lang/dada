# Types and Permissions

This chapter specifies the syntax for types and permissions in Dada.

## Types

### Named Types

:::{spec}
A type may be a simple name: `String`, `i32`, `bool`.
:::

:::{spec} paths
A type may be a dotted path: `module.Type`.
:::

### Generic Application

:::{spec}
A type may be applied to generic arguments in square brackets:
`Vec[String]`, `Pair[i32, bool]`.
:::

### Permission-Qualified Types

:::{spec}
A type may be preceded by a permission to form a permission-qualified type:
`my String`, `ref Point`, `mut Vec[i32]`.
:::

## Permissions

:::{spec}
The following permission keywords are available:

* {spec}`my` `my` — exclusive ownership.
* {spec}`our` `our` — shared ownership.
* {spec}`ref` `ref` — immutable reference.
* {spec}`mut` `mut` — mutable reference.
* {spec}`given` `given` — a permission supplied by the caller.
:::

### Place Lists

:::{spec}
The permissions `ref`, `mut`, and `given` may include a place list
in square brackets specifying which places they refer to:
`ref[x, y]`, `mut[self]`, `given[p]`.
:::

:::{spec} place-list-optional
The place list is optional.
When omitted, the permission applies without place restrictions.
:::

## Generic Declarations

### In Type Position

:::{spec}
A generic type parameter is declared as `type T`.
:::

:::{spec} permission-declaration
A generic permission parameter is declared as `perm P`.
:::

### Ambiguity

:::{spec}
A single identifier in a generic position is ambiguous
between a type and a permission.
The ambiguity is resolved during type checking, not parsing.
:::
