# SPDX-License-Identifier: MPL-2.0
#
# Containerfile — 0patch-lsa-sentinel
#
# Nix retirement note: Guix became the estate-primary packaging path on
# 2026-06-01; flake.nix is kept (it predates that ruling and Guix
# packaging is not yet wired up here) but no longer satisfies the
# governance container gate on its own — a sealed, buildable
# Containerfile is the accepted escape hatch. This file is a real,
# non-stub build: every dependency install below is an active RUN
# step, and the binary produced here is the actual `lsa-sentinel`
# crate (host-rust/), the default in-tree Rust reference classifier.
#
# Repo shape note: this is a multi-language repo (Rust + SPARK/Ada +
# Idris2 + Zig), but only ONE of those is actually buildable in a
# sealed container without extra provers/toolchains this estate does
# not run in CI:
#   - host-rust/   — the real Rust crate; DEFAULT features need only
#                     rustc/cargo (see host-rust/build.rs — the
#                     `verified-core` feature is the only thing that
#                     would need the SPARK static lib + GNAT adalib,
#                     and it is off by default). This is what ships.
#   - core-spark/   — SPARK/Ada classifier, proved with GNATprove; not
#                     containerized here (no gprbuild/GNAT toolchain
#                     in Wolfi; out of scope for this packaging pass).
#   - abi-idris2/   — Idris2 ABI spec; same reasoning as above.
#   - ffi-zig/      — Zig static C-ABI library exposing the classifier
#                     over FFI. NOT linked by the default Rust build
#                     (see host-rust/build.rs), but it is real,
#                     buildable Zig code, so this Containerfile
#                     compiles it too as an active verification step
#                     in the builder stage — it is not shipped in the
#                     final runtime image since nothing in the default
#                     binary depends on it.
#
# Toolchain: Rust, edition 2021, rust-version = "1.74" (see
# host-rust/Cargo.toml). No rust-toolchain.toml pin exists in this
# repo to honour, so this uses Wolfi's rust-1.89 package (a recent
# stable release bundling both rustc and cargo; Wolfi does not ship a
# bare "cargo" package — `apk add cargo` fails with "no such
# package"). Zig 0.15 (see ffi-zig/build.zig header comment) is
# available in Wolfi as a plain `zig` package.
#
# Multi-stage build:
#   Stage 1: compile the lsa-sentinel binary (default features, no
#             native toolchain needed) with cargo --release, and
#             separately build the ffi-zig static library to verify
#             it compiles
#   Stage 2: copy only the release Rust binary into a minimal
#             Chainguard glibc runtime image (Wolfi-built Rust
#             binaries are dynamically linked, so glibc-dynamic is
#             used, not the static runtime variant)
#
# Build:  podman build -t 0patch-lsa-sentinel-verify:latest -f Containerfile .
# Run:    podman run --rm -it 0patch-lsa-sentinel-verify:latest --help
# Seal:   podman build --no-cache -t 0patch-lsa-sentinel:sealed -f Containerfile .

# --- Stage 1: Build (Rust + Zig verification) ---
FROM cgr.dev/chainguard/wolfi-base:latest AS builder

# Rust toolchain (rustc + cargo, rust-1.89 bundles both) and Zig, as
# packaged by Wolfi.
RUN apk add --no-cache rust-1.89 gcc zig

WORKDIR /build

# --- Rust crate (host-rust/): the real, shipped artifact ---
COPY host-rust/Cargo.toml host-rust/Cargo.lock ./host-rust/
COPY host-rust/src ./host-rust/src
# benches/ must be present even for a plain `cargo build`: the
# manifest declares an explicit [[bench]] target and Cargo validates
# that its path exists at manifest-parse time, regardless of whether
# the bench itself gets compiled.
COPY host-rust/benches ./host-rust/benches

RUN cd host-rust && \
    cargo build --release --bin lsa-sentinel && \
    cp target/release/lsa-sentinel /build/lsa-sentinel

# --- Zig FFI crate (ffi-zig/): active build-only verification ---
# Not linked into the default Rust binary; compiled here purely to
# prove it still builds, per the repo's own multi-language layout.
COPY ffi-zig ./ffi-zig

RUN cd ffi-zig && zig build

# --- Stage 2: Runtime ---
FROM cgr.dev/chainguard/glibc-dynamic:latest

COPY --from=builder /build/lsa-sentinel /usr/bin/lsa-sentinel

USER nonroot

ENTRYPOINT ["/usr/bin/lsa-sentinel"]
