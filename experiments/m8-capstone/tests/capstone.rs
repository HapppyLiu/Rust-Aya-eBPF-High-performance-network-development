//! US8 综合实验 —— 解析正确性、错误路径、分配次数、Send/Sync、迭代器。
//!
//! 每个 `#[test]` 带 `CLAIM`（FR-015）。不断言地址、耗时、线程交错（R-05）。

use m8_capstone::MAGIC;
use m8_capstone::buffer::{OwnedBytes, PacketBuf};
use m8_capstone::error::ParseError;
use m8_capstone::iter::{FrameIter, boxed_header_parser, map_ok_payloads};
use m8_capstone::parse::{Frame, HeaderParser, ParseHeader, parse_frame, parse_header_dyn};
use m8_capstone::stats::ParseStats;
use m8_capstone::wire::{FrameKind, HEADER_SIZE, encode_frame, encode_header};
use rf_harness::counting_alloc::{CountingAllocator, measure};

#[global_allocator]
static A: CountingAllocator = CountingAllocator::new();

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn one_frame(kind: FrameKind, payload: &[u8]) -> Vec<u8> {
    encode_frame(kind, payload)
}

/// CLAIM: 合法 Data 帧解析成功，载荷是原缓冲的子切片（零拷贝，指针关系而非地址数值）。
#[test]
fn parses_data_frame_zero_copy() {
    let bytes = one_frame(FrameKind::Data, b"hi");
    let buf = PacketBuf::new(&bytes);
    let (frame, rest) = parse_frame(buf).expect("valid frame");
    assert_eq!(frame.kind().unwrap(), FrameKind::Data);
    assert_eq!(frame.payload.as_bytes(), b"hi");
    assert!(rest.is_empty());
    assert_eq!(
        frame.payload.as_bytes().as_ptr(),
        bytes[HEADER_SIZE..].as_ptr(),
        "payload view must point into the original buffer"
    );
}

/// CLAIM: `OwnedBytes` 拥有拷贝后的字节；`as_buf` 的寿命钉在拥有者上，解析结果一致。
#[test]
fn owned_bytes_borrow_matches_parse() {
    let bytes = one_frame(FrameKind::Control, b"xy");
    let owned = OwnedBytes::from_slice(&bytes);
    let (frame, _) = parse_frame(owned.as_buf()).expect("valid");
    assert_eq!(frame.kind().unwrap(), FrameKind::Control);
    assert_eq!(frame.payload.as_bytes(), b"xy");
    let moved = owned.into_vec();
    assert_eq!(moved, bytes);
}

/// CLAIM: 未知 `kind` 字节 → `ParseError::BadKind`。
#[test]
fn unknown_kind_is_bad_kind() {
    let mut bytes = encode_header(FrameKind::Data, 0).to_vec();
    bytes[3] = 9;
    let err = parse_frame(PacketBuf::new(&bytes)).unwrap_err();
    assert_eq!(err, ParseError::BadKind);
}

/// CLAIM: 魔数错误 → `BadMagic`；版本错误 → `BadVersion`。
#[test]
fn bad_magic_and_version_are_distinct() {
    let mut magic = encode_header(FrameKind::Data, 0);
    magic[0] = 0x00;
    magic[1] = 0x00;
    assert_eq!(
        parse_frame(PacketBuf::new(&magic)).unwrap_err(),
        ParseError::BadMagic
    );

    let mut ver = encode_header(FrameKind::Data, 0);
    ver[2] = 99;
    assert_eq!(
        parse_frame(PacketBuf::new(&ver)).unwrap_err(),
        ParseError::BadVersion
    );
    assert_ne!(MAGIC, 0);
}

/// CLAIM: 切片短于报文头 → `Truncated`；头完整但载荷不足 → 同样 `Truncated`。
#[test]
fn truncated_header_and_payload() {
    let short = [0xA5, 0xA5, 1];
    assert_eq!(
        parse_frame(PacketBuf::new(&short)).unwrap_err(),
        ParseError::Truncated
    );
    let header_only = encode_header(FrameKind::Data, 4);
    assert_eq!(
        parse_frame(PacketBuf::new(&header_only)).unwrap_err(),
        ParseError::Truncated
    );
}

/// CLAIM: `&dyn ParseHeader` 与具体类型解析出同一个头。
#[test]
fn dyn_header_parser_matches_static() {
    let bytes = encode_header(FrameKind::Data, 0);
    let buf = PacketBuf::new(&bytes);
    let (a, _) = HeaderParser.parse_header(buf).unwrap();
    let (b, _) = parse_header_dyn(&HeaderParser, buf).unwrap();
    assert_eq!(a, b);
}

/// CLAIM: 空缓冲上的 [`FrameIter`] 立即结束，不产出错误。
#[test]
fn empty_iter_yields_nothing() {
    let mut iter = FrameIter::new(PacketBuf::empty());
    assert!(iter.next().is_none());
}

/// CLAIM: 两帧拼接时迭代器按顺序产出两帧；`map_ok_payloads` 闭包看到各自载荷。
#[test]
fn iterates_two_frames_and_maps_payloads() {
    let mut bytes = one_frame(FrameKind::Data, b"ab");
    bytes.extend_from_slice(&one_frame(FrameKind::Control, b"cd"));
    let lens = map_ok_payloads(FrameIter::new(PacketBuf::new(&bytes)), |p| p.len());
    assert_eq!(lens, [2, 2]);
}

/// CLAIM: `Box<dyn ParseHeader>` 独占堆上带状态的解析器，解析结果与栈上的 `HeaderParser` 相同。
#[test]
fn boxed_parser_agrees_with_stack() {
    let bytes = encode_header(FrameKind::Control, 0);
    let boxed = boxed_header_parser();
    let (h1, _) = boxed.parse_header(PacketBuf::new(&bytes)).unwrap();
    let (h2, _) = HeaderParser.parse_header(PacketBuf::new(&bytes)).unwrap();
    assert_eq!(h1, h2);
}

/// CLAIM: `PacketBuf` / `Frame` / `ParseStats` 均为 `Send + Sync`（编译器为裁判）。
#[test]
fn views_and_stats_are_send_sync() {
    assert_send::<PacketBuf<'static>>();
    assert_sync::<PacketBuf<'static>>();
    assert_send::<Frame<'static>>();
    assert_sync::<Frame<'static>>();
    assert_send::<ParseStats>();
    assert_sync::<ParseStats>();
}

/// CLAIM: 两个 scoped 线程各 `record_ok` 一次，终值为 2（不断言交错顺序）。
#[test]
fn concurrent_stats_end_at_two() {
    let stats = ParseStats::new();
    std::thread::scope(|scope| {
        scope.spawn(|| stats.record_ok());
        scope.spawn(|| stats.record_ok());
    });
    assert_eq!(stats.frames(), 2);
    assert_eq!(stats.errors(), 0);
}

/// CLAIM: 带 stats 的迭代器在两帧成功时 `frames == 2`；截断帧记一次 `errors`。
#[test]
fn iter_updates_stats() {
    let bytes = one_frame(FrameKind::Data, b"z");
    let stats = ParseStats::new();
    let n = FrameIter::with_stats(PacketBuf::new(&bytes), &stats)
        .filter_map(Result::ok)
        .count();
    assert_eq!(n, 1);
    assert_eq!(stats.frames(), 1);

    let stats2 = ParseStats::new();
    let bad = encode_header(FrameKind::Data, 8);
    let err = FrameIter::with_stats(PacketBuf::new(&bad), &stats2)
        .next()
        .unwrap()
        .unwrap_err();
    assert_eq!(err, ParseError::Truncated);
    assert_eq!(stats2.errors(), 1);
}

/// CLAIM: 解析栈上的借用视图不堆分配；`OwnedBytes::from_slice` 恰好一次分配。
#[test]
fn parse_borrowed_view_allocates_zero() {
    let bytes = one_frame(FrameKind::Data, b"ok");
    let (_, parse_stats) = measure(|| {
        let _ = parse_frame(PacketBuf::new(&bytes)).unwrap();
    });
    assert_eq!(parse_stats.allocs, 0, "zero-copy parse must not allocate");

    let (_, owned_stats) = measure(|| {
        let _owned = OwnedBytes::from_slice(&bytes);
    });
    assert_eq!(owned_stats.allocs, 1, "to_vec copies onto the heap once");
}

/// CLAIM: `boxed_header_parser` 装箱带 `u16` 状态的解析器，恰好一次堆分配（ZST 的 `Box` 不会分配）。
#[test]
fn boxing_parser_allocates_once() {
    let (_, stats) = measure(|| {
        let _p = boxed_header_parser();
    });
    assert_eq!(stats.allocs, 1);
}
