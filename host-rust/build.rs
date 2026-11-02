// SPDX-License-Identifier: MPL-2.0
//
// Links the SPARK-proved static library (libsentinel_core.a) only when the
// `verified-core` feature is enabled. The default build uses the in-tree Rust
// reference classifier and needs no native toolchain.
use std::path::{Path, PathBuf};
use std::process::Command;

/// Verify whether `dir` contains a usable GNAT runtime archive or shared object.
fn has_gnat_runtime(dir: &Path) -> bool {
    dir.join("libgnat.so").exists() || dir.join("libgnat.a").exists()
}

/// Query the active `gnatls -v` object search path first so multi-GCC or
/// cross-compiler hosts link the exact runtime matching `gprbuild`, falling
/// back to a deterministic target-filtered scan of `/usr/lib/gcc`.
fn discover_gnat_adalib() -> Option<PathBuf> {
    if let Some(out) = Command::new("gnatls")
        .arg("-v")
        .output()
        .ok()
        .filter(|o| o.status.success())
    {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let trimmed = line.trim().trim_end_matches('/');
            if trimmed.ends_with("/adalib") {
                let candidate = PathBuf::from(trimmed);
                if has_gnat_runtime(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let gcc_root = Path::new("/usr/lib/gcc");
    let mut triples: Vec<PathBuf> = std::fs::read_dir(gcc_root)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            target_arch.is_empty()
                || matches!(
                    p.file_name().and_then(|n| n.to_str()),
                    Some(n) if n.starts_with(&target_arch)
                )
        })
        .collect();
    triples.sort();

    let mut best: Option<(u32, PathBuf)> = None;
    for triple in triples {
        let Ok(versions) = std::fs::read_dir(&triple) else {
            continue;
        };
        let mut vers: Vec<PathBuf> = versions.flatten().map(|e| e.path()).collect();
        vers.sort();
        for ver in vers {
            let major = ver
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.split('.').next())
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(0);
            let candidate = ver.join("adalib");
            let prefer = match &best {
                Some((m, _)) => major >= *m,
                None => true,
            };
            if has_gnat_runtime(&candidate) && prefer {
                best = Some((major, candidate));
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Configure Cargo native link directives when `--features verified-core` is active.
fn main() {
    if std::env::var("CARGO_FEATURE_VERIFIED_CORE").is_ok() {
        // Built by `gprbuild -P core-spark/sentinel_core.gpr` (see Justfile).
        println!("cargo:rustc-link-search=native=../core-spark/lib");
        println!("cargo:rustc-link-lib=static=sentinel_core");
        // GNAT runtime needed by the Ada object code. Honour explicit
        // GNAT_ADALIB when set, otherwise resolve via `gnatls -v`.
        let adalib = std::env::var("GNAT_ADALIB")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(discover_gnat_adalib)
            .unwrap_or_else(|| PathBuf::from("/usr/lib/gcc/x86_64-linux-gnu/13/adalib"));
        println!("cargo:rustc-link-search=native={}", adalib.display());
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
        if adalib.join("libgnat.so").exists() {
            println!("cargo:rustc-link-lib=dylib=gnat");
        } else {
            println!("cargo:rustc-link-lib=static=gnat");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=GNAT_ADALIB");
}
