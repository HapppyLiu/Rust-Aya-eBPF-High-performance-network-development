//! C-05 Struct / Enum —— 被 `c05_layout` 的 example 与 test 复用的最小设施。
//!
//! 三组对象分别对应三个问题：
//!
//! - [`PacketHeader`] / [`LooseHeader`]：`repr(C)` 下字段顺序如何改变填充；
//! - [`ConnState`]：enum 如何同时放下判别式与变体载荷；
//! - [`Option`] 对照：哪些 `T` 能走 null-pointer optimization，哪些不能。

/// 显式 C 布局的包头。字段顺序把两个 `u8` 凑在对齐边界之前，不需要额外填充。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketHeader {
    pub version: u8,
    pub kind: u8,
    pub _pad: [u8; 2],
    pub length: u32,
}

impl PacketHeader {
    #[must_use]
    pub const fn new(version: u8, kind: u8, length: u32) -> Self {
        Self {
            version,
            kind,
            _pad: [0, 0],
            length,
        }
    }
}

/// 同样三个有效字段，但 `u16` 插在两个 `u8` 之间，迫使编译器插入填充。
///
/// 与 [`PacketHeader`] 对照：数据量相同，`size_of` 不同。差的是空位，不是有效载荷。
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LooseHeader {
    pub version: u8,
    pub length: u16,
    pub kind: u8,
}

/// 连接状态机。`repr(u8)` 把判别式宽度钉死，便于把"我是谁"和"我带着什么"分开量。
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnState {
    Idle,
    Connecting { attempt: u8 },
    Established { peer: u32 },
    Closed { reason: u16 },
}

impl ConnState {
    #[must_use]
    pub const fn tag(self) -> u8 {
        // `repr(u8)` 下判别式就是这个整数。不读载荷，只读"我是谁"。
        match self {
            Self::Idle => 0,
            Self::Connecting { .. } => 1,
            Self::Established { .. } => 2,
            Self::Closed { .. } => 3,
        }
    }
}

/// 只有判别式、没有载荷的线类型。对照 [`ConnState`]：没有变体数据时，大小就是判别式本身。
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireKind {
    Syn = 1,
    Ack = 2,
    Fin = 3,
}
