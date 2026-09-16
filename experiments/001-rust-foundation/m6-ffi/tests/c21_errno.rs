//! C-21 Linux 错误码封装 —— 原始 `errno` 整数不得丢失（US6 AS3）。

use std::ffi::CString;
use std::io;

use m6_ffi::c21::{close_fd, open_path};

fn missing_path() -> CString {
    CString::new("/no/such/m6-ffi/errno-probe").unwrap()
}

/// CLAIM: `open` 不存在的路径失败，封装后的 `ErrnoError.code` 等于 `libc::ENOENT`。
#[test]
fn open_missing_preserves_enoent() {
    let err = open_path(&missing_path(), libc::O_RDONLY).unwrap_err();
    assert_eq!(err.code, libc::ENOENT);
}

/// CLAIM: 原始码能通过 `from_raw_os_error` 圆回来，说明封装没有丢掉内核给的整数。
/// 不断言 `Display` 全文（诊断措辞随 locale / libc 变化，§C2.2）。
#[test]
fn raw_os_error_roundtrips_through_io_error() {
    let err = open_path(&missing_path(), libc::O_RDONLY).unwrap_err();
    assert_eq!(
        io::Error::from_raw_os_error(err.code).raw_os_error(),
        Some(err.code)
    );
    let shown = err.to_string();
    assert!(
        shown.contains(&err.code.to_string()),
        "Display must keep the numeric code, got {shown:?}"
    );
}

/// CLAIM: `close(-1)` 失败码为 `EBADF`，第二种 errno 同样不被封装层吞掉。
#[test]
fn close_invalid_fd_preserves_ebadf() {
    let err = close_fd(-1).unwrap_err();
    assert_eq!(err.code, libc::EBADF);
}

/// CLAIM: 成功路径 `open("/dev/null")` + `close` 返回 `Ok`，与失败封装互为对照。
#[test]
fn open_dev_null_then_close_ok() {
    let path = CString::new("/dev/null").unwrap();
    let fd = open_path(&path, libc::O_RDONLY).expect("open /dev/null");
    assert!(fd >= 0);
    close_fd(fd).expect("close /dev/null fd");
}
