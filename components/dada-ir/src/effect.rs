/// The "effects" that can be declared on functions.
///
/// Ordering: a "lesser" effect permits fewer things.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effect {
    /// Executes atomically. Permits atomic statements, but they are no-ops.
    Atomic,

    /// Does not permit await statements, permits atomic statements.
    Default,

    /// May contain "await" statements, permits atomic statements.
    Async,
}

impl Effect {
    pub fn permits_await(self) -> bool {
        self >= Effect::Async
    }

    pub fn permits_atomic(self) -> bool {
        self >= Effect::Atomic
    }

    pub fn is_atomic(self) -> bool {
        self <= Effect::Atomic
    }
}
