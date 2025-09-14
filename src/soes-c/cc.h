#ifndef CC_STM32_H
#define CC_STM32_H

#ifdef __cplusplus
extern "C"
{
#endif
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <tinyprintf.h>

    // ----------------------
    // Basic types (stdint.h)
    // ----------------------
    // typedef unsigned char uint8_t;
    // typedef unsigned short uint16_t;
    // typedef unsigned int uint32_t;
    // typedef signed char int8_t;
    // typedef signed short int16_t;
    // typedef signed int int32_t;

    // ----------------------
    // Boolean type (stdbool.h)
    // ----------------------
    // typedef uint8_t bool;
#define true 1
#define false 0

// ----------------------
// Standard definitions (stddef.h)
// ----------------------
#define NULL ((void *)0)
    // typedef uint32_t size_t;

// ----------------------
// Helpers
// ----------------------
#ifndef MIN
#define MIN(a, b) (((a) < (b)) ? (a) : (b))
#endif

#ifndef MAX
#define MAX(a, b) (((a) > (b)) ? (a) : (b))
#endif

#define CC_PACKED_BEGIN
#define CC_PACKED_END
#define CC_PACKED __attribute__((packed))

#ifdef __rtk__
#define CC_ASSERT(exp) ASSERT(exp)
#else
#define CC_ASSERT(exp) // stub, optional
#endif
#define CC_STATIC_ASSERT(exp) _Static_assert(exp, "")

#define CC_DEPRECATED __attribute__((deprecated))

#define CC_SWAP32(x) __builtin_bswap32(x)
#define CC_SWAP16(x) __builtin_bswap16(x)

#define CC_ATOMIC_SET(var, val) __atomic_store_n(&var, val, __ATOMIC_SEQ_CST)
#define CC_ATOMIC_GET(var) __atomic_load_n(&var, __ATOMIC_SEQ_CST)
#define CC_ATOMIC_ADD(var, val) __atomic_add_fetch(&var, val, __ATOMIC_SEQ_CST)
#define CC_ATOMIC_SUB(var, val) __atomic_sub_fetch(&var, val, __ATOMIC_SEQ_CST)
#define CC_ATOMIC_AND(var, val) __atomic_and_fetch(&var, val, __ATOMIC_SEQ_CST)
#define CC_ATOMIC_OR(var, val) __atomic_or_fetch(&var, val, __ATOMIC_SEQ_CST)

// #if BYTE_ORDER == BIG_ENDIAN
// #define htoes(x) CC_SWAP16(x)
// #define htoel(x) CC_SWAP32(x)
// #else
#define htoes(x) (x)
#define htoel(x) (x)
    // #endif

#define etohs(x) htoes(x)
#define etohl(x) htoel(x)

    extern void DPRINT_RUST(const char *msg);

#define DPRINT(msg, ...)                                \
    do                                                  \
    {                                                   \
        char buf[128];                                  \
        snprintf(buf, sizeof(buf), msg, ##__VA_ARGS__); \
        DPRINT_RUST(buf);                               \
    } while (0)

#ifdef __cplusplus
}
#endif

#endif /* CC_STM32_H */
