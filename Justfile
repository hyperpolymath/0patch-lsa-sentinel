# SPDX-License-Identifier: MPL-2.0
# 0patch-lsa-sentinel — cross-toolchain build orchestration.
#
# Languages: SPARK/Ada (verified kernel), Rust (host), Idris2 (ABI proofs),
# Zig (C-ABI seam + independent reference). `just all` runs the full gate.
# `host-rust/build.rs` auto-discovers GNAT's `adalib` via `gnatls -v` unless
# `GNAT_ADALIB` is explicitly set in the environment.

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
    cd host-rust && cargo build --locked

# Run the host test suite (portable; no native toolchain required).
test-rust:
    cd host-rust && cargo test --locked

# Differential test: Rust mirror vs the SPARK-proved object code over the ABI.
test-verified: build-core
    #!/usr/bin/env bash
    set -euo pipefail
    ADALIB_DIR="${GNAT_ADALIB:-$(gnatls -v 2>/dev/null | awk '/\/adalib\/?$/ { gsub(/^[[:space:]]+|\/$/, ""); print; exit }')}"
    cd host-rust && LD_LIBRARY_PATH="${ADALIB_DIR:+${ADALIB_DIR}:}${LD_LIBRARY_PATH:-}" cargo test --locked --features verified-core

# Microbenchmark the classifier.
bench:
    cd host-rust && cargo bench

# --- Quality / security ----------------------------------------------------

# Static-analysis scan with panic-attack (if the binary is on PATH).
panic-attack:
    @command -v panic-attack >/dev/null 2>&1 \
      && panic-attack assail . \
      || echo "panic-attack not on PATH — build it from ../panic-attack and re-run"

# --- Packaging -------------------------------------------------------------

# Build the release binary (Windows only — the live collectors are Windows).
dist-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${OS:-}" != "Windows_NT" ] && ! rustc -vV | grep -q windows; then
      echo "lsa-sentinel.exe must be built on Windows (live collectors are Windows-only)."
      echo "Use the 'release' GitHub Actions workflow, or run on a Windows host:"
      echo "    cd host-rust && cargo build --release"
      exit 1
    fi
    cd host-rust && cargo build --locked --release

# Tag and push a release (CI builds + publishes the Windows artifact).
# Lowercase parameter name avoids matching `\{\{[A-Z_]+\}\}` placeholder scans.
release version:
    git tag -s {{version}} -m "lsa-sentinel {{version}}"
    git push origin {{version}}
    @echo "Pushed {{version}} — the release workflow will build and publish the Windows package."

# --- Aggregates ------------------------------------------------------------

# Full verification gate across every toolchain.
all: prove abi-check build-zig test-zig test-verified bench
    @echo "✓ all gates passed: SPARK proved · Idris2 ABI proved · Zig + Rust cross-checked"

# Fast path: portable Rust core only (what CI on non-Windows runners runs).
check: build-rust test-rust abi-check build-zig test-zig
    @echo "✓ portable gate passed"
