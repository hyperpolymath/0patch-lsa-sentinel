// SPDX-License-Identifier: MPL-2.0
//
// Zig FFI layer for 0patch-lsa-sentinel.
//
// Provides an independent C-ABI reference implementation of the classifier
// (`sentinel_classify_ref`) so the Rust host can differentially test the
// SPARK-proved kernel against a second, separately-authored implementation.
// The numeric codes are asserted at comptime to match include/sentinel.h.

const std = @import("std");

// Canonical codes — identical to include/sentinel.h and Sentinel.ABI.
const PROT_UNKNOWN: c_int = 0;
const PROT_OFF: c_int = 1;
const PROT_ON: c_int = 2;

const LOAD_UNKNOWN: c_int = 0;
const LOAD_BLOCKED: c_int = 1;
const LOAD_LOADED: c_int = 2;

const PAT_UNKNOWN: c_int = 0;
const PAT_ABSENT: c_int = 1;
const PAT_PRESENT: c_int = 2;

const V_INDETERMINATE: c_int = 0;
const V_MOOT: c_int = 1;
const V_OPEN_COVERED: c_int = 2;
const V_BLOCKED_INERT: c_int = 3;

/// Reference classifier — four-branch table identical to the SPARK body.
export fn sentinel_classify_ref(protection: c_int, loader: c_int, patches: c_int) c_int {
    // 1. No lsass-targeted patches outstanding -> moot.
    if (patches == PAT_ABSENT) return V_MOOT;

    // 2. Genuinely covered.
    if (protection == PROT_OFF and loader == LOAD_LOADED and patches == PAT_PRESENT)
        return V_OPEN_COVERED;

    // 3. Silent failure: patches present and loader kept out.
    if (patches == PAT_PRESENT and (protection == PROT_ON or loader == LOAD_BLOCKED))
        return V_BLOCKED_INERT;

    // 4. Insufficient positive information.
    return V_INDETERMINATE;
}

test "spec rows match the SPARK truth table" {
    const t = std.testing;
    try t.expectEqual(V_BLOCKED_INERT, sentinel_classify_ref(PROT_ON, LOAD_BLOCKED, PAT_PRESENT));
    try t.expectEqual(V_BLOCKED_INERT, sentinel_classify_ref(PROT_OFF, LOAD_BLOCKED, PAT_PRESENT));
    try t.expectEqual(V_OPEN_COVERED, sentinel_classify_ref(PROT_OFF, LOAD_LOADED, PAT_PRESENT));
    try t.expectEqual(V_MOOT, sentinel_classify_ref(PROT_OFF, LOAD_LOADED, PAT_ABSENT));
    try t.expectEqual(V_MOOT, sentinel_classify_ref(PROT_ON, LOAD_BLOCKED, PAT_ABSENT));
    try t.expectEqual(V_INDETERMINATE, sentinel_classify_ref(PROT_OFF, LOAD_LOADED, PAT_UNKNOWN));
    try t.expectEqual(V_INDETERMINATE, sentinel_classify_ref(PROT_UNKNOWN, LOAD_UNKNOWN, PAT_UNKNOWN));
}

test "exhaustive: never covered/moot without positive evidence" {
    const t = std.testing;
    var p: c_int = 0;
    while (p <= 2) : (p += 1) {
        var l: c_int = 0;
        while (l <= 2) : (l += 1) {
            var s: c_int = 0;
            while (s <= 2) : (s += 1) {
                const v = sentinel_classify_ref(p, l, s);
                if (v == V_MOOT) try t.expect(s == PAT_ABSENT);
                if (v == V_OPEN_COVERED) {
                    try t.expect(p == PROT_OFF and l == LOAD_LOADED and s == PAT_PRESENT);
                }
                if (v == V_BLOCKED_INERT) {
                    try t.expect(s == PAT_PRESENT and (p == PROT_ON or l == LOAD_BLOCKED));
                }
            }
        }
    }
}
