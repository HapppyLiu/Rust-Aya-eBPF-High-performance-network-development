# Send / Sync 判定题集 —— 一次性作答

**作答日期**: 2026-09-08
**题集**: [send-sync-quiz.md](./send-sync-quiz.md)（冻结 2026-09-04）
**规则**: 判定以编译器为准（`cargo test -p m4-concurrency --test c12_send_sync_quiz`）。
每题推导按 auto trait 规则与标准库 impl 写出；与编译器冲突时以编译器为准。

## 作答表

| # | 类型 | Send? | Sync? | 推导依据 |
|---|-----|-------|-------|---------|
| 1 | `Tick` | 是 | 是 | 唯一字段 `u64` 两者皆是，自动推导通过 |
| 2 | `Slot` | 是 | 否 | `Cell<u32>`：`T: Send` 则 `Send`；`Cell` 显式 `!Sync` |
| 3 | `Shared` | 否 | 否 | `Rc<u32>` 显式 `!Send` 与 `!Sync`（非原子计数） |
| 4 | `RawSlot` | 否 | 否 | `*mut u8` 显式两者皆非；`usize` 拉不回来 |
| 5 | `Guarded` | 是 | 是 | `Mutex<T>: Sync` 的条件是 `T: Send`。`Cell<u32>: Send`，锁把可移动提升为可共享 |
| 6 | `SharedMut` | 否 | 否 | `Arc<T>` 两边都要求 `T: Send + Sync`。`RefCell<u32>` 不是 `Sync`，`Arc` 不提升 |
| 7 | `Held<'a>` | 否 | 是 | `MutexGuard` 显式 `!Send`（同一线程解锁）；`T: Sync` 时守卫是 `Sync` |
| 8 | `Marked` | 否 | 否 | `PhantomData<*const u8>` 按 `*const u8` 推导，两者皆非；零大小仍参与 |
| 9 | `Ticker` | 是 | 是 | `AtomicUsize` 有显式 `Send` + `Sync`（原子指令） |
| 10 | `Callback` | 是 | 否 | `Box` 原样传递；`dyn Fn() + Send` 只标了 `Send` |
| 11 | `RawCell` | 是 | 否 | `UnsafeCell<u32>`：`T: Send` 则 `Send`；显式 `!Sync` |
| 12 | `Peek<'a>` | 否 | 否 | `&T: Send`/`Sync` 都要求 `T: Sync`。`Cell<u32>` 不是 `Sync` |

## 逐题推导

### 1. `Tick { count: u64 }` — Send 是 / Sync 是

`u64` 是原始整数，两者皆是。结构体没有显式 impl，按字段自动推导。
这是基线：后面每题都是"某个字段把它拉下来"。

### 2. `Slot { value: Cell<u32> }` — Send 是 / Sync 否

`Cell<u32>` 内含 `UnsafeCell`，有显式 `!Sync`。共享 `&Slot` 等于两个线程同时
`set` 同一个非原子格子。
但它是 `Send`：把整个 `Slot` 移走之后原线程碰不到它。
**含内部可变性 ≠ 不能跨线程**；不能的是共享。

### 3. `Shared { handle: Rc<u32> }` — Send 否 / Sync 否

`Rc` 的引用计数是普通 `Cell<usize>`。两个线程同时 `clone`/`drop` 会丢更新。
只标 `!Sync` 不够：把一个 `Rc` 移走、另一个克隆留在原线程，两边仍改同一计数器，
所以必须连 `!Send` 一起写。

### 4. `RawSlot { ptr: *mut u8, len: usize }` — Send 否 / Sync 否

裸指针两者皆非。不是因为解引用不安全（那是另一问），而是编译器对指向的东西
一无所知，默认拒绝。要覆盖就得手写 `unsafe impl`，那句 `unsafe` 的含义是
"我来为跨线程安全性负责"。

### 5. `Guarded { inner: Mutex<Cell<u32>> }` — Send 是 / Sync 是

`Mutex<T>: Sync` 要求 **`T: Send`**，不是 `T: Sync`。
`Cell<u32>` 是 `Send`（第 2 题），所以整个 `Guarded` 两者皆是。
锁保证同一时刻最多一个线程碰到内部的 `Cell`，因此不要求 `Cell` 自己可共享。
这是"用锁换取共享能力"的类型级表述。

### 6. `SharedMut { inner: Arc<RefCell<u32>> }` — Send 否 / Sync 否

与第 5 题成对：`Arc` 不提升。`Arc<T>` 的 `Send`/`Sync` 都要求 `T: Send + Sync`。
`RefCell<u32>` 不是 `Sync`（`BorrowFlag` 非原子），条件不满足。
多个线程可以同时拿到 `&RefCell` 并 `borrow_mut()`，检查本身就会竞争。
正确组合是 `Arc<Mutex<T>>`。

### 7. `Held<'a> { guard: MutexGuard<'a, u32> }` — Send 否 / Sync 是

`MutexGuard` 显式 `!Send`：POSIX 要求 `pthread_mutex_unlock` 由加锁的同一线程调用。
守卫的 `Drop` 会解锁，移到别的线程再析构就会在错误的线程上解锁。
`&MutexGuard` 只能用来读 `u32`，不能触发 `Drop`，所以 `Sync` 成立。
**两条标记独立**，不存在"Sync 比 Send 强"。

### 8. `Marked { id: u32, _marker: PhantomData<*const u8> }` — Send 否 / Sync 否

`PhantomData<T>` 在 auto trait 推导里与 `T` 完全一致。`*const u8` 两者皆非。
`PhantomData` 大小为 0，运行期不占空间，却照样参与推导。
"占不占内存"和"参不参与 auto trait"是两件事。

### 9. `Ticker { hits: AtomicUsize }` — Send 是 / Sync 是

原子类型有显式 `Send`/`Sync`。内部也是 `UnsafeCell`，但所有操作走原子指令。
与第 2 题对照：差别不在有没有内部可变性，而在修改是否原子。

### 10. `Callback { f: Box<dyn Fn() + Send> }` — Send 是 / Sync 否

`Box<T>` 原样传递。trait object 的 auto trait 集合就是**写在类型里的那些**。
这里只写了 `+ Send`，所以是 `Send`、不是 `Sync`。
要两者都有，得写 `Box<dyn Fn() + Send + Sync>`。

### 11. `RawCell { inner: UnsafeCell<u32> }` — Send 是 / Sync 否

第 2 题的去皮版。`UnsafeCell` 显式 `!Sync`，`T: Send` 时是 `Send`。
它是整个语言里唯一能从 `&T` 合法得到 `&mut T` 的类型；
需要并发安全的类型再自己 `unsafe impl Sync` 加回来。

### 12. `Peek<'a> { view: &'a Cell<u32> }` — Send 否 / Sync 否

`&T: Send` 与 `&T: Sync` 都由 **`T: Sync`** 决定。
`Cell<u32>` 不是 `Sync`，所以 `&Cell<u32>` 两者皆否。
`&T: Send` 要求 `T: Sync` 而不是 `T: Send`：把 `&T` 送到另一线程，
就等于两个线程同时持有 `T` 的引用 —— 那正是 `Sync` 的定义。
即 **`T: Sync` ⟺ `&T: Send`**。

## 与编译器对照

```bash
cargo test -p m4-concurrency --test c12_send_sync_quiz
```

2026-09-08：8 个测试全绿（7 条正向 + 9 个负向样本的 E0277）。
每题 `Send`/`Sync` 都与编译器一致，且每题有推导依据。
**错题数 = 0（≤ 1）→ SC-007 / 规则 F5 通过。**
