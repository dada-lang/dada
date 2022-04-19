# Permissions

Every reference to a Dada object has an associated **permission**. Each permission has its own unique identity. In the interpreter, permissions are actual values that are allocated and referenced; this allows us to check for permission violations. In compiled Dada, the type system ensures there are no permission violations, so they can be erased and carry no runtime cost.

## Kinds of permissions

Dada has four kinds of permissions, and they can be categorized along two dimensions:

|        | Exclusive | Shared   |
| ------ | --------- | -------- |
| Owned  | my        | our      |
| Leased | leased    | shleased |

**Exclusive permissions** give exclusive access to the object, meaning that while that permission is active, no other permission gives access to the same data. They give full ability to mutate. Exclusive permissions cannot be duplicated, but they can be *leased*, as will be discussed shortly. **Shared permissions** do not guarantee exclusive access. They can be freely duplicated.

**Owned permissions** are permissions that independently guarantee that data is live and will not be collected. **Leased** permissions are tied to some owned permission (called the *lessor*, see the section on the permission forest).

## Owning and leasing: the permission forest

Permissions are structured into a forest to represent leasing relationships. When one permission `p` is leased to create another permission `q`, that makes `p` a parent of `q` in the forest. We refer to parent-child relationships in this forest as lessor-tenant: i.e., `p` is the **lessor** of `q` and `q` is the **tenant** of `p`.

### Cancelling a tenancy

Leased permissions are not permanent. They last until the lessor permission is used in some incompatible way. Using the lessor permission causes it to reassert its access to the object, cancelling the tenant permission. Once a tenant permission is cancelled, any further attempt to use it will create an error (in the interpreter, this is a runtime error; in the compiler, this is a detected by the type system and rejected).

The following sorts of actions cause cancellation:

* Reads: Reading with a permission `p` cancels any exclusive tenants of `p` (if `p` is a shared permission, though, it cannot have any exclusive tenants).
* Writes: Writing with a permission `p` cancels all tenants of `p` (only possible if `p` is an exclusive permission).
* Drops or giving: Dropping an object with permission `p` cancels all tenants of `p`, as does giving `p` (only possible if `p` is exclusively owned).

The final point is interesting: we can have an `our` object (shared ownership) that is leased, and that lease can be cancelled:

```
class Point()

fn test() -> {
    any p = Point().share        # our Point
    any q = p.lease              # shleased Point
    q                            # returning to the caller drops `p`, cancelling `q`
}

fn main() {
    t = test()               # error, this `our leased Point` has been cancelled
}
```

## Mutation of fields

The ability to mutate a field is dependent on the kind of permission that you have and whether or not the field is declared `atomic`:

|                      | ordinary field | atomic field                 |
| -------------------- | -------------- | ---------------------------- |
| shared permission    | immutable      | mutable in an atomic section |
| exclusive permission | mutable        | mutable                      |



## Actions on a permission

There are three actions one can take on a permission:

* `give`: Giving a permission `p` creates another permission `q` with equal capabilities. It does this in the "least disturbing" way possible:
    * If `p` is a shared permission (`our`, `our leased`), it is duplicated and so `q = p`.
    * If `p` is a `leased` permission, then a sublease `q` is created with `p` as the lessor.
    * For a `my` permission `p`, the older permission is revoked.
* `lease`: Leasing a permission `p` creates another permission `q` with equal access, but which can be revoked if `p` is used again:
    * If `p` is an exclusive permission (`my`, `my leased`), then `q` becomes a `my leased` permission. `q` is revoked if `p` is used again in any way (or if `p` is revoked).
    * If `p` is a shared permission (`our`, `our leased`), then `q` becomes an `our leased` permission. `q` is revoked if `p` is revoked (`p` cannot be written to).
* `share`: Sharing a permission `p` converts it into shared mode and then duplicates it:
    * If `p` is already a joint permission, it simply duplicates `p`, so `p = q`.
    * If `p` is a `leased` permission, it creates a shared tenant `q` of `p`.
        * If you write to `p` again, then `q` is revoked; reads from `p` have no effect.
    * If `p` is a `my` permission, it converts `p` into a shared permission (`our`) and then duplicates it.
        * You can continue reading from `p`, but attempting to write to it will yield an error.

Effectively:

* `give` creates a permission that occupies the same cell in the table.
* `lease` moves one row down.
* `share` moves one column to the right.

## Giving a `my` permission away

When you have a `my` permission `p`, that permission is relatively "fragile". The only way to create a second permission with access to the same object is to lease from `p`. The other actions, giving and sharing, both modify `p` to create the new permission `q` that now has access to the same object: give will cancel `p`, sharing converts `p` into shared ownership. In both cases, `p` is surrendering some of its permissions to create the new permission. This is only possible if you are the owner of the `p` permission. This is in contrast to leased permissions, discussed in the next section, which can be cancelled by their lessor permissions.
