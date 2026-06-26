/* SPDX-License-Identifier: MPL-2.0
 *
 * sentinel.h — the canonical C ABI for the 0patch-lsa-sentinel classifier.
 *
 * This header is the single source of truth for the cross-language seam.
 * The integer values below MUST equal:
 *   - Sentinel.Classifier / Sentinel.Abi   (SPARK, core-spark/)
 *   - Sentinel.ABI encode/decode           (Idris2, abi-idris2/, machine-proved)
 *   - model.rs enum discriminants          (Rust, host-rust/)
 * Renumbering any of these breaks the Idris2 round-trip proofs and the Rust
 * differential test.
 */
#ifndef SENTINEL_H
#define SENTINEL_H

enum sentinel_protection {
    SENTINEL_PROT_UNKNOWN = 0,
    SENTINEL_PROT_OFF     = 1,
    SENTINEL_PROT_ON      = 2
};

enum sentinel_loader {
    SENTINEL_LOAD_UNKNOWN = 0,
    SENTINEL_LOAD_BLOCKED = 1,
    SENTINEL_LOAD_LOADED  = 2
};

enum sentinel_patches {
    SENTINEL_PAT_UNKNOWN = 0,
    SENTINEL_PAT_ABSENT  = 1,
    SENTINEL_PAT_PRESENT = 2
};

enum sentinel_verdict {
    SENTINEL_INDETERMINATE = 0,
    SENTINEL_MOOT          = 1,
    SENTINEL_OPEN_COVERED  = 2,
    SENTINEL_BLOCKED_INERT = 3
};

#ifdef __cplusplus
extern "C" {
#endif

/* The production classifier, provided by the SPARK-proved kernel
 * (libsentinel_core.a, External_Name => "sentinel_classify"). */
int sentinel_classify(int protection, int loader, int patches);

/* An independent Zig reference implementation (libsentinel_ffi.a) used for
 * differential testing against the SPARK kernel. */
int sentinel_classify_ref(int protection, int loader, int patches);

#ifdef __cplusplus
}
#endif

#endif /* SENTINEL_H */
