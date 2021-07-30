# Modes

There are four modes, defined along two axes:

|                 | Unique         | Joint    |
| --------------- | ----------     | -------- |
| **Owned**       | `my`           | `our`    |
| **Leased**      | `borrowed(leases)` | `shared(leases)` |

## The axes

There are two axes that divide the modes. The first is "uniqueness":

* **Unique:** When a [place] `p` has a type with a unique mode (`my` or `borrowed`), that means that whenever `p` is used, it is the only place that can access that particular value (there may be other aliases, but they are not usable between an two uses of `p`).
* **Joint:** When a [place] `p` has a type with joint mode, there may be other places that have access to the same value. This implies that the value's fields cannot be modified unless they are declared [`atomic`].

[place]: ./places.md
[`atomic`]: ./atomicity.md

The second access is "ownership":

* **Owned:** When a [place] `p` has a type with an owned mode, that means that this place independently keeps that value alive. 
* **Leased:** When a [place] `p` has a leased mode, that means that the value is owned by some other place `q` (or possibly multiple places, for joint modes), and that `p` is only allowed to access the value so long as `q` remains alive. These modes have a list of [leases] indicating the possible places from which the value may have been leased (note that in some cases, these places are not owners, but are themselves using a leased value).
