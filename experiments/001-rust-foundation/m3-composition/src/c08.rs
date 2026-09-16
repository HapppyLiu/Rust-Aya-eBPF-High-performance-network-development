//! C-08 Error handling —— 被 `c08_error` 的 example 与 test 复用的最小设施。
//!
//! 同一套端口解析，两条传播路径：
//! - [`parse_port_manual`]：每个调用点手写 `match` + `From::from`；
//! - [`parse_port`]：`?` 走 `FromResidual`，转换仍是同一个 `From`。

/// 内层错误：字符串还不是一个 `u16`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErr {
    Empty,
    BadDigit,
}

/// 外层错误：解析失败，或解析成功但端口不在本层允许的范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErr {
    Parse(ParseErr),
    OutOfRange,
}

impl From<ParseErr> for AppErr {
    fn from(e: ParseErr) -> Self {
        AppErr::Parse(e)
    }
}

/// 只负责"这段字节是不是 `u16`"，错误类型停留在 [`ParseErr`]。
pub fn parse_u16(s: &str) -> Result<u16, ParseErr> {
    if s.is_empty() {
        return Err(ParseErr::Empty);
    }
    s.parse::<u16>().map_err(|_| ParseErr::BadDigit)
}

fn check_port(n: u16) -> Result<u16, AppErr> {
    // 教学用的本层规则：0 与 >1023 都拒绝。失败类型已经是 AppErr，不再经过 From。
    if n == 0 || n > 1023 {
        Err(AppErr::OutOfRange)
    } else {
        Ok(n)
    }
}

/// 手写传播。每个 `Err` 都看得见 `From::from`。
pub fn parse_port_manual(s: &str) -> Result<u16, AppErr> {
    match parse_u16(s) {
        Ok(n) => check_port(n),
        Err(e) => Err(AppErr::from(e)),
    }
}

/// `?` 传播。失败路径的转换由 `FromResidual` 调用同一个 `From`。
pub fn parse_port(s: &str) -> Result<u16, AppErr> {
    let n = parse_u16(s)?;
    check_port(n)
}
