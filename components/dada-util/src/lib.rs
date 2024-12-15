#![feature(async_closure)]
#![feature(trait_upcasting)]

pub use fxhash::FxHashMap as Map;
pub use fxhash::FxHashSet as Set;
pub use imstr::ImString as Text;
pub type IndexMap<K, V> = indexmap::IndexMap<K, V, fxhash::FxBuildHasher>;

pub type Fallible<T> = anyhow::Result<T>;

pub use anyhow::anyhow;
pub use anyhow::bail;
pub use anyhow::Context;
pub use anyhow::Error;

pub use dada_util_procmacro::*;

#[macro_export]
macro_rules! debug {
    ($($t:tt)*) => {
        eprintln!($($t)*);
    }
}

pub mod typedvec;
pub mod vecset;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Never {}

unsafe impl salsa::Update for Never {
    unsafe fn maybe_update(_old_pointer: *mut Self, _new_value: Self) -> bool {
        unreachable!()
    }
}

pub mod arena;

pub mod log;

pub async fn indirect<T>(op: impl async FnOnce() -> T) -> T {
    let boxed_future = futures::future::FutureExt::boxed_local(op());
    boxed_future.await
}
