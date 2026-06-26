# SPDX-License-Identifier: MPL-2.0
# 0patch-lsa-sentinel — cross-toolchain build orchestration.
#
# Languages: SPARK/Ada (verified kernel), Rust (host), Idris2 (ABI proofs),
# Zig (C-ABI seam + independent reference). `just all` runs the full gate.

adalib := env_var_or_default("GNAT_ADALIB", "/usr/lib/gcc/x86_64-linux-gnu/13/adalib")

# List recipes.
default:
    @just --list

# --- SPARK verified kernel -------------------------------------------------

# Build the static library libsentinel_core.a.
build-core:
    cd core-spark && gprbuild -P sentinel_core.gpr -p

# Prove the classifier postcondition with gnatprove.
prove:
    cd core-spark && gnatprove -P sentinel_core.gpr --level=2 --report=all

# --- Idris2 ABI conformance ------------------------------------------------

# Type-check (and totality-check) the ABI proofs.
abi-check:
    cd abi-idris2 && idris2 --build sentinel-abi.ipkg

# --- Zig FFI seam ----------------------------------------------------------

# Build libsentinel_ffi.a and install sentinel.h.
build-zig:
    cd ffi-zig && zig build

# Run the Zig reference-impl unit tests.
test-zig:
    cd ffi-zig && zig build test

# --- Rust host -------------------------------------------------------------

# Build the host (default = in-tree Rust reference classifier, no native deps).
build-rust:
    cd host-rust && cargo build

# Run the host test suite (portable; no native toolchain required).
test-rust:
    cd host-rust && cargo test

# Differential test: Rust mirror vs the SPARK-proved object code over the ABI.
test-verified: build-core
    cd host-rust && GNAT_ADALIB={{adalib}} cargo test --features verified-core

# Microbenchmark the classifier.
bench:
    cd host-rust && cargo bench

# --- Quality / security ----------------------------------------------------

# Static-analysis scan with panic-attack (if the binary is on PATH).
panic-attack:
    @command -v panic-attack >/dev/null 2>&1 \
      && panic-attack assail . \
      || echo "panic-attack not on PATH — build it from ../panic-attack and re-run"

# --- Aggregates ------------------------------------------------------------

# Full verification gate across every toolchain.
all: prove abi-check build-zig test-zig test-verified bench
    @echo "✓ all gates passed: SPARK proved · Idris2 ABI proved · Zig + Rust cross-checked"

# Fast path: portable Rust core only (what CI on non-Windows runners runs).
check: build-rust test-rust abi-check build-zig test-zig
    @echo "✓ portable gate passed"
