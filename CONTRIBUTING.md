<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Contributing to 0patch-lsa-sentinel

Thank you for your interest in contributing. This repository is a public
design, alert, and multi-language reference implementation for containing the
0patch ↔ Windows LSA Protection conflict. Contributions — corrections,
evidence, code, and review — are welcome.

Please also read the [Code of Conduct](CODE_OF_CONDUCT.md), the
[Governance model](GOVERNANCE.adoc), and the [Security policy](SECURITY.md).

## Ground truth first

This project's headline value is *honesty about coverage*. The same discipline
applies to contributions:

- Do not claim something is verified unless a check produces that result.
- Distinguish proven (SPARK/Idris2), tested (Rust/Zig), and untested
  (the Windows collectors) — see [`AFFIRMATION.adoc`](AFFIRMATION.adoc).
- A patch that cannot be deployed is not the same as a machine that is
  protected; do not let a contribution imply otherwise.

## Project layout

```
core-spark/    SPARK/Ada verified classifier kernel (gnatprove)
abi-idris2/    Idris2 ABI conformance proofs (totality-checked)
host-rust/     Rust host application + Windows collectors
ffi-zig/       Zig C-ABI seam + independent reference
audits/        Accepted structural residuals + assail classifications
DESIGN.md      The design and the public alert
README.md      Entry point
Justfile       Cross-toolchain build orchestration
```

## Toolchain

You will need, depending on what you touch:

- **SPARK/Ada** — `gnatprove`, `gprbuild` (GNAT toolchain)
- **Idris2** — `idris2`
- **Rust** — `cargo` / `rustc`
- **Zig** — `zig`

The full gate runs with:

```sh
just all          # build-core, prove, abi-check, ffi, rust host
just prove        # gnatprove on the classifier
just abi-check    # idris2 ABI proofs
```

## How to contribute

### Reporting issues

Search existing issues first. Include environment details (OS, toolchain
versions), steps to reproduce, and expected vs actual behaviour. For anything
touching the LSA Protection / 0patch interaction, cite the source (vendor KB,
Event ID, registry key) where you can.

### Pull requests

1. Fork and create a topic branch (`fix/...`, `feat/...`, `docs/...`).
2. Keep changes focused; one logical change per PR.
3. Ensure every source file carries an SPDX header
   (`MPL-2.0` for code, `CC-BY-SA-4.0` for prose).
4. Run the relevant checks locally (`just prove`, `just abi-check`,
   `cargo test`, `zig build test`).
5. Sign your commits (SSH or GPG).
6. Follow [Conventional Commits](https://www.conventionalcommits.org/).

### Licensing of contributions

By contributing you agree that your contributions are licensed under the
project's terms: **MPL-2.0** for code, **CC-BY-SA-4.0** for documentation and
prose.

## Review

This is a sole-maintainer project; the maintainer reviews all contributions.
Significant or structural changes should be discussed in an issue first (see
[GOVERNANCE.adoc](GOVERNANCE.adoc)).
