# Dada types

## General

A full Dada type consists of a *permission*, a *class*, and *generic arguments* to that class. Examples:

* `my String`
* `our Vec[String]`
* `leased{p} Vec[String]`

*Permissions* have the structure of

* `my`, which is shorthand for `given{}` and means "fully owned by this variable"
* `our`, which is shorthand for `shared{}` and means "shared" (jointly owned by this variable and possibly others)
* `leased{path1, ..., pathN}` which means "leased from one of the paths in `path1...pathN`"
    * leased values are owned by another variable, but this variable has unique access
    * there cannot be an empty set of paths.
* `shared{path1, ..., pathN}` which means "shared from one of the paths in `path1...pathN`"
    * if the set of paths is non-empty, then the value is owned by one of them (or by their lessors)
    * an empty set of paths is equivalent to `our`.

## Elided permissions

When permissions are elided, they are elision rules as follows:

* Permissions on parameter types are replaced with a fresh permission variable (so e.g. `x: String` defaults to `x: P String` for a fresh permission variable `P`).
* Permissions on generic arguments default to `my` (so e.g. `Vec[String]` defaults to `Vec[my String]`)
* Permissions on return types default to `given{self}` in methods but have no default in functions.

## Permissions at runtime

Permissions at runtime are a struct

```rust
struct Permission {
    is_shared: bool,
    has_lessor: bool,
    tenants: Vec<Permission>,
}
```

You can map the user's permisions to ...

* `my = {is_share: false, has_lessor: false}`
* `our = {is_share: true, has_lessor: false}`
* `leased{a_0,..,a_N} = {is_share: false, has_lessor: true}` where `P` appears (transitively) as a tenant of some `a_i`
* `shared{a_0,..,a_N} = {is_share: true, has_lessor: true}` where `P` appears (transitively) as a tenant of some `a_i`

But there is this weird discontinuity: if something is *leased*, it must be represented as a pointer when fully compiled (but not in the abstract machine).

### Representation at interpreter vs compilation time

In the Dada interpreter, everything is a pointer, and everything is fine.

In the compiler, given/shared values are represented "by value", but "unique leased" values must be represented by pointer. This creates a challenge, particularly since you cannot convert a given value into uleased one (uleased can be converted into shared by loading).

## Checking static types at runtime

A Dada static type `P C[G ...]` combines a permission `P` with a class `C` (and various generic arguments `G ...` to that class; each argument can be a permission or a type). Permissions can be a variable or a known permission.

### Matching permission variables

The first step is to match permission variables from the inputs. We look at the actual input types that were given. Each permission variable declared on the function becomes the union of the permissions from the argument(s) in which it appears.

Example 1:

```
fn test_fn[P](x: P String)
```

When `test_fn` is invoked, `P` becomes the permission of the string in its argument.

Example 2:

```
fn test_fn[P](x: P String, y: P String)
```

When `test_fn` is invoked, `P` becomes a set of two permissions of the string in its argument.

### Testing where clauses

Dada has two forms of where-clauses for now:

* `shared{P}`
* `leased{P}`

These are tested as follows:

* `shared{P}` is true if all the permissions in `P` have `is_shared: true`.
* `leased{P}` is true if all the permissions in `P` have `is_shared: false` and `has_lessor: true`.

### Checking known permissions

The static types are written in terms of paths (e.g., `given{x1, x2}`) and/or permision variables (e.g., `given{P}`).  They are implemented as a test from a machine permission `perm_target` that yields true (if the type matches) or false (otherwise). In the code below, the list is flattened into a lits of permissions (each variable `x` is mapped to the permission of its argument, and each permission variable `P` is mapped to its elements).In this test, `perm(x1)` refers to the permission of the object stored in `x1`.

```rust
fn matches_given(perm_target: Perm, permissions: Vec<Permission>) -> true {
    let is_shared = permissions.iter().any(|p| p.is_shared);
    let lessors = permissions.iter().flat_map(|p| new_lessor(p)).collect()
    matches_test(is_shared, lessors, perm_target)
}

fn matches_shared(perm_target: Perm, permissions: Vec<Permission>) -> true {
    let lessors = permissions.iter().flat_map(|p| new_lessor(p)).collect();
    matches_test(perm_target, true, lessors)
}

/// This is a weird function. The intuition is that it returns:
///
/// * If you give a permission `perm`, returns the lessor on the new permission.
/// 
/// Not coincidentally, this is also the same as the lessor if you share a value with the permission `P`.
///
/// It is NOT the same as the lessor you get if you lease a place with the permission `P`, because that
/// has `P` as the lessor.
fn new_lessor(perm: Perm) -> Option<Permission> {
    if !perm.has_lessor { None } else { Some(perm) }
}

fn matches_leased(perm_target: Perm, permissions: Vec<Variable>) -> true {
    matches_test(perm_target, false, permissions)
}

/// True if the permission `perm_target` of the value being returned matches the
/// characteristics desired by its return type:
/// 
/// * `is_shared` -- if false, i.e., return type demands a unique return, then `perm_target` must be unique
/// * `lessors` -- `perm_target` must be leased from one of the lessors in this list, 
fn matches_test(perm_target: Perm, is_shared: bool, lessors: Vec<Permission>) -> true { 
    // If the return type demands a unique value, but a shared type was returned, false.
    if perm_target.is_shared && !is_shared {
        return false;
    }

    // If the value returned has a lessor...
    if perm_target.has_lessor {
        // ...then `perm_target` must be leased from a member of `lessors`.
        return lessors.iter().any(|l| l.transitive_lessor_of(perm_target));
    }

    // Otherwise, the return value is owned. That is ok if the return type is
    // shared or `my`, but we can't have an owned return value when the return
    // type is something like `leased{a}`. In that case the value HAS to be leased
    // from `a`. This is required because the compiler will represent it as a pointer
    // to `a`, so we can't substitute an owned value.
    is_shared || lessors.is_empty()
}
```

## Examples

### Taking inputs

```
fn process(x: String, l: Vec[String])
```

defaults to

```
fn process[perm P, perm Q](x: P String, l: Q Vec[my String])
```

effectively, these permissions mean that `process` can only be sure to have shared (read-only) access to `x` and `l`.

Furthermore, if `Q` is shared, then Dada's type normalization rules mean that either `shared Vec[String]` or `shared Vec[shared String]` would be accepted.

### Signatures that are generic over permissions

```
class Character(
    name: String
    age: String
)

fn name(c: Character) -> given{c} String {
    c.name.give
}
```

This function takes in a character with some permission `P` and returns data *given* from that character. As described in the dyn tutorial, giving data gives all permissions to a new value -- so this effectively means the return value has permission `P` as well.

This means callers can either get shared, leased, or given access to `name`:

```
n = name(c.give) # takes ownership of the name

n = name(c.lease) # gets back a `leased String` with `c` as the lessor

n = name(c.share) # gets back a `shared String` with `c` as the lessor

n = name(c) # `c` defaults to `c.share`, so equivalent to the previous one
```

### Returning my when lease *may* be expected

```
class Character(
    name: String
    age: String
)

fn name(c: Character) -> given{c} String {
    "Arjuna" // say this returns a `my String` for now
}
```

This function returns a `my String`. It sometimes works:

* If you invoke `name(c.give)`, it works. The return value of `my String` is expected.
* If you invoke `name(c.share)`, it works. The return value of `my String` is allowed when a shared result is expected.

But it sometimes fails:

* If you invoke `name(c.lease)`, it false. The return value should be been leased from `c`, and it's not.

Therefore, it should fail the static type check too, once we get there. It will, because `my String` will not be coercible to the type `given{c}`.

### Returning an owned value when shared is expected

```
class Character(
    name: String
    age: String
)

fn name(c: Character) -> shared{c} String {
    "Arjuna"
}
```

This function always works. Even if `c` is leased, the resulting value is `my` (or `our`), doesn't matter, and so passes the test for `shared`.

### Signatures that are generic over permissions

```
class Character(
    name: String
    age: String
)

fn pick_name(c1: Character, c2: Character) -> given{c1, c2} String {
    if true { c1.name.give } else { c2.name.give }
}
```

Interesting example, particularly given the fact that `leased` has this weird "twist" that it has to be represented as a pointer.
I'm still not sure if it's going to become a huge problem.

Some calls can fail:

* e.g,. `pick_name(a.give, b.lease)` would return a `my String` but the test requires something leased from `a` or `b`

Other calls work out:

* e.g., `pick_name(a.share, b.lease)` returns a `shared{a} String`; the test requires something leased from `a` or `b`, so that works.

The static type check would, I think, succeed. But maybe it should fail, per the above. Imagine `let x: given{c1, c2} String = ...` in the code?

Two options:

* Implied bounds that add something like `OK(given{c1,c2})`, ensuring that the permissions can be combined.
* Error at the declaration site that `given{c1,c2}` is ill-formed, requiring you to write either `shared{c1,c2}`, `leased{c1,c2}`, or some more details on the arguments. Annoyingly, you can't quite right the thing you *want*, which is that "if c1 is leased or c2 is leased, then both are".


### Inner lease (Bug!)

```
fn pick_name(c1: leased Vec[leased String]) -> leased(c1) String {
    if true { c1.name.give } else { c2.name.give }
}
```

This is expanded to `P Vec[Q String]` where `leased(P, Q)`. We get back a value with a lease of both P and Q. The test above is happy because we needed a lease of P.

Note that static type system would have to track dep between P and Q, because in fact it's possible that Q could be invalidated but not P otherwise. Example:

```
s = "foo"
v = [s.lease]
t = pick_name(v.lease)
s.push("bar") # invalidates v's contents and hence t
```

### Inner shared (Bug!)

```
fn pick_name(c1: shared Vec[shared String]) -> shared(c1) String {
    if true { c1.name.give } else { c2.name.give }
}
```

This is expanded to `P Vec[Q String]` where `shared(P, Q)`. We get back a value with a shlease of Q. But the code above fails because it expects a tenant of P.

Options: 

* traverse `c1` to find permissions contained witin. I don't like this because the data may be added during execution of `pick_name` -- not in this case, but still, it feels wrong.
* make result a shlease of P too -- again, wrong, invalidating P doesn't invalidate Q (unlike with a lease)
* somehow consider this ok -- don't be as selective with the resulting perm? seems wrong. compute the result by walking values at the end? seems wrong.
* record that we traversed `P` somehow
* PS inferring Q seems hard...?
    * actually I think this is the right answer. We know that the shared fields are immutable, we can walk them.

