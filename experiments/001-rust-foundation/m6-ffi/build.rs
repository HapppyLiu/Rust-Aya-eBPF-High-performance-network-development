//! 用 `cc` 编译 `c/roundtrip.c` 并链进本 crate。
//!
//! ASan 判定（`tools/run-asan.sh`）会带 `-Zsanitizer=address`。
//! 此时 C 侧也必须用同一 sanitizer 编译，否则边界上的检测是空的。

fn rustflags_enable_asan() -> bool {
    let rustflags = std::env::var("RUSTFLAGS").unwrap_or_default();
    let encoded = std::env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
    rustflags.contains("sanitizer=address") || encoded.contains("sanitizer=address")
}

fn main() {
    let mut build = cc::Build::new();
    build.file("c/roundtrip.c");
    build.warnings(true);
    if rustflags_enable_asan() {
        build.flag("-fsanitize=address");
        build.flag("-fno-omit-frame-pointer");
    }
    build.compile("m6_roundtrip");
    println!("cargo:rerun-if-changed=c/roundtrip.c");
}
