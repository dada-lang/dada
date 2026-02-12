# String Literals

This chapter specifies string literal syntax in Dada.

## Delimiters

:::{spec} rfc0001
There are multiple forms of string literals:

* {spec}`quoted` Single-quoted string literals begin with a `"` and end with a `"`.
* {spec}`triple-quoted` Triple-quoted string literals begin with a `"""` and end with a `"""`.
:::

:::{spec} disambiguation rfc0001
The syntax `"""` is interpreted as the start of a triple-quoted string literal
and not a single-quoted string literal followed by the start of another single-quoted string literal.
:::

:::{spec} triple-quote-termination rfc0001
A triple-quoted string literal cannot contain three consecutive unescaped double-quote characters.
:::

## Type

:::{spec} rfc0001
String literals have type `my String`.
:::

## Escape Sequences

:::{spec} rfc0001
String literals support the following escape sequences:

* {spec}`backslash` `\\` produces a literal backslash.
* {spec}`double-quote` `\"` produces a literal double quote.
* {spec}`newline` `\n` produces a newline.
* {spec}`carriage-return` `\r` produces a carriage return.
* {spec}`tab` `\t` produces a tab.
* {spec}`open-brace` `\{` produces a literal `{`.
* {spec}`close-brace` `\}` produces a literal `}`.
:::

:::{spec} triple-quoted rfc0001
The `\"` escape sequence is not needed in triple-quoted strings,
since embedded double quotes do not terminate the string.
:::

:::{spec} invalid rfc0001
A `\` followed by a character not listed above is an error.
:::

## Interpolation

:::{spec} rfc0001 unimpl
String literals may contain interpolation expressions
delimited by curly braces (`{` and `}`).
Any valid Dada expression may appear inside the braces.
:::

:::{spec} brace-escaping rfc0001
Literal brace characters are produced by the `\{` and `\}` escape sequences.
:::

:::{spec} nesting rfc0001 unimpl
The lexer tracks brace nesting depth,
so that braces within interpolated expressions (e.g., block expressions, struct literals)
do not prematurely terminate the interpolation.
:::

:::{spec} nested-quotes rfc0001 unimpl
Quotes inside interpolated expressions do not terminate the enclosing string literal.
:::

:::{spec} scope rfc0001 unimpl
Interpolated expressions are evaluated at runtime in the enclosing scope.
:::

:::{spec} order rfc0001 unimpl
Interpolated expressions are evaluated left-to-right.
:::

:::{spec} type-check rfc0001 unimpl
Each interpolated expression must produce a value that can be converted to a string.
This is checked at compile time.
:::

:::{spec} permissions rfc0001 unimpl
The permission system applies normally to interpolated expressions.
:::

## Multiline Strings

:::{spec} rfc0001 unimpl
A string literal that begins with a newline immediately after the opening quote
(either `"` or `"""`) is a multiline string literal
with automatic indentation handling.
:::

:::{spec} leading-newline rfc0001 unimpl
The leading newline immediately after the opening quote is removed.
:::

:::{spec} trailing-whitespace rfc0001 unimpl
The trailing newline immediately before the closing quote is removed,
along with any whitespace on the final line.
:::

:::{spec} dedenting rfc0001 unimpl
The common whitespace prefix across all non-empty lines is removed
from the start of each line.
:::

:::{spec} trailing-newline rfc0001 unimpl
A multiline string literal ending with `\n` before the closing quote
includes a trailing newline in the final string value.
:::

:::{spec} raw rfc0001 unimpl
A string literal beginning with `"\` followed by a newline
disables automatic dedenting.
The string preserves its content exactly as written,
including the leading newline and all indentation.
:::

## String Conversion

:::{spec} rfc0001 unimpl
Interpolated expressions must produce values that can be converted to strings.
The exact conversion mechanism is not yet defined
and depends on Dada's trait/interface system.
:::

## Implementation Notes

> A string literal with no interpolation expressions can be compiled
> as a simple string constant with no runtime overhead.
