//! C-05 Struct / Enum —— 非断言观察输出。
//!
//! 运行：`cargo run -p m2-types --example c05_layout`

use m2_types::c05::{ConnState, LooseHeader, PacketHeader, WireKind};
use std::mem::{align_of, offset_of, size_of};

fn main() {
    println!("=== 1. repr(C) 结构体：大小、对齐、字段偏移 ===");
    let h = PacketHeader::new(1, 6, 64);
    println!(
        "  PacketHeader {{ version: {}, kind: {}, length: {} }}",
        h.version, h.kind, h.length
    );
    println!(
        "  size={} align={} offset(length)={}",
        size_of::<PacketHeader>(),
        align_of::<PacketHeader>(),
        offset_of!(PacketHeader, length)
    );

    println!();
    println!("=== 2. 同样的有效字段，换一个顺序：填充把尺寸拉开 ===");
    println!(
        "  LooseHeader size={} align={} offset(length)={} offset(kind)={}",
        size_of::<LooseHeader>(),
        align_of::<LooseHeader>(),
        offset_of!(LooseHeader, length),
        offset_of!(LooseHeader, kind)
    );
    println!("  —— 有效载荷都是 1+2+1 字节；多出来的是对齐空位，不是数据");

    println!();
    println!("=== 3. enum 状态机：判别式与变体载荷共享一块空间 ===");
    let states = [
        ConnState::Idle,
        ConnState::Connecting { attempt: 3 },
        ConnState::Established { peer: 0x0a00_0001 },
        ConnState::Closed { reason: 54 },
    ];
    println!(
        "  ConnState size={} align={}  (最大载荷 u32 的 size={})",
        size_of::<ConnState>(),
        align_of::<ConnState>(),
        size_of::<u32>()
    );
    for s in states {
        println!("    {s:?}  tag={}", s.tag());
    }
    println!(
        "  WireKind（无载荷）size={}  —— 只有判别式时，大小就是判别式本身",
        size_of::<WireKind>()
    );

    println!();
    println!("=== 4. Option 的宽度：有的 T 能省掉判别式，有的不能 ===");
    println!(
        "  size(Option<&u8>)={}  size(&u8)={}  同宽? {}",
        size_of::<Option<&u8>>(),
        size_of::<&u8>(),
        size_of::<Option<&u8>>() == size_of::<&u8>()
    );
    println!(
        "  size(Option<u8>)={}  size(u8)={}  同宽? {}",
        size_of::<Option<u8>>(),
        size_of::<u8>(),
        size_of::<Option<u8>>() == size_of::<u8>()
    );
}
