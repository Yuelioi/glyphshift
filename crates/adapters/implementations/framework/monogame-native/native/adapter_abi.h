#pragma once
#include <cstdint>
#include <cstddef>

// Mirrors native-abi V1; checked by the authoritative Rust loader contract.
// The managed harness checks sizes and invokes the actual function pointers.
struct FixedUtf8 { uint16_t len; uint8_t bytes[96]; };
struct Descriptor {
    uint32_t size;
    FixedUtf8 id;
    uint16_t major, minor, patch, abi_major, abi_minor;
    uint32_t apply_model, placement;
    uint64_t features;
    uint32_t platforms, architectures;
};
struct Negotiation { int32_t status; uint64_t active; };
struct Decision {
    int32_t status;
    uint64_t generation;
    uint32_t bits, text_len, font_len;
};
using Decide = Decision (__cdecl *)(void*, const uint16_t*, uint32_t,
    uint16_t*, uint32_t, uint16_t*, uint32_t);
struct Host {
    uint32_t size;
    void* context;
    Decide decide;
    uint32_t (__cdecl *characters)(void*, uint16_t*, uint32_t);
};
struct Api {
    uint32_t size;
    Descriptor descriptor;
    Negotiation (__cdecl *negotiate)(uint64_t, uint64_t);
    Negotiation (__cdecl *activate)(const Host*, uint64_t, uint64_t);
    int32_t (__cdecl *deactivate)();
    void (__cdecl *refresh)();
};
static_assert(sizeof(FixedUtf8) == 98);
static_assert(sizeof(Descriptor) == 136);
static_assert(sizeof(Decision) == 32);
static_assert(sizeof(Host) == (sizeof(void*) == 8 ? 32 : 16));
static_assert(sizeof(Api) == (sizeof(void*) == 8 ? 176 : 160));
