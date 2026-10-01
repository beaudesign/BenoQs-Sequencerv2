//! `octoweb`: the WebAssembly module the page loads (ADR-0008). Owner: Forge.
//!
//! It joins the sequencer core (`octocore`) and the panel controller (`octoface`) behind a small
//! C ABI, `octoweb-abi/1`, documented in `ABI.md` and pinned by the tests. It contains no
//! sequencing logic and no panel logic: it builds the controller's page view from the engine,
//! applies the commands the controller returns, renders blocks, and hands the events and the LED
//! frame to the page as bytes in fixed buffers.
//!
//! `unsafe` is denied crate-wide. A C export needs `#[no_mangle]`, which the lint counts, so the
//! export shims in `exports.rs` allow it (attribute only, one line each, no `unsafe` keyword), and
//! `measure.rs`, behind the feature of that name, holds the one `unsafe impl`. A test pins both.

#![deny(unsafe_code)]

mod encode;
pub mod exports;
pub mod host;
#[cfg(feature = "measure")]
pub mod measure;
#[cfg(feature = "spike")]
pub mod spike;
