use std::cell::Cell;

/// This struct helps to manage tracking how many viable alternatives
/// there are for proving subtypes. Once the number of viable alternatives
/// drops to 1 we can make stronger inference.
///
/// Think of a proof tree:
///
/// * Root
///   * Option A
///     * Option A.1
///       * Option A.1.1
///       * Option A.1.2
///     * Option A.2
///   * Option B
///     * Option B.1
///     * Option B.2
///
/// Each node in this tree will have an `Alternative` struct. Each `Alternative`
/// will have a reference to its parent and will also track the number of active
/// children. To determine if a given node is required, all of its parents must
/// either be the root or have a single child. When an `Alternative` struct is dropped,
/// it decrements its parent's counter of the number of children.
///
/// In this example, no nodes but the would be considered required.
/// But once (say) the alternative for option B is dropped, then the root would have
/// one child, and hence the node for Option A would be considered required.
/// Likewise, if the node for Option A.1, then Option A.2 would be considered required
/// as Option A would have 1 child.
pub(crate) struct Alternative<'p> {
    parent: Option<&'p Alternative<'p>>,
    counter: Cell<usize>,
}

impl<'p> Alternative<'p> {
    /// Invokes `op` with an alternative that will never be considered required.
    /// This is used in scenarios where we are iterating over a series of inference
    /// bounds but we always have to assume that a future bound may arrive.
    pub async fn the_future_never_comes<R>(op: impl AsyncFnOnce(&mut Alternative<'_>) -> R) -> R {
        let mut root = Alternative::root();
        let mut children = root.spawn_children(2);
        op(&mut children[0]).await
    }

    /// Create the root alternative.
    pub fn root() -> Self {
        Self {
            parent: None,
            counter: Cell::new(0),
        }
    }

    /// Create a new child.
    fn child(parent: &'p Alternative<'p>) -> Self {
        parent.new_child();
        Self {
            parent: Some(parent),
            counter: Cell::new(0),
        }
    }

    /// Invoked by children when they are created. Increments our counter of
    /// active children.
    fn new_child(&self) {
        self.counter.set(self.counter.get() + 1);
    }

    /// Invoked by children when they are dropped. Decrements our counter of
    /// active children.
    fn drop_child(&self) {
        self.counter.set(self.counter.get().checked_sub(1).unwrap());
    }

    /// Returns true if this node is required.
    pub fn is_required(&self) -> bool {
        match self.parent {
            None => true,
            Some(p) => p.is_required() && p.counter.get() == 1,
        }
    }

    /// Spawn N children. Each of the alternatives returned will be considered active
    /// until it is dropped.
    ///
    /// It is important that all children are spawned "atomically"
    /// because that way none of them are considered required yet. If we spawned them one by one
    /// and began executing a child before other children were spawned, then they would
    /// consider themselves required incorrectly.
    pub fn spawn_children<'me>(&'me mut self, count: usize) -> Vec<Alternative<'me>> {
        assert_eq!(self.counter.get(), 0, "node already has children");
        (0..count).map(|_| Alternative::child(self)).collect()
    }
}

impl Drop for Alternative<'_> {
    fn drop(&mut self) {
        if let Some(parent) = self.parent {
            parent.drop_child();
        }
    }
}
