//! The atomics and `Arc` the ring and the snapshot buffer use. Under `--cfg loom` they are
//! loom's, so a loom test can explore every interleaving; otherwise they are the standard
//! library's. Nothing else in the crate imports atomics, so a plain build is unaffected.

#[cfg(loom)]
pub use loom::sync::{
    atomic::{AtomicU32, AtomicU64, Ordering},
    Arc,
};
#[cfg(not(loom))]
pub use std::sync::{
    atomic::{AtomicU32, AtomicU64, Ordering},
    Arc,
};
