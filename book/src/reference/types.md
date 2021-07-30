# Types

Dada has two kinds of user-declared types:

* Classes, declared with `class`, which have identity;
* Data, declared with `data`, which is copied around freely.

## Types

The full syntax of dada types is as follows:

```
Type = Mode Type
     | NamedType
     | 'box' Type            // boxed version of type
     | '[' Type ']'          // shorthand for vectors
     | '{' Type ':' Type '}' // shorthand for maps
NamedType = Id Params
Params = '<' Param '>'
Param = 
```

## Declaring classes and data

Both classes and data are declared in a similar way:

```
TypeDecl = 'box'? ClassOrData Generics? Fields Supertypes?
ClassOrData = 'class' | 'data'
GenericDecls = '<' Generic* '>'
GenericDecl = Variance? 'type' Id
Variance = in | atomic | out
Fields = '(' Field* ')'
Field = Mutability? Id ':' Type
Mutability = var | atomic
Supertypes = ':' NamedType
```

Examples:

```
class Person(
    name: String
    age: uint
)

// A point
data Point2d(
    x: u32
    y: u32
)

// Modeling the `enum Option<T>` in Rust:
data Option<T>
data None<T>: Option<T>
data Some<T>(value: T): Option<T>
```
