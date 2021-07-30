# Modes

There are four modes, defined along two axes:

|                 | Unique         | Joint    |
| --------------- | ----------     | -------- |
| **Owned**       | `my`           | `our`    |
| **Lent**        | `borrowed(..)` | `shared(..)` |

## The axes

There are two axes that divide the modes:

* **Unique:** When a [place] `p` has a type with a unique mode (`my` or `borrowed`), that means that whenever `p` is used, it is the only place that can access that particular value (there may be other aliases, but they are not usable between an two uses of `p`).
* **Joint:** When a [place] `p` has a type with joint mode, there may be other places that have access to the same value. This implies that the value's fields cannot be modified unless they are declared [`atomic`].

[place]: ./places.md
[`atomic`]: ./atomicity.md