//! C-04 Lifetime —— 非断言观察输出。
//!
//! 运行：`cargo run -p m1-ownership --example c04_lifetime`

use m1_ownership::c04::{Egress, Excerpt, Ingress, Tagged, first_word, longest};
use std::marker::PhantomData;

fn main() {
    println!("=== 1. 需要显式标注 vs 省略规则足够 ===");
    let a = String::from("longer string");
    let b = String::from("short");
    println!(
        "  longest(a, b) = {:?}   （两个入参引用，必须显式标注 'a）",
        longest(&a, &b)
    );
    println!(
        "  first_word(a)  = {:?}   （一个入参引用，省略规则可推出）",
        first_word(&a)
    );

    println!();
    println!("=== 2. 标注只影响编译期：等价函数返回相同结果 ===");
    fn elided(s: &str) -> &str {
        s
    }
    // clippy 正确地指出 'a 可以省略 —— 而这恰恰是本段要展示的：
    // 省略与不省略在语义上等价。刻意保留标注，用于与 elided 对照。
    #[allow(clippy::needless_lifetimes)]
    fn annotated<'a>(s: &'a str) -> &'a str {
        s
    }
    println!("  elided    = {:?}", elided(&a));
    println!("  annotated = {:?}", annotated(&a));
    println!("  —— 二者生成的机器码没有区别；生命周期在编译后被完全擦除");

    println!();
    println!("=== 3. 持有引用的结构体 ===");
    let text = String::from("first sentence. second sentence.");
    let excerpt = Excerpt::new(&text[..15]);
    println!("  excerpt.part() = {:?}", excerpt.part());
    println!("  —— 'a 表达的是：excerpt 不得比 text 活得更久");

    println!();
    println!("=== 4. PhantomData：零大小的类型级区分 ===");
    let ingress = Tagged::<Ingress>::new(1);
    let egress = Tagged::<Egress>::new(2);
    println!(
        "  size_of::<PhantomData<Ingress>>()  = {}",
        size_of::<PhantomData<Ingress>>()
    );
    println!(
        "  size_of::<Tagged<Ingress>>()       = {}",
        size_of::<Tagged<Ingress>>()
    );
    println!(
        "  size_of::<u32>()                   = {}",
        size_of::<u32>()
    );
    println!(
        "  ingress.raw = {}, egress.raw = {}",
        ingress.raw, egress.raw
    );
    println!("  —— 两个类型的运行期表示完全相同，却不能互相赋值");
}
