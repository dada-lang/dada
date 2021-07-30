# Types

Dada has two kinds of user-declared types:

* Classes, declared with `class`, which have identity;
* Data, declared with `data`, which is copied around freely.

## Types

The full syntax of dada types is as follows:

```
Type = Mode Type
     | Id Params
     | 'box' Type            // boxed version of type
     | '[' Type ']'          // shorthand for vectors
     | '{' Type ':' Type '}' // shorthand for maps
Params = '<' Param '>'
Param = 
```

## Declaring classes and data

Both classes and data are declared in a similar way:

```
TypeDecl = ClassOrData Generics? Fields
ClassOrData = 'class' | 'data'
GenericDecls = '<' Generic* '>'
GenericDecl = Variance? 'type' Id
Variance = in | atomic | out
Fields = '(' Field* ')'
Field = Mutability? Id ':' Type
Mutability = var | atomic
```

```
class MyClass /* (
    /* fields */
)
```


```
class MyClass /* (
    /* fields */
)
```

## Data
