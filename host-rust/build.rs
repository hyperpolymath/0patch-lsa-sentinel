// SPDX-License-Identifier: MPL-2.0
//
// Links the SPARK-proved static library (libsentinel_core.a) only when the
// `verified-core` feature is enabled. The default build uses the in-tree Rust
// reference classifier and needs no native toolchain.
use std::path::PathBuf;

fn discover_gnat_adalib() -> Option<PathBuf> {
    let gcc_root = std::path::Path::new("/usr/lib/gcc");
    let triples = std::fs::read_dir(gcc_root).ok()?;
    let mut best: Option<PathBuf> = None;
    for triple in triples.flatten() {
        let Ok(versions) = std::fs::read_dir(triple.path()) else {
            continue;
        };
        for ver in versions.flatten() {
            let candidate = ver.path().join("adalib");
            if candidate.join("libgnat.so").exists() || candidate.join("libgnat.a").exists() {
                best = Some(candidate);
            }
        }
    }
    best
}

fn main() {
    if std::env::var("CARGO_FEATURE_VERIFIED_CORE").is_ok() {
        // Built by `gprbuild -P core-spark/sentinel_core.gpr` (see Justfile).
        println!("cargo:rustc-link-search=native=../core-spark/lib");
        println!("cargo:rustc-link-lib=static=sentinel_core");
        // GNAT runtime needed by the Ada object code. Honour explicit
        // GNAT_ADALIB when set, otherwise auto-discover under /usr/lib/gcc.
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
