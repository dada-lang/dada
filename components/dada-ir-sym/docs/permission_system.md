# Permission System and Predicates

Dada's permission system implements memory safety guarantees. It ensures that memory is accessed safely without garbage collection by tracking ownership and borrowing relationships at compile time.

## The Four Core Permissions

Dada's permission system is built around four fundamental permissions:

### `my` - Unique Ownership
- **Semantics**: Exclusive ownership of a value
- **Operations**: Can read, write, move, or lend the value
- **Analogous to**: Rust's owned values (`T`)

```text
let account = BankAccount("Alice", 100)  // account has 'my' permission
account.deposit(50)  // can mutate because we own it
```

### `our` - Shared Ownership  
- **Semantics**: Shared ownership among multiple references
- **Operations**: Can read the value, but not mutate or move it
- **Analogous to**: Rust's `Arc<T>` or `Rc<T>`

```text
let shared_config: our Config = get_config()
shared_config.read_setting("timeout")  // can read
// shared_config.update_setting(...) // ERROR: cannot mutate shared data
```

### `mut` - Mutable Borrow
- **Semantics**: Temporary exclusive access for mutation
- **Operations**: Can read and write, but not move the value
- **Analogous to**: Rust's `&mut T`

```text
fn process_account(mut account: BankAccount) {
    account.deposit(100)  // can mutate the borrowed account
    // account cannot be moved or returned
}
```

### `ref` - Immutable Borrow  
- **Semantics**: Temporary read-only access
- **Operations**: Can only read the value
- **Analogous to**: Rust's `&T`

```text
fn read_balance(ref account: BankAccount) -> Amount {
    account.balance  // can read fields
    // account.deposit(50) // ERROR: cannot mutate through ref
}
```

## Permission Predicates

The type checker uses predicates to reason about permissions. Each predicate asks whether a type satisfies a specific ownership property:

### Provability Predicates
- **`Shared`** - Can the value be safely shared among multiple references?
- **`Unique`** - Does this reference have exclusive access to the value?
- **`Owned`** - Do we have ownership (can move/destroy) of this value?
- **`Lent`** - Is this a borrowed reference to a value owned elsewhere?

### Requirement Predicates
- **`require_shared`** - Assert that a value must be shareable
- **`require_unique`** - Assert that a value must have unique access
- **`require_owned`** - Assert that a value must be owned
- **`require_lent`** - Assert that a value must be borrowed

## Permission Lattice

Permissions form a lattice structure that guides subtyping and conversion:

```
     my (unique + owned)
    /  \
  mut   our (shared)
   |   /
  ref (shared + lent)
```

This lattice defines legal conversions:
- `my` can be converted to `mut`, `our`, or `ref`
- `mut` can be converted to `ref` 
- `our` can be converted to `ref`
- Conversions up the lattice are not allowed

## Permission Inference

The type checker automatically infers permissions based on usage:

```text
fn example() {
    let x = create_object()  // x inferred as 'my Object'
    
    process_readonly(x)      // x converted to 'ref Object' for this call
    process_mutably(x.mut)   // explicit conversion to 'mut Object'
    
    let y = x               // x moved to y, x is no longer accessible
}
```

## Borrowing Rules

Dada enforces borrowing rules similar to Rust but with some differences:

1. **Exclusive mutation**: Only one `mut` borrow can exist at a time
2. **Shared reading**: Multiple `ref` borrows can coexist
3. **No aliasing mutation**: Cannot have `mut` and `ref` borrows simultaneously
4. **Lifetime management**: Borrows must not outlive the borrowed value

## Implementation

The permission system is implemented through:

- **[`Predicate`]** - Enumeration of the four core predicates
- **Provability modules** - Logic for determining if predicates hold
- **Requirement modules** - Logic for asserting predicates must hold
- **[`var_infer`]** - Permission inference for type variables

Each predicate has both "provably" and "require" implementations:
- **Provably**: Can we prove this predicate holds? (used for subtyping)
- **Require**: Assert this predicate must hold (used for constraint generation)

## Error Messages

When permission rules are violated, the system generates error messages:

```text
error: cannot mutate shared value
  --> example.dada:5:8
   |
5  |     shared_data.update_field(new_value)
   |     ^^^^^^^^^^^ this value has 'our' permission, which only allows reading
   |
help: to mutate this value, you need 'my' or 'mut' permission
```

The permission system's predicates work together with type inference to provide memory safety without requiring extensive annotations from the programmer.