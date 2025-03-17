pub trait VecExt<T> {
    fn push_if_not_contained(&mut self, element: T) -> bool
    where
        T: PartialEq;
}

impl<T> VecExt<T> for Vec<T> {
    fn push_if_not_contained(&mut self, element: T) -> bool
    where
        T: PartialEq,
    {
        if self.contains(&element) {
            false
        } else {
            self.push(element);
            true
        }
    }
}
