# Sketchy ideas

This page notes some "general plans" for how to extend Dada past the dynamic ownership core we are working on right now. Most of it is very rough. There's been a fair amount of work on the static type system, but it's rather out of date and needs to be synchronized.

## Dynamic dada: beyond ownership

### Methods

```
pub class Point(our x, our y) {
    pub fn manhattan_distance(self, other) -> {
        (other.x - self.x).absolute_value() + (other.y - self.y).absolute_value()
    }
}
```

### Closures

* syntax for a closure: `{ ... }`
* with arguments: `{ n -> ... }`

### Subclassing and pattern matching

Subclassing is permitted, but by default only within the same crate

```
class Option { }
class None: Option { }
class Some(my value): Option { }
```

then permits destructuring (in dynamic dada, no exhaustiveness checking)

```
match option {
    None => ...
    Some(x) => ... # x is owned
}
```

as in Rust you can use leasing

```
match option.lease {
    None => ...
    Some(x) => ... # x is leased
}
```

probably want some way to declare open classes (both to world and to future versions)

### Collections

```
my list = [1, 2, 3]
my map = [key: value, ...]
```

### Iterating over classes

```
for x in list { <takes ownership> }
for x in list.share { <shares the things within> }
for x in list.lease { <leases the things within> }
```

### Iterable and iterator

```
my iterator = list.share.iter()
x.next()
```

### nondefault constructors

```
class Foo(
    my x, 
    my y,
) {

}
```

### Privacy

oh god I don't know... pub/crate/priv?

I know I like the Rust idea of "private by default"

### modules and imports

oh god I don't know

but no pub use please, that makes life very hard

### FFI, Rust interop, crates.io

gotta figure this out :) 

but I want really smooth interop with wasm! that much I know.

## Checked dada

### types

```
my variable: Type = <value>
```

this is short for

```
any variable: my Type = <value>
```

a type is:

```
Type = Mode ClassName `<` Parameters `>`
Mode = `my`, `our`, `leased`, `our leased` ... need to track lessor
Parameter = Type, Lessor ... figure that stuff out
```

I'm not sure the exact notation for modes/parameters; I'd like to be able to do `from(x)` where `x` is a path to mean "this was given from `x`, so inherit its mode", that much I know. We probably need the equivalent of lifetime parameters but I'm not thrilled about it, and need a good name for them ("modes"? meh).

### generics

* probably use `<>`; declare a new generic with `type`

Idea is that we can do

```
struct Foo<type T> {

}
```

and then later

```
impl Foo<type T> {
    ...
}
```

vs

```
impl Foo<Integer> { 
    ...
}
```

is this nice? I don't know.

### variance

* generics are always covariant unless annotated with `=`, which makes them invariant
* to use a generic type within an atomic field, must be annotated with `=`
    * alternative: `atomic`
    * also within fn type parameter lists, if we have those
    * maybe we eventually want a contravariant notation, e.g. `in`

```
class Cell<type =T> {

}

our x: Cell<=int> = ....;
#           ^^^^ also here? Not sure.
```

