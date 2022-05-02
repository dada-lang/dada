# Subleases

When you have a leased value, you can lease it again, creating a sublease. Here is an example where we create a lease `l1` and then a sublease `l2`. Try putting your cursor after `leased l2 = l1`, you will see that both `p` and `l1` are drawn with "dashes", indicating that those variables have leased our their object to another:

```
class Point(our x, our y)

my p = Point(22, 44)

# `l1` is leased from `p`
leased l1 = p

# `l2` is leased from `l1`
leased l2 = l1
#             ▲
# ────────────┘

# You see:
# ┌────┐
# │    │                  
# │ p  ├╌my╌╌╌╌╌╌╌╌╌╌╌╌╌╌►┌───────┐
# │    │                  │ Point │
# │ l1 ├╌leased╌╌╌╌╌╌╌╌╌╌►│ ───── │
# │    │                  │ x: 22 │
# │ l2 ├─leased──────────►│ y: 44 │
# │    │                  └───────┘
# └────┘                  
```

Subleases can be ended just like any other lease, except that a sublease can be terminated either by the lessor (`l1`, here) or by the original owner (`p`, here). Try inserting commands like `l1.x += 1` or `p.x += 1` and see how the diagram changes.

## Giving a leased value

When you [`give`](./my.md) a lease value, it results in a sublease. This preserves the rule for "give", that giving an object always creates a new value with equivalent permissions: a sublease permits all the same access to the object as the original lease.

```
class Point(our x, our y)
my p = Point(22, 44)
leased l1 = p
any l2 = l1.give           # subleases from `l1`
l2.x += 1                  # modifies the `Point`
```
