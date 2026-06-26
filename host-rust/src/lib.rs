// SPDX-License-Identifier: MPL-2.0
//! 0patch-lsa-sentinel — library core.
//!
//! The host collects three signals about a Windows machine and asks the
//! classifier whether 0patch's lsass-targeted micropatch coverage is live,
//! silently inert, or moot. The classifier semantics are defined once, in
//! SPARK (`core-spark/`), proved total and correct; [`classify::classify`]
//! is a byte-for-byte Rust mirror used for tests, benches, and the default
//! (no-native-toolchain) build. Enable the `verified-core` feature to call
//! the SPARK object code over the C ABI instead.

pub mod classify;
pub mod collectors;
pub mod model;
pub mod report;

#[cfg(feature = "verified-core")]
pub mod ffi;

pub use classify::classify;
pub use model::{LoaderPresence, LsaProtection, PatchStatus, Signals, Verdict};
