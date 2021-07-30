# Leases

A *lease* is a combination of a place like `a.b.c` and a set of "terms", which can be `shared` or `borrowed`. When you have a [leased value](./modes.md), its mode contains a set of leases that indicate where it came from. Typically you don't have to write the "terms" of the lease explicitly. Easiest to explain by example:

```
let x: my String = "foo";
let y = 
```

## Subleasing

