use std::{cell::Cell, task::Poll};

use dada_ir_ast::diagnostic::Errors;
use futures::FutureExt;

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
/// Each node in this tree (root, A, B, A.1, etc) will have an `Alternative` struct.
/// Each alternative will inherit a `flag` from its parent and also store its
/// own children. The flag indicates whether this node is required to be true
/// or is merely one alternative out of many. The root flag will always be true
/// but other flags begin as false. Once a given node has a true flag and a single
/// viable child, it sets that child's flag to true (which can then propagate
/// down the tree).
pub(crate) struct Alternative<'p> {
    parent: Option<&'p Alternative<'p>>,
    counter: Cell<usize>,
}

impl<'p> Alternative<'p> {
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
    fn is_required(&self) -> bool {
        match self.parent {
            None => true,
            Some(p) => p.is_required() && p.counter.get() == 1,
        }
    }

    /// Spawn N children. Each of the alternatives returned will be considered active
    /// until it is dropped.
    pub fn spawn_children<'me>(&'me self, count: usize) -> Vec<Alternative<'me>> {
        (0..count).map(|_| Alternative::child(self)).collect()
    }

    /// Choose between two options:
    ///
    /// * If the current node is required, then execute `if_required`. This is preferred
    ///   because it will generate stronger inference constraints.
    /// * If the current node is not required, execute `not_required` until it returns
    ///   true or false.
    pub fn if_required(
        &self,
        is_required: impl Future<Output = Errors<()>>,
        not_required: impl Future<Output = Errors<bool>>,
    ) -> impl Future<Output = Errors<bool>> {
        let mut not_required = Box::pin(not_required);
        let mut is_required = Box::pin(is_required);
        std::future::poll_fn(move |cx| {
            if self.is_required() {
                match is_required.poll_unpin(cx) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(true)),
                    Poll::Ready(Err(reported)) => Poll::Ready(Err(reported)),
                    Poll::Pending => Poll::Pending,
                }
            } else {
                not_required.poll_unpin(cx)
            }
        })
    }
}

impl Drop for Alternative<'_> {
    fn drop(&mut self) {
        if let Some(parent) = self.parent {
            parent.drop_child();
        }
    }
}
