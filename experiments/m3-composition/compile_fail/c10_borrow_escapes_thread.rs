//! EXPECT: E0373
//! CLAIM: `thread::spawn` 要求闭包 `'static`。本样本按引用捕获局部 `String`，
//! 借用无法活过当前函数，编译器拒绝（closure may outlive the current function）。
//! 这是捕获方式决定能否跨线程的一侧；另一侧（拥有所有权但类型不是 Send）见 C-11。

pub fn spawn_borrow() {
    let local = String::from("hi");
    std::thread::spawn(|| {
        let _n = local.len();
    });
}
