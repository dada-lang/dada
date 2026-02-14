# Lexical Structure

This chapter specifies the lexical structure of Dada programs.
A Dada source file is a sequence of Unicode characters,
which the lexer converts into a sequence of tokens.

## Source Encoding

:::{spec}
Dada source files are encoded as UTF-8.
:::

## Tokens

:::{spec}
The lexer produces a sequence of tokens. A token is one of the following kinds:

* {spec}`identifier` An identifier.
* {spec}`keyword` A keyword.
* {spec}`literal` A literal (integer, string, or boolean).
* {spec}`op-char` A single punctuation or operator character.
* {spec}`delimited` A delimited group: matched pair of brackets and their contents.
:::

:::{spec} preceding-whitespace
Each token records whether it was preceded by whitespace, a newline, or a comment.
This information is used by the parser but does not produce separate tokens.
:::

## Whitespace and Comments

### Whitespace

:::{spec}
Whitespace characters (spaces, tabs, and other Unicode whitespace excluding newlines)
separate tokens but are otherwise not significant.
:::

:::{spec} newlines
Newline characters (`\n`) are tracked by the lexer.
Whether a token is preceded by a newline
may affect how the parser interprets certain constructs.
:::

### Comments

:::{spec}
A comment begins with `#` and extends to the end of the line.
:::

:::{spec} content
The content of a comment, including the leading `#`, is ignored by the lexer.
A comment implies a newline for the purpose of preceding-whitespace tracking.
:::

:::{spec} no-block-comments
Dada does not have block comments.
:::

## Identifiers

:::{spec}
An identifier begins with a Unicode alphabetic character or underscore (`_`),
followed by zero or more Unicode alphanumeric characters or underscores.
:::

:::{spec} case-sensitivity
Identifiers are case-sensitive.
:::

:::{spec} keyword-recognition
A word that matches a keyword is always lexed as a keyword token,
not as an identifier.
:::

## Keywords

:::{spec}
The following words are reserved as keywords:

* {spec}`as` `as`
* {spec}`async` `async`
* {spec}`await` `await`
* {spec}`class` `class`
* {spec}`else` `else`
* {spec}`enum` `enum`
* {spec}`export` `export`
* {spec}`false` `false`
* {spec}`fn` `fn`
* {spec}`give` `give`
* {spec}`given` `given`
* {spec}`if` `if`
* {spec}`is` `is`
* {spec}`let` `let`
* {spec}`match` `match`
* {spec}`mod` `mod`
* {spec}`mut` `mut`
* {spec}`my` `my`
* {spec}`our` `our`
* {spec}`perm` `perm`
* {spec}`pub` `pub`
* {spec}`ref` `ref`
* {spec}`return` `return`
* {spec}`self` `self`
* {spec}`share` `share`
* {spec}`shared` `shared`
* {spec}`struct` `struct`
* {spec}`true` `true`
* {spec}`type` `type`
* {spec}`unsafe` `unsafe`
* {spec}`use` `use`
* {spec}`where` `where`
:::

## Punctuation and Operators

:::{spec}
The following single characters are recognized as operator tokens:

* {spec}`plus` `+`
* {spec}`minus` `-`
* {spec}`star` `*`
* {spec}`slash` `/`
* {spec}`percent` `%`
* {spec}`equals` `=`
* {spec}`bang` `!`
* {spec}`less-than` `<`
* {spec}`greater-than` `>`
* {spec}`ampersand` `&`
* {spec}`pipe` `|`
* {spec}`colon` `:`
* {spec}`comma` `,`
* {spec}`dot` `.`
* {spec}`semicolon` `;`
* {spec}`question` `?`
:::

:::{spec} multi-character
Multi-character operators such as `&&`, `||`, `==`, `<=`, `>=`, and `->`
are formed by the parser from adjacent operator tokens.
:::

## Delimiters

:::{spec}
A delimited token contains a matched pair of brackets and their contents:

* {spec}`parentheses` Parentheses: `(` and `)`.
* {spec}`square-brackets` Square brackets: `[` and `]`.
* {spec}`curly-braces` Curly braces: `{` and `}`.
:::

:::{spec} balanced
Delimiters must be balanced.
An opening delimiter without a matching closing delimiter is an error.
:::

:::{spec} nesting
The lexer tracks delimiter nesting.
Content between matching delimiters is treated as a unit,
which enables deferred parsing of function bodies and other nested structures.
:::

## Literals

### Integer Literals

:::{spec}
An integer literal is a sequence of one or more ASCII decimal digits (`0`–`9`).
:::

:::{spec} underscores
Underscores (`_`) may appear between digits as visual separators.
They do not affect the value of the literal.
:::

### Boolean Literals

:::{spec}
The keywords `true` and `false` are boolean literals.
:::

### String Literals

:::{spec}
String literal syntax is specified in [String Literals](string-literals.md).
:::

## Lexical Errors

:::{spec}
Characters that do not begin a valid token are accumulated
and reported as a single error spanning the invalid sequence.
:::
