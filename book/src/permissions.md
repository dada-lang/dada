# Permissions

Every reference to a Dada object has an associated **permision**. Each permission has its own unique identity. In the interpreter, permissions are actual values that are allocated and referenced; this allows us to check for permission violations. In compiled Dada, the type system ensures there are no permission violations, so they can be erased and carry no runtime cost.

## Kinds of permissions

Dada has four kinds of permissions, and they can be categorized along two dimensions:

|        | Exclusive | Joint  |
| ------ | --------- | ------ |
| Owned  | my        | our    |
| Leased | leased    | shared |

**Exclusive permissions** give exclusive access to the object, meaning that while that permission is active, no other permission gives access to the same data. They give full ability to mutate. Exclusive permissions cannot be duplicated, but they can be *leased*, as will be discussed shortly. **Joint permissions** do not guarantee exclusive access. They can be freely duplicated.

**Owned permissions** are permissions that independently guarantee that data is live and will not be collected. **Leased** permissions are tied to some owned permission (called the *lessor*, see the section on the permission forest).

## Owning and leasing: the permission forest

Permissions are structured into a forest to repesent leasing relationships. When one permission `p` is leased to create another permission `q`, that makes `p` a parent of `q` in the forest. We refer to parent-child relationships in this forest as lessor-tenant: i.e., `p` is the **lessor** of `q` and `q` is the **tenant** of `p`.

## Mutation of fields

The ability to mutate a field is dependent on the kind of permission that you have:

|                      | shared    | var       | atomic                       |
| -------------------- | --------- | --------- | ---------------------------- |
| joint permission     | immutable | immutable | mutable in an atomic section |
| exclusive permission | immutable | mutable   | mutable                      |

## Actions on a permission

There are three actions one can take on a permission:

* `give`: Giving a permission `p` creates another permission `q` with equal capabilities. It does this in the "least disturbing" way possible:
    * If `p` is a joint permission (`our`, `shared`), it is duplicated and so `q = p`.
    * If `p` is a `leased` permission, then a sublease `q` is created with `p` as the lessor.
    * For a `my` permission `p`, the older permission is revoked.
* `lease`: Leasing a permission `p` creates another permission `q` with equal ability to read/write but whose access can be revoked if `p` is used again:
    * If `p` is a joint permission, then there is no action that can be taken on `p` that would revoke `q`, so we simply duplicate `p`.
        * e.g., leasing an `our` permission creates another `our` permission
        * FIXME: *It might make sense to have leasing an `our` permission create a `shared` permission, in which case there would be an action that could invalidate: dropping `p`. Think about it.* 
* `give-share`: Give-sharing a permission `p` creates a joint permission `q`:
    * If `p` is already a joint permission, it simply duplicates `p`, so `p = q`.
    * If `p` is a `leased` permission, it creates a `shared` tenant `q` of `p`.
    * If `p` is a `my` permision, it cancels `p` and creates an `our` permission `q`.

Effectively:

* `give` creates a permission that occupies the same cell in the table.
* `lease` moves one row down.
* `give-share` moves one column to the right.

**Important.** The name `give-share` probably seems strange. This is because the `share` keyword in Dada doesn't map directly to `give-share`. When you apply the `share` keyword to a place expression (like a variable or a field)...

```
var a = Point(22, 44)
var b = p.share
```

...this actually compiles to two operations: first, a lease of `a` to create a `leased` permission `p`, and then a `give-share` of `p` to create a shared permission `q`. In other words, `p.share` is the equivalent of `p.lease.give-share` (if `give-share` were a keyword).

If however you apply the `share` keyword to a value expression (one that creates a value, like a function call), as in these examples...

```
var a = Point(22, 44).share
var b = a.give.share
```

...this is compiled to a `give-share` operation directly.

## Giving a `my` permission away

When you have a `my` permission `p`, that permisson is relatively "fragile". The only way to create a second permission with access to the same object is to lease from `p`. The other actions, giving and give-sharing, both cancel `p` to create the new permission `q` that now has access to the same object. The important point to note here is that this cancellation is actually more like *giving* -- the permission `p` is surrendering its permissions to create new ones. This is only possible if you are the owner of the `p` permission. This is in contrast to leased permissions, discussed in the next section, which can be cancelled by their lessor permissions.

## Cancelling a tenancy

Leased permissions are not permanent. They last until the lessor permission is used in some incompatible way. Using the lessor permission causes it to reassert its access to the object, cancelling the tenant permision. Once a tenant permission is cancelled, any further attempt to use it will create an error (in the interpreter, this is a runtime error; in the compiler, this is a detected by the type system and rejected).

## Examples

### Fixture

Given:

```
class Box1(var field0)
class Box2(var field0, var field1)
```

### 

```
p = Box1(Box1(22))
q = p.field0.share
```

