use std::{any::Any, cell::RefCell, pin::Pin};

/// A really dumb arena implementation intended not for efficiency
/// but rather to prolong lifetimes.
pub struct Arena {
    /// List of values inserted into the arena.
    /// They cannot be moved out from the box or dropped until the arena is dropped.
    ///
    /// The use of Box is needed to ensure the address of the value is stable.
    /// The `Pin` and `dyn Any` parts are just for fun and/or convenience.
    /// The pin is expressing the "don't move" constraint but is neither necessary
    /// nor sufficient for soundness (it doesn't prevent drops),
    /// and the `dyn Any` is just to capture the destructor but we don't do
    /// any downcasting.
    data: RefCell<Vec<Pin<Box<dyn Any>>>>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

impl Arena {
    pub fn new() -> Self {
        Self {
            data: Default::default(),
        }
    }

    pub fn insert<T>(&self, value: T) -> &T
    where
        T: Any,
    {
        let data = Box::pin(value);
        let ptr: *const T = &*data;
        self.data.borrow_mut().push(data);

        // UNSAFE: WE don't ever remove anything from `self.data` until self is dropped.
        // UNSAFE: The value is guaranteed to be valid for the lifetime of `self`
        unsafe { &*ptr }
    }
}
