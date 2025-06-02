# Specification Draft

*This file contains draft specification text for RFC-0001: String Literals*

## String Literals

r[syntax.string-literals.type]
String literals have type `my String`.

r[syntax.string-literals.double-quoted]
String literals can be delimited by double quotes, beginning with `"` and ending with `"`. Note that `"""` is interpreted as the start of a triple-quoted string and not an empty string followed by a `"` character.

r[syntax.string-literals.triple-quoted]
String literals can be delimited by triple quotes, beginning with `"""` and ending with `"""`. Embedded double quotes do not need escaping.

r[syntax.string-literals.interpolation]
String literals may contain interpolation expressions within curly braces (`{expression}`).

r[syntax.string-literals.lexical-analysis]
The lexer recognizes string literals with interpolation and treats characters inside `{}` as part of the interpolated expression, not the string literal. Quotes inside interpolated expressions do not terminate the string literal.

r[syntax.string-literals.escaping]
Literal braces are escaped by doubling: `{{` produces `{` and `}}` produces `}`.

r[syntax.string-literals.evaluation]
Interpolated expressions are evaluated at runtime in the current scope, converted to strings, and evaluated left-to-right.

r[syntax.string-literals.multiline]
A string literal that begins with a newline immediately after the opening quote is a multiline string literal with automatic indentation handling.

r[syntax.string-literals.multiline.dedenting]
For multiline string literals where each line is either empty or has a consistent whitespace prefix:
- Leading and trailing whitespace is trimmed
- The common whitespace prefix is removed from the start of each line

r[syntax.string-literals.multiline.raw]
A multiline string literal that begins with `"\` followed by a newline preserves the string exactly as written, including the leading newline and all indentation.

r[syntax.string-literals.multiline.trailing-newline]
A multiline string literal ending with `\n` before the closing quote includes a trailing newline in the final string value.

*More detailed specification text to be developed during implementation*