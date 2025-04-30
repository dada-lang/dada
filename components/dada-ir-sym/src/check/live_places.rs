use crate::ir::types::SymPlace;

use super::env::Env;

/// Placeholder for the liveness computation we will be doing
#[derive(Copy, Clone)]
pub struct LivePlaces {}

#[expect(unused_variables)]
impl LivePlaces {
    /// Assume no places are live.
    pub fn none<'db>(env: &Env<'db>) -> Self {
        Self {}
    }

    /// Used where we have to think about the right value
    pub fn fixme() -> Self {
        Self {}
    }

    pub fn is_live<'db>(&self, env: &Env<'db>, place: SymPlace<'db>) -> bool {
        true // FIXME
    }
}
