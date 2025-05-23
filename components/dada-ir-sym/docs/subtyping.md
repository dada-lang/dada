# Subtyping System

Dada's subtyping system defines when one type can be used in place of another. It combines structural subtyping (based on the shape/capabilities of types) with nominal subtyping (based on explicit inheritance relationships).

## Core Subtyping Relations

### Permission Subtyping

Permissions form a subtyping lattice that allows safe conversions:

```
my T  <:  mut T  <:  ref T
my T  <:  our T  <:  ref T
```

This means:
- A uniquely owned value (`my T`) can be used where any permission is expected
- A mutable borrow (`mut T`) can be used where an immutable borrow (`ref T`) is expected  
- A shared value (`our T`) can be used where an immutable borrow (`ref T`) is expected

### Structural Subtyping

Dada uses structural subtyping for certain built-in types:

- **Numeric types**: `Int <: Float` (integers can be used where floats are expected)
- **Future types**: `Future[T] <: T` when `T` is async-compatible
- **Generic covariance**: `Vec[U] <: Vec[T]` when `U <: T`

### Class Hierarchy Subtyping

Classes participate in nominal subtyping through inheritance:

```text
class Animal { ... }
class Dog extends Animal { ... }
class Labrador extends Dog { ... }

# Labrador <: Dog <: Animal
```

## Subtyping Rules

### Contravariance in Function Parameters

Function types are contravariant in their parameter types:

```text
fn process_animal(f: (Animal) -> Unit) { ... }

let dog_handler: (Dog) -> Unit = ...
process_animal(dog_handler)  # ERROR: Dog handler can't handle arbitrary Animals

let animal_handler: (Animal) -> Unit = ...  
process_animal(animal_handler)  # OK: Animal handler can handle any Animal
```

### Covariance in Return Types

Function types are covariant in their return types:

```text
fn get_pet_factory() -> (() -> Animal) { ... }

let dog_factory: () -> Dog = ...
let pet_factory: () -> Animal = dog_factory  # OK: Dog factory produces Animals
```

### Permission Preservation

Subtyping must preserve permission requirements:

```text
fn needs_mutable(mut x: SomeClass) { ... }
fn needs_readonly(ref x: SomeClass) { ... }

let owned: my SomeClass = ...
needs_mutable(owned.mut)   # OK: convert my -> mut
needs_readonly(owned.ref)  # OK: convert my -> ref

let shared: our SomeClass = ...
needs_readonly(shared.ref) # OK: convert our -> ref  
needs_mutable(shared.mut)  # ERROR: cannot get mut from our
```

## Implementation

The subtyping checker is implemented through several key functions:

### Type Relations
- **[`is_numeric`]** - Handles numeric type conversions
- **[`is_future`]** - Manages async/await type relationships  
- **[`terms`]** - Core subtyping logic for type terms

### Permission Relations
- **[`perms`]** - Permission subtyping and conversion rules
- **[`relate_infer_bounds`]** - Constraint propagation for inference variables

## Subtyping and Inference

The subtyping system works closely with type inference:

1. **Constraint generation**: Subtyping requirements create inference constraints
2. **Bound propagation**: Subtype relationships are propagated to inference variables
3. **Conflict resolution**: Contradictory subtyping constraints generate type errors

Example:
```text
let x = if condition { Dog() } else { Cat() }
# x gets inferred type: some supertype of Dog and Cat
# If no common supertype exists, this is a type error
```

## Error Messages

Subtyping failures generate specific error messages:

```
error: type mismatch
  --> example.dada:3:5  
   |
3  |     dog_only_function(cat)
   |     ^^^^^^^^^^^^^^^^^ expected Dog, found Cat
   |
   = note: Cat is not a subtype of Dog
   = help: consider using a common supertype like Animal
```

## Advanced Features

### Bounded Quantification

Generic types can have subtyping bounds:

```text
class Container[T where T <: Serializable] {
    fn save(self) { self.data.serialize() }
}
```

### Intersection and Union Types

Future versions of Dada may support:
- **Intersection types**: `A & B` (has both A and B capabilities)
- **Union types**: `A | B` (is either A or B)

The subtyping system provides the foundation for these advanced type features.