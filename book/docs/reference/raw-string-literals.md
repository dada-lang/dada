# Raw string literals

:::info
Raw string literals are not yet implemented. See [#179](https://github.com/dada-lang/dada/issues/179) for the current status.
:::

A raw string literal `r"..."` is a string literal with no escape sequences. All characters within the raw string literal are added into the final string until the terminator. Raw string literals can also include any number of `#` characters to permit embedding `"`.

## Examples

```
x = r"fo\o{}" 
```

yields the [string literal](./string-literals) `"fo\\o\{\}"`.


```
x = r#"fo\o{}"bar"# 
```

yields the [string literal](./string-literals) `"fo\\o\{\}\"bar"`.

