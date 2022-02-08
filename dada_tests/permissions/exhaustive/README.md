Naming pattern:

```
Name := Op "-var-" Field* "-" Input

Op := "give" // ends in `.give`
   |  "share" // ends in `.share`
   |  "lease" // ends in `.lease`

Field := "field" // access to a field declared like `class Pair(a)`

Input := "my" // indicates: Pair(22, 44) stored directly
      |  "our" // indicates: Pair(22, 44).share
      |  "leased" // indicates: Pair(22, 44).lease
      |  "lease-share" // indicates: Pair(22, 44).lease.share
```

e.g. the test `give-var-my.dada` is...

```
    p = Pair(22, 44)
#       ^^^^^^^^^^^^ -my

    q = p.give
#         ^^^^ give-
```


e.g. and the test `share-var-our.dada` is...

```
    p = Pair(22, 44).share
#                    ^^^^^^ -our

    q = p.share
#         ^^^^ share-
```