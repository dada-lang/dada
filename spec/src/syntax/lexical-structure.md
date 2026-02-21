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
The lexer produces a sequence of tokens:

```ebnf
Token ::= ...
```

A token `Token` is one of the following kinds:

* {spec}`identifier-nt` An identifier `Identifier`.
* {spec}`keyword-nt` A keyword `Keyword`.
* {spec}`literal-nt` A literal `Literal` (integer, string, or boolean).
* {spec}`operator-nt` A single punctuation or operator character `Operator`.
* {spec}`delimiter-nt` A delimited group `Delimiter`: matched pair of brackets and their contents.
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

## `Identifier` definition

:::{spec}
An identifier `Identifier` begins with a Unicode alphabetic character or underscore (`_`),
followed by zero or more Unicode alphanumeric characters or underscores,
provided it is not a keyword `Keyword`:

```ebnf
Identifier ::= (Alphabetic | `_`) (Alphanumeric | `_`)*    (not a Keyword)
```
:::

:::{spec} case-sensitivity
Identifiers are case-sensitive.
:::

## `Keyword` definition

:::{spec}
The following words are reserved as keywords:

```ebnf
Keyword ::= ...
```

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

## `Operator` definition

:::{spec}
The following single characters are recognized as operator tokens:

```ebnf
Operator ::= `+` | `-` | `*` | `/` | `%` | `=` | `!`
           | `<` | `>` | `&` | `|` | `:` | `,` | `.` | `;` | `?`
```

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

## `Delimiter` definition

:::{spec}
A delimited token contains a matched pair of brackets and their contents:

```ebnf
Delimiter ::= `(` Token* `)` | `[` Token* `]` | `{` Token* `}`
```

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

## `Literal` definition

:::{spec}
A literal `Literal` is one of the following:

```ebnf
Literal ::= ...
```

* {spec}`integer-literal-nt` An integer literal `IntegerLiteral`.
* {spec}`boolean-literal-nt` A boolean literal `BooleanLiteral`.
* {spec}`string-literal-nt` A string literal `StringLiteral`.
:::

### `IntegerLiteral` definition

:::{spec}
An integer literal `IntegerLiteral` is a sequence of one or more ASCII decimal digits (`0`–`9`),
optionally separated by underscores (`_`) that do not affect the value:

```ebnf
IntegerLiteral ::= Digit (`_`? Digit)*
Digit ::= `0` | `1` | ... | `9`
```
:::

### `BooleanLiteral` definition

:::{spec}
The keywords `true` and `false` are boolean literals:

```ebnf
BooleanLiteral ::= `true` | `false`
```
:::

### `StringLiteral` definition

:::{spec}
String literal syntax is specified in [String Literals](string-literals.md).
:::

## Lexical Errors

:::{spec}
Characters that do not begin a valid token are accumulated
and reported as a single error spanning the invalid sequence.
:::
