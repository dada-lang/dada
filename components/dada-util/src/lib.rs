pub use fxhash::FxHashMap as Map;
pub use fxhash::FxHashSet as Set;
pub use imstr::ImString as Text;

pub type Fallible<T> = anyhow::Result<T>;

pub use anyhow::anyhow;
pub use anyhow::bail;
pub use anyhow::Context;
