//! C-07 Generic —— 非断言观察输出。
//!
//! 运行：`cargo run -p m2-types --example c07_generic`
//!
//! 观察单态化实例：
//!   cargo +nightly rustc -p m2-types --example c07_generic -- -Z print-mono-items=yes

use m2_types::c07::{Wrapper, count_gt, max_cmp, max_cmp_dyn, max_of};
use std::mem::size_of;

fn main() {
    println!("=== 1. 一份源码，三种具体类型 ===");
    println!("  max_of<u8>  = {}", max_of(3u8, 9u8));
    println!("  max_of<u16> = {}", max_of(3u16, 9u16));
    println!("  max_of<u32> = {}", max_of(3u32, 9u32));

    println!();
    println!("=== 2. 单态化的布局后果：Wrapper<T> 的大小随 T 变 ===");
    println!(
        "  size(Wrapper<u8>)={}  size(Wrapper<u64>)={}",
        size_of::<Wrapper<u8>>(),
        size_of::<Wrapper<u64>>()
    );

    println!();
    println!("=== 3. Iterator + PartialOrd bound ===");
    println!(
        "  count_gt([1, 5, 3, 9], 4) = {}",
        count_gt([1, 5, 3, 9], 4)
    );

    println!();
    println!("=== 4. 同一套比较：泛型 vs trait 对象 ===");
    let a = 4i32;
    let b = 11i32;
    println!("  max_cmp        = {}", max_cmp(a, b));
    println!("  max_cmp_dyn    = {}", max_cmp_dyn(&a, &b));
}
