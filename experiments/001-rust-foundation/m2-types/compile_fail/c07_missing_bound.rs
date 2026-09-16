//! EXPECT: E0277
//! CLAIM: 调用要求 `T: PartialOrd` 的函数时，若调用方的 `T` 没有该 bound，
//! 编译器报 E0277（trait bound not satisfied）。直接对无 bound 的 `T` 写 `>`
//! 会先落成 E0369（运算符找不到）；本样本刻意走"调用带 bound 的函数"
//! 这条路径，让错误码对准"缺 bound"本身。

fn needs_ord<T: PartialOrd>(a: T, b: T) -> bool {
    a > b
}

pub fn max_unbound<T>(a: T, b: T) -> bool {
    needs_ord(a, b)
}
