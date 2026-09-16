//! EXPECT: E0597
//! CLAIM: 结构体持有的引用其指向数据先失效时，被生命周期约束拒绝（E0597 "borrowed value does not live long enough"）。
//!
//! C-04：被引用的数据不能比引用它的结构体先失效。
//!
//! 关注点：`Excerpt<'a>` 的标注在这里**起了作用** —— 它把"实例不得比 part
//! 指向的数据活得更久"这一约束交给了借用检查器，于是这段代码在编译期被拒绝。
//! 若没有标注，编译器根本无从判断 `part` 何时失效。

pub struct Excerpt<'a> {
    pub part: &'a str,
}

pub fn demo() -> usize {
    let excerpt;
    {
        let owned = String::from("short-lived");
        excerpt = Excerpt { part: &owned };
    }
    excerpt.part.len()
}
