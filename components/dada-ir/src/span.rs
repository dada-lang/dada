#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[track_caller]
    pub fn from(start: impl TryInto<u32>, end: impl TryInto<u32>) -> Self {
        Self {
            start: start.try_into().ok().unwrap(),
            end: end.try_into().ok().unwrap(),
        }
    }

    pub fn len(self) -> u32 {
        self.end - self.start
    }
}
