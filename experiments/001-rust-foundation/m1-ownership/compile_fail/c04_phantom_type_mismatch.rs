//! EXPECT: E0308
//! CLAIM: `Tagged<Egress>` 传给要求 `Tagged<Ingress>` 的函数被类型检查拒绝（E0308），
//!        尽管两者运行期表示逐字节相同。
//!
//! C-04：`PhantomData` 让两个运行期表示完全相同的类型在类型层面不可互换。
//!
//! 关注点：`Tagged<Ingress>` 与 `Tagged<Egress>` 的内存布局逐字节相同，大小也相同 ——
//! 区分**只存在于类型检查阶段**，运行期零开销。
//! 这就是"零成本抽象"最纯粹的形态，也是 eBPF map 类型标注一类设计的基础形状。

use std::marker::PhantomData;

pub struct Ingress;
pub struct Egress;

pub struct Tagged<T> {
    pub raw: u32,
    _marker: PhantomData<T>,
}

impl<T> Tagged<T> {
    pub fn new(raw: u32) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }
}

pub fn only_ingress(_t: Tagged<Ingress>) {}

pub fn demo() {
    only_ingress(Tagged::<Egress>::new(7));
}
