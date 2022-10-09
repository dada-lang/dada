# String literals

String literals in Dada support

- escape characters like `\n`
- interspersed expressions
- margin stripping

All of these can be disabled by using [raw string literals](./raw-string-literals).

## Escape characters

The `\` character is used to introduce an escape. The following escapes are recognized:

- `\n` -- newline
- `\r` -- carriage return
- `\t` -- tab
- `\\` -- literal `\`
- `\{` -- opening brace (otherwise interpreted as an interspersed expression)
- `\}` -- closing brace (otherwise interpreted as an interspersed expression)
- `\"` -- literal quote

More escapes may be added later.

## Interspersed expressions

Expressions can be included in the string literal by using `{}`. These expressions are evaluated, stringified, and then concatenated to form the final string.

## Margin stripping

If a string literal begins with an unescaped newline character, as shown here...

```
let example = "
   ...
"
```

...then the first and last newlines (if any) will be removed. In addition, for all remaining lines, the common whitespace prefix is computed and removed (ignoring lines that are entirely whitespace). For the purposes of computing margins, all interspersed expressions `{...}` are treated as if they were the text `{}`.

:::info
Note that the whitespace prefix must match exactly. For example, a string that uses tabs on one line and spaces on the next has no prefix.
:::

### Examples

```
let example = "
    Hello, world
"
```

This is equivalent to `"Hello, world"`

---

```
let example = "

    Hello, world

"
```

This is equivalent to "\nHello, world\n". Note that the margin of ` ` was removed from the middle line even though there were two empty lines before it that did not have the same whitespace prefix.

---

```
let example = "

    Hello,
      world

"
```

This is equivalent to "\nHello,\n world\n". Note that the margin of ` ` was removed from the middle line even though there were two empty lines before it that did not have the same whitespace prefix.

---

```
let example = "\n
    Hello, world
"
```

This is equivalent to "\n\n Hello, world\n". The initial `\n` disables all stripping, and so the remaining newlines and whitespace are included.

---

```
let example = "{""}
    Hello,
      world
"
```

This is equivalent to "\n Hello,\n world\n". The initial `{""}` disables all subsequent margin stripping.

---

```
let example = "Hello,
  world
"
```

This is equivalent to "Hello,\n world\n".

---

```
let example = "
  Hello, {"\nworld,\n"}
  how are you?
"
```

This is equivalent to `"Hello, \nworld,\n\nhow are you?"`. The interspersed `"\nworld,\n"` is reproduced exactly and does not affect the margin of `" "`.
