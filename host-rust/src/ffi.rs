// SPDX-License-Identifier: MPL-2.0
//! C ABI binding to the SPARK-proved classifier (and the canonical
//! `sentinel.h` published by the Zig layer). Linked only under the
//! `verified-core` feature; see `build.rs`.

use core::ffi::c_int;

extern "C" {
    /// `int sentinel_classify(int protection, int loader, int patches);`
    /// Exported by `Sentinel.Abi` (Ada) with the encodings in `model.rs`.
    fn sentinel_classify(protection: c_int, loader: c_int, patches: c_int) -> c_int;
}

/// Safe wrapper over the proved C-ABI classifier — the only `unsafe` in the
/// crate, isolated and discharged here.
///
/// # Safety
/// `sentinel_classify` is proved in SPARK to be pure and total: it reads three
/// `int`s by value, dereferences no pointers, touches no global or thread
/// state, and always returns. Calling it with any `i32` triple is therefore
/// sound and cannot fail, block, or race.
pub fn classify_raw(protection: i32, loader: i32, patches: i32) -> i32 {
    // SAFETY: pure, total, pointer-free FFI call — see the contract above.
    // panic-attack: accepted - reviewed RuntimeAbi boundary; the callee is SPARK-proved pure and total
    unsafe { sentinel_classify(protection as c_int, loader as c_int, patches as c_int) }
}
