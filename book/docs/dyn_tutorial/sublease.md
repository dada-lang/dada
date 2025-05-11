# Subleases

When you have a mutable value, you can lease it again, creating a sublease. Here is an example where we create a lease `l1` and then a sublease `l2`. Try putting your cursor after `let l2: mutable = l1`, you will see that both `p` and `l1` are drawn with "dashes", indicating that those variables have mutable our their object to another:

```
class Point(x: our, y: our)

let p: my = Point(22, 44)

# `l1` is mutable from `p`
let l1: mutable = p

# `l2` is mutable from `l1`
let l2: mutable = l1
#                  ▲
# ─────────────────┘

# You see:
# ┌────┐
# │    │
# │ p  ├╌my╌╌╌╌╌╌╌╌╌╌╌╌╌╌►┌───────┐
# │    │                  │ Point │
# │ l1 ├╌mutable╌╌╌╌╌╌╌╌╌╌►│ ───── │
# │    │                  │ x: 22 │
# │ l2 ├─mutable──────────►│ y: 44 │
# │    │                  └───────┘
# └────┘
```

Subleases can be ended just like any other lease, except that a sublease can be terminated either by the lessor (`l1`, here) or by the original owner (`p`, here). Try inserting commands like `l1.x += 1` or `p.x += 1` and see how the diagram changes.

## Giving a mutable value

When you [`give`](./my.md) a lease value, it results in a sublease. This preserves the rule for "give", that giving an object always creates a new value with equivalent permissions: a sublease permits all the same access to the object as the original lease.

```
class Point(x: our, y: our)
let p: my = Point(22, 44)
let l1: mutable = p
let l2: any = l1.give           # subleases from `l1`
l2.x += 1                  # modifies the `Point`
```
