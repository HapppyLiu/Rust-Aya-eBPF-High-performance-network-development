/* C-21 配套：被 Rust 调用，也回调 Rust 导出的符号。
 *
 * 布局函数返回 sizeof / alignof / offsetof，供双侧断言，
 * 禁止只靠"我声明了 repr(C)"认定一致。
 */

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

struct PacketHdr {
    uint8_t kind;
    uint32_t id;
    uint16_t len;
};

/* 由 Rust `#[unsafe(no_mangle)] extern "C"` 提供。 */
int32_t m6_rust_mul(int32_t a, int32_t b);
int32_t m6_rust_hdr_sum(const struct PacketHdr *h);

size_t m6_c_hdr_size(void) { return sizeof(struct PacketHdr); }
size_t m6_c_hdr_align(void) { return _Alignof(struct PacketHdr); }
size_t m6_c_hdr_off_kind(void) { return offsetof(struct PacketHdr, kind); }
size_t m6_c_hdr_off_id(void) { return offsetof(struct PacketHdr, id); }
size_t m6_c_hdr_off_len(void) { return offsetof(struct PacketHdr, len); }

int32_t m6_c_add(int32_t a, int32_t b) { return a + b; }

int32_t m6_c_call_rust_mul(int32_t a, int32_t b) { return m6_rust_mul(a, b); }

int32_t m6_c_hdr_sum(const struct PacketHdr *h) {
    if (h == NULL) {
        return 0;
    }
    return (int32_t)h->kind + (int32_t)h->id + (int32_t)h->len;
}

void m6_c_fill_hdr(struct PacketHdr *out) {
    if (out == NULL) {
        return;
    }
    out->kind = 7;
    out->id = 0x01020304u;
    out->len = 42;
}

int32_t m6_c_call_rust_hdr_sum(void) {
    struct PacketHdr h;
    h.kind = 1;
    h.id = 2;
    h.len = 3;
    return m6_rust_hdr_sum(&h);
}

/* 约定：C malloc，Rust libc::free。字面量与 Rust 侧 OWNED_MSG 对齐。 */
char *m6_c_alloc_message(void) {
    static const char src[] = "m6-owned-by-rust";
    size_t n = sizeof(src); /* 含 NUL */
    char *p = (char *)malloc(n);
    if (p == NULL) {
        return NULL;
    }
    memcpy(p, src, n);
    return p;
}
