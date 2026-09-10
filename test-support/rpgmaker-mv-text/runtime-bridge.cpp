// Experimental x86 / Node 9.7.1 bridge for the authored MV fixture only.
// Runtime may load this DLL first. Registration occurs only during Node's
// own module load, never in a DLL constructor. It stays resident until exit.
#define WIN32_LEAN_AND_MEAN
#define BUILDING_NODE_EXTENSION
#define EXTERNAL_NAPI
#include <windows.h>
#include <node_api.h>
#include <mutex>
#include <vector>
#include <cstring>
#include "../../crates/adapters/implementations/framework/monogame-native/native/adapter_abi.h"

struct TextEvent {
    uint32_t size, kind;
    uint64_t surface, run, epoch;
    uint32_t ordinal;
    const uint16_t* source;
    uint32_t length;
};
struct TextHost {
    uint32_t size, version;
    void* context;
    Decision (__cdecl *decide)(void*, const TextEvent*, uint16_t*, uint32_t, uint16_t*, uint32_t);
    uint64_t (__cdecl *enter)(void*);
    void (__cdecl *leave)(void*, uint64_t);
};
static_assert(sizeof(void*) == 4, "Verified x86 fixture only");
static_assert(sizeof(TextEvent) == 48);
static_assert(sizeof(TextHost) == 24);
static std::mutex gate;
static TextHost textHost{};
static bool active = false;
static uint64_t features = 0;
static bool nodeReady = false;
#ifdef GLYPHSHIFT_MV_MANAGED_BOOTSTRAP
bool startEngine() noexcept;
bool stopEngine() noexcept;
#endif

// Resolve against the runtime already in this process, with no import library
// and no second copy of Node. Every API is checked before module registration.
#define NAPI_FUNCTIONS(X) \
    X(napi_module_register) X(napi_get_undefined) X(napi_create_function) \
    X(napi_set_named_property) X(napi_get_cb_info) X(napi_get_value_string_utf16) \
    X(napi_get_value_uint32) X(napi_create_string_utf16)
#define DECLARE(name) static decltype(&name) p_##name = nullptr;
NAPI_FUNCTIONS(DECLARE)
#undef DECLARE

static napi_value resolve(napi_env env, napi_callback_info info) noexcept {
    napi_value result = nullptr;
    if (p_napi_get_undefined(env, &result) != napi_ok) return nullptr;
    try {
        size_t argc = 2;
        napi_value argv[2]{};
        if (p_napi_get_cb_info(env, info, &argc, argv, nullptr, nullptr) != napi_ok || argc != 2) return result;
        uint32_t kind = 0;
        size_t length = 0;
        if (p_napi_get_value_uint32(env, argv[1], &kind) != napi_ok || (kind != 1 && kind != 3)) return result;
        if (p_napi_get_value_string_utf16(env, argv[0], nullptr, 0, &length) != napi_ok || !length || length > 16384) return result;
        std::vector<char16_t> source(length + 1);
        size_t copied = 0;
        if (p_napi_get_value_string_utf16(env, argv[0], source.data(), source.size(), &copied) != napi_ok || copied != length) return result;
        std::vector<uint16_t> output(16384);
        Decision decision{};
        {
            std::lock_guard<std::mutex> lock(gate);
            if (!active || !textHost.decide) return result;
            const TextEvent event{sizeof(TextEvent), kind, 0, 0, 0, 0,
                reinterpret_cast<const uint16_t*>(source.data()), static_cast<uint32_t>(length)};
            decision = textHost.decide(textHost.context, &event, output.data(),
                static_cast<uint32_t>(output.size()), nullptr, 0);
            if (!(features & 2)) return result;
        }
        if (kind == 1 && decision.status == 0 && (decision.bits & 1) &&
            decision.text_len > 0 && decision.text_len <= output.size()) {
            napi_value translated = nullptr;
            if (p_napi_create_string_utf16(env, reinterpret_cast<const char16_t*>(output.data()),
                decision.text_len, &translated) == napi_ok) return translated;
        }
    } catch (...) { /* Keep the game's original text on bridge failures. */ }
    return result;
}

static napi_value initialize(napi_env env, napi_value exports) {
    napi_value function = nullptr;
    if (p_napi_create_function(env, "resolve", NAPI_AUTO_LENGTH, resolve, nullptr, &function) != napi_ok) return nullptr;
    if (p_napi_set_named_property(env, exports, "resolve", function) != napi_ok) return nullptr;
    std::lock_guard<std::mutex> lock(gate);
    nodeReady = true;
    return exports;
}
static napi_module module{1, 0, "runtime-bridge.cpp", initialize, "glyphshift_mv_fixture", nullptr, {nullptr}};
// Called by the experimental entry's narrowly filtered uv_dlopen return hook.
// Calling this before process.dlopen would violate Node's pending-module check.
extern "C" __declspec(dllexport) int32_t __cdecl glyphshift_mv_register_node_v1() {
        std::lock_guard<std::mutex> lock(gate);
        if (nodeReady) return 1;
        const auto node = GetModuleHandleW(L"node.dll");
        if (!node) return 2;
#define LOAD(name) p_##name = reinterpret_cast<decltype(p_##name)>(GetProcAddress(node, #name)); if (!p_##name) return 2;
#pragma warning(push)
#pragma warning(disable:4191)
        NAPI_FUNCTIONS(LOAD)
#pragma warning(pop)
#undef LOAD
        p_napi_module_register(&module);
        return 0;
}

static Negotiation __cdecl negotiate(uint64_t requested, uint64_t granted) {
    if (requested & ~uint64_t{3}) return {1, 0};
    if (requested & ~granted) return {2, 0};
    return {0, requested};
}
static Negotiation __cdecl activate(const Host* host, uint64_t requested, uint64_t granted) {
    const auto result = negotiate(requested, granted);
    if (result.status) return result;
#ifdef GLYPHSHIFT_MV_MANAGED_BOOTSTRAP
    // Never hold the callback gate while waiting for the game thread to load
    // Node exports, which also needs this gate.
    if (!startEngine()) return {3, 0};
#endif
    std::lock_guard<std::mutex> lock(gate);
    if (!nodeReady) return {3, 0};
    if (!host || host->size != sizeof(Host) || !host->decide || !textHost.decide || textHost.context != host->context) return {4, 0};
    features = result.active;
    active = true;
    return result;
}
static int32_t __cdecl deactivate() {
    {
        std::lock_guard<std::mutex> lock(gate);
        active = false;
        features = 0;
    }
#ifdef GLYPHSHIFT_MV_MANAGED_BOOTSTRAP
    if (!stopEngine()) return 3;
#endif
    return 0;
}
extern "C" __declspec(dllexport) int32_t __cdecl glyphshift_adapter_bind_text_host_v1(const TextHost* host) {
    std::lock_guard<std::mutex> lock(gate);
    if (!host) { textHost = {}; return 0; }
    if (host->size != sizeof(TextHost) || host->version != 1 || !host->decide || (!!host->enter != !!host->leave)) return 4;
    textHost = *host;
    return 0;
}
static void __cdecl refresh() { /* Next-message replacement only. */ }
static Api makeApi() {
    Api api{};
    api.size = sizeof(Api);
    auto& d = api.descriptor;
    d.size = sizeof(Descriptor);
    const char id[] = "fixture.rpgmaker-mv.runtime-bridge";
    d.id.len = sizeof(id) - 1;
    std::memcpy(d.id.bytes, id, d.id.len);
    d.major = d.abi_major = 1;
    d.apply_model = d.placement = 1;
    d.features = 3;
    d.platforms = d.architectures = 1;
    api.negotiate = negotiate;
    api.activate = activate;
    api.deactivate = deactivate;
    api.refresh = refresh;
    return api;
}
extern "C" __declspec(dllexport) Api __cdecl glyphshift_adapter_entry_v1() {
    static const Api api = makeApi();
    return api;
}
