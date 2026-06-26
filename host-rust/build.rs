// SPDX-License-Identifier: MPL-2.0
//
// Links the SPARK-proved static library (libsentinel_core.a) only when the
// `verified-core` feature is enabled. The default build uses the in-tree Rust
// reference classifier and needs no native toolchain.
fn main() {
    if std::env::var("CARGO_FEATURE_VERIFIED_CORE").is_ok() {
        // Built by `gprbuild -P core-spark/sentinel_core.gpr` (see Justfile).
        println!("cargo:rustc-link-search=native=../core-spark/lib");
        println!("cargo:rustc-link-lib=static=sentinel_core");
        // GNAT runtime needed by the Ada object code. The adalib path can be
        // overridden with GNAT_ADALIB for non-default toolchain locations.
        let adalib = std::env::var("GNAT_ADALIB")
            .unwrap_or_else(|_| "/usr/lib/gcc/x86_64-linux-gnu/13/adalib".to_string());
        println!("cargo:rustc-link-search=native={adalib}");
        println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
        println!("cargo:rustc-link-lib=dylib=gnat");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
