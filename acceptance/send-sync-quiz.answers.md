# Send / Sync 判定题集 —— 参考答案

> # ⛔ 作答完成并提交 `send-sync-quiz.result.md` 之前，不要读这个文件。
>
> 规则 F3。这不是形式主义 —— 题集的全部价值在于"一次性作答"，
> 提前看过答案之后，SC-007 就再也无法测出你的 Send/Sync 心智模型是否成立。

**冻结日期**: 2026-09-04 | **对应题集**: [send-sync-quiz.md](./send-sync-quiz.md)

---

## 判定所依据的规则

`Send` 与 `Sync` 是 **auto trait**（`core/src/marker.rs`）。判定按下列顺序：

1. 若类型有**显式负向 impl**（`impl !Send for X`），直接为否；
2. 否则若有**显式 unsafe impl**，按该 impl 的约束判定；
3. 否则**自动推导**：所有字段都 `Send` 则该类型 `Send`，所有字段都 `Sync` 则该类型 `Sync`。

两者的含义**不同**，这是本题集要测的核心：

| | 约束的是 | 直觉说法 |
|---|---------|---------|
| `Send` | 该类型的**值**可以被**移动**到另一个线程 | 「所有权换个线程持有，安全吗」 |
| `Sync` | `&T` 是 `Send`，即多个线程可以**同时持有引用** | 「两个线程同时读它，安全吗」 |

由此可推出一条常被忽略的等价关系：**`T: Sync` ⟺ `&T: Send`**。第 12 题考的就是它。

标准库中的关键 impl：

| 类型 | Send 条件 | Sync 条件 |
|------|----------|----------|
| `UnsafeCell<T>` | `T: Send` | **永不**（`impl<T: ?Sized> !Sync`） |
| `Cell<T>` / `RefCell<T>` | `T: Send` | 永不（内含 `UnsafeCell`） |
| `Rc<T>` | **永不**（显式 `!Send`） | **永不**（显式 `!Sync`） |
| `Arc<T>` | `T: Send + Sync` | `T: Send + Sync` |
| `Mutex<T>` | `T: Send` | **`T: Send`**（注意不是 `T: Sync`） |
| `RwLock<T>` | `T: Send` | `T: Send + Sync` |
| `MutexGuard<'_, T>` | **永不**（显式 `!Send`） | `T: Sync` |
| `*const T` / `*mut T` | **永不** | **永不** |
| `PhantomData<T>` | `T: Send` | `T: Sync` |
| `&'a T` | `T: Sync` | `T: Sync` |
| `&'a mut T` | `T: Send` | `T: Sync` |
| 原子类型 | 是 | 是 |

---

## 逐题答案

| # | 类型 | Send | Sync |
|---|-----|------|------|
| 1 | `Tick` | ✅ | ✅ |
| 2 | `Slot` | ✅ | ❌ |
| 3 | `Shared` | ❌ | ❌ |
| 4 | `RawSlot` | ❌ | ❌ |
| 5 | `Guarded` | ✅ | ✅ |
| 6 | `SharedMut` | ❌ | ❌ |
| 7 | `Held<'a>` | ❌ | ✅ |
| 8 | `Marked` | ❌ | ❌ |
| 9 | `Ticker` | ✅ | ✅ |
| 10 | `Callback` | ✅ | ❌ |
| 11 | `RawCell` | ✅ | ❌ |
| 12 | `Peek<'a>` | ❌ | ❌ |

---

### 1. `Tick { count: u64 }` — Send ✅ / Sync ✅

唯一字段 `u64` 两者皆是，自动推导直接通过。基线题：确认你知道**默认**情况长什么样，
后面 11 题全部是"某个字段把它拉了下来"。

### 2. `Slot { value: Cell<u32> }` — Send ✅ / Sync ❌

`Cell<T>` 内含 `UnsafeCell<T>`，而 `UnsafeCell` 有显式的 `!Sync`。所以 `Slot` 不是 `Sync`。

但它**是** `Send`：把整个 `Slot` 移动到另一个线程完全安全 —— 移动之后原线程再也碰不到它，
不存在两个线程同时改同一个 `Cell` 的问题。

这一题是 US4 AS1 的直接对应：**"含内部可变性" ≠ "不能跨线程"**。
不能的是**共享**，不是**移动**。

### 3. `Shared { handle: Rc<u32> }` — Send ❌ / Sync ❌

`Rc` 有显式的 `impl !Send` 与 `impl !Sync`。

原因是引用计数用的是**普通** `Cell<usize>` 而非原子类型。两个线程同时 clone/drop 同一个
`Rc`，计数就会丢更新 —— 轻则内存泄漏，重则提前释放导致 use-after-free。

注意 `!Send` 是必需的，光有 `!Sync` 不够：即使不共享引用，把一个 `Rc` **移动**走、
另一个克隆留在原线程，两个线程仍会同时改同一个计数器。

### 4. `RawSlot { ptr: *mut u8, len: usize }` — Send ❌ / Sync ❌

裸指针两者皆非。这**不是**因为解引用裸指针不安全 —— 而是因为编译器对指针指向的东西
一无所知，无法推断跨线程使用是否安全，于是保守地拒绝，把判断权交还给你。

要覆盖它就得手写 `unsafe impl Send for RawSlot {}`，那句 `unsafe` 的含义正是
"我来为跨线程安全性负责"。这是 C-15 的 Safety Invariant 在 C-12 上的预演。

### 5. `Guarded { inner: Mutex<Cell<u32>> }` — Send ✅ / Sync ✅

**本题集最容易错的一题。**

`Mutex<T>` 的 `Sync` 条件是 **`T: Send`**，不是 `T: Sync`：

```rust
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
```

`Cell<u32>` 是 `Send`（见第 2 题），所以 `Mutex<Cell<u32>>` **既 Send 又 Sync**。

为什么条件可以放宽到 `T: Send`：互斥锁保证任一时刻**最多一个线程**能拿到 `&mut T`。
既然不存在并发访问，`T` 自己是否支持并发访问（`Sync`）就无关紧要了，
只要它能被**移交**给拿到锁的那个线程（`Send`）即可。

一句话：`Mutex` 把 `Send` **升级**成了 `Sync`。这正是"用锁换取共享能力"的类型级表述。

### 6. `SharedMut { inner: Arc<RefCell<u32>> }` — Send ❌ / Sync ❌

与第 5 题成对，考的是"不是所有容器都会提升能力"。

`Arc<T>` 两个方向都要求 **`T: Send + Sync`**：

```rust
unsafe impl<T: ?Sized + Sync + Send> Send for Arc<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for Arc<T> {}
```

`RefCell<u32>` 是 `Send` 但**不是** `Sync`，条件不满足 → 两者皆否。

为什么 `Arc` 不能像 `Mutex` 那样放宽：`Arc` 只保证**引用计数**是原子的，
它对内部数据不提供任何互斥。多个线程可以同时拿到 `&RefCell<u32>` 并各自 `borrow_mut()`，
`RefCell` 的 `BorrowFlag` 是非原子的，检查本身就会竞争。

`Arc<RefCell<T>>` 是新手最常写错的组合，正确写法是 `Arc<Mutex<T>>`。

### 7. `Held<'a> { guard: MutexGuard<'a, u32> }` — Send ❌ / Sync ✅

唯一一题 `Sync` 成立而 `Send` 不成立 —— 专门用来打破"Sync 比 Send 强"的错觉。

```rust
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}
```

`!Send` 的原因是平台约束：POSIX 要求 `pthread_mutex_unlock` 必须由**加锁的同一线程**调用。
guard 的 `Drop` 会解锁，若 guard 被移动到别的线程再析构，解锁就发生在错误的线程上。

`Sync` 却成立：`&MutexGuard<'_, u32>` 只能用来**读** `u32`，不能触发 `Drop`，
多个线程同时持有这种引用没有危险。

两者独立，不存在谁蕴含谁。

### 8. `Marked { id: u32, _marker: PhantomData<*const u8> }` — Send ❌ / Sync ❌

`PhantomData<T>` 在 auto trait 推导中的行为与 `T` **完全一致**：
`Send` iff `T: Send`，`Sync` iff `T: Sync`。`*const u8` 两者皆非，于是把整个结构体拉下水。

关键点：`PhantomData` **大小为 0**（`size_of::<PhantomData<T>>() == 0`），
运行期不占任何空间，却照样参与类型级推导。

"占不占内存"和"参不参与 auto trait 推导"是两件不相干的事 ——
这正是 `PhantomData` 存在的意义：**零成本地表达类型级约束**。
用 `PhantomData<*const ()>` 让类型 `!Send` 是标准库里常见的手法。

### 9. `Ticker { hits: AtomicUsize }` — Send ✅ / Sync ✅

原子类型两者皆是。`AtomicUsize` 内部也是 `UnsafeCell`，但它有显式的
`unsafe impl Sync`，因为所有操作都通过原子指令完成，并发访问不会撕裂。

与第 2 题对照：同样含 `UnsafeCell`，`Cell` 不是 `Sync` 而 `AtomicUsize` 是。
差别不在"有没有内部可变性"，而在**修改是否原子**。

### 10. `Callback { f: Box<dyn Fn() + Send> }` — Send ✅ / Sync ❌

`Box<T>` 原样传递：`Send` iff `T: Send`，`Sync` iff `T: Sync`。

trait object `dyn Fn() + Send` 的 auto trait 集合就是**写在类型里的那些**。
这里只写了 `+ Send`，所以它是 `Send`、不是 `Sync`。

这一题考的是：trait object 的 auto trait 不靠推导，而是**类型标注的一部分**。
想要两者都有，得写 `Box<dyn Fn() + Send + Sync>`。

### 11. `RawCell { inner: UnsafeCell<u32> }` — Send ✅ / Sync ❌

第 2 题的"去皮"版本。`Cell` 的行为完全来自它内部的 `UnsafeCell`：

```rust
impl<T: ?Sized> !Sync for UnsafeCell<T> {}
unsafe impl<T: ?Sized + Send> Send for UnsafeCell<T> {}   // 经由自动推导
```

`UnsafeCell` 是**整个语言里唯一**能合法地从 `&T` 得到 `&mut T` 的类型，
所有内部可变性（`Cell` / `RefCell` / `Mutex` / 原子类型）都建立在它之上。
它默认 `!Sync`，需要并发安全的类型（如 `Mutex`、`AtomicUsize`）再自己
`unsafe impl Sync` 把它加回来 —— 加回来的同时就承担了证明义务。

### 12. `Peek<'a> { view: &'a Cell<u32> }` — Send ❌ / Sync ❌

考 `&T` 的推导规则，两个方向都由 **`T: Sync`** 决定：

```rust
unsafe impl<T: Sync + ?Sized> Send for &T {}
unsafe impl<T: Sync + ?Sized> Sync for &T {}
```

`Cell<u32>` 不是 `Sync`，所以 `&Cell<u32>` 两者皆否。

`&T: Send` 要求 `T: Sync` 而不是 `T: Send`，这条最反直觉。理由是：
把 `&T` 送到另一个线程，就等于让两个线程**同时持有** `T` 的引用 ——
那正是 `Sync` 的定义。这也就是那条等价关系 **`T: Sync` ⟺ `&T: Send`**。

顺带对照：`&mut T: Send` 要求的是 `T: Send`，因为 `&mut` 是独占的，
送过去之后原线程访问不到，不构成共享。

---

## 评分

- 每题 `Send` 与 `Sync` **都**答对才算该题正确。
- 全卷错题数 ≤ 1 → **pass**；≥ 2 → **fail**，按 §Remediation 登记补齐任务。
- 判定对但**没写推导依据**的题，计为错题（规则 F5 第 2 条）。

## 最容易错的三题

如果错，多半错在这里 —— 它们各自打破一个常见的错觉：

| 题 | 打破的错觉 |
|---|-----------|
| 5 `Mutex<Cell<u32>>` | "容器原样传递内部类型的能力"——`Mutex` 会把 `Send` 提升为 `Sync` |
| 7 `MutexGuard` | "Sync 比 Send 强"——两者独立，这题 Sync 成立而 Send 不成立 |
| 12 `&Cell<u32>` | "`&T: Send` 取决于 `T: Send`"——实际取决于 `T: Sync` |
