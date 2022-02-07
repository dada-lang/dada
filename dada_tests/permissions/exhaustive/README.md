Naming pattern:

```
Name := Op "-" Var "-" Field* "-" Input

Op := "give" // ends in `.give`
   |  "share" // ends in `.share`
   |  "lease" // ends in `.lease`

Place := "var" // access to a local variable declared like `var p =`
      |  "shared" // access to a local variable declared `p =`

Field := "field(var)" // access to a field declared like `class Pair(a)`
      |  "field(shared)" // access to a field declared like `class Pair(b)`

Input := "my" // indicates: Pair(22, 44) stored directly
      |  "our" // indicates: Pair(22, 44).share
      |  "leased" // indicates: Pair(22, 44).lease
      |  "lease-share" // indicates: Pair(22, 44).lease.share
```

e.g. the test `give-var-my.dada` is...

```
    var p = Pair(22, 44)
#   ^^^ var ^^^^^^^^^^^^ -my

    var q = p.give
#             ^^^^ give-
```


e.g. and the test `share-var-our.dada` is...

```
    var p = Pair(22, 44).share
#   ^^^ var             ^^^^^^ -our

    var q = p.share
#             ^^^^ share-
```