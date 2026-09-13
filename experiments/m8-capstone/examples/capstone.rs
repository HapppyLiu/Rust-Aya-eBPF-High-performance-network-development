//! 综合实验可观察走查（NON-ASSERTION）。
//!
//! 运行：`cargo run -p m8-capstone --example capstone`

use m8_capstone::PacketBuf;
use m8_capstone::iter::FrameIter;
use m8_capstone::parse::parse_frame;
use m8_capstone::stats::ParseStats;
use m8_capstone::wire::{FrameKind, encode_frame};

fn main() {
    let mut bytes = encode_frame(FrameKind::Data, b"ping");
    bytes.extend_from_slice(&encode_frame(FrameKind::Control, b"ack"));

    println!("buffer_len={}", bytes.len());

    match parse_frame(PacketBuf::new(&bytes)) {
        Ok((frame, rest)) => {
            println!(
                "first kind={:?} payload_len={} rest_len={}",
                frame.kind().unwrap(),
                frame.payload.len(),
                rest.len()
            );
        }
        Err(e) => println!("first_err={e}"),
    }

    let stats = ParseStats::new();
    let n = FrameIter::with_stats(PacketBuf::new(&bytes), &stats)
        .filter_map(Result::ok)
        .count();
    println!("frames_ok={} stats_frames={}", n, stats.frames());

    let truncated = encode_frame(FrameKind::Data, b"");
    let too_short = &truncated[..3];
    println!(
        "truncated={:?}",
        parse_frame(PacketBuf::new(too_short)).unwrap_err()
    );
}
