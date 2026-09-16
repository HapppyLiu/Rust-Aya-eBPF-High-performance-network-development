//! C-21 errno —— 失败的 `open` 封装后仍保留原始错误码。
//!
//! 运行：`cargo run -p m6-ffi --example c21_errno`
//! PREDICT-UB: clean（ASan；覆盖面窄于 Miri，见 OBSERVATIONS）

use std::ffi::CString;

use m6_ffi::c21::{close_fd, open_path};

fn main() {
    let missing = CString::new("/no/such/m6-ffi/errno-probe").unwrap();
    let err = open_path(&missing, libc::O_RDONLY).unwrap_err();
    println!("=== open missing path ===");
    println!("  raw code = {}", err.code);
    println!("  ENOENT   = {}", libc::ENOENT);
    println!("  preserved = {}", err.code == libc::ENOENT);
    println!("  display (NON-ASSERTION) = {err}");

    let bad = close_fd(-1).unwrap_err();
    println!("=== close(-1) ===");
    println!("  raw code = {}", bad.code);
    println!("  EBADF    = {}", libc::EBADF);
}
