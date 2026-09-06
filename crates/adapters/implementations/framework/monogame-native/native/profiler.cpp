#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <cor.h>
#include <corprof.h>
#include <atomic>
#include <algorithm>
#include <cstring>
#include <mutex>
#include <new>
#include <shared_mutex>
#include <string>
#include <thread>
#include <vector>
#include "callback_base.h"
#include "adapter_abi.h"
#include "embedded_fallback.h"

// A Native Adapter and CLR profiler share this verified library and callback state.
static const GUID profiler_id = {0x903d5c72,0x54b7,0x4d31,{0x95,0x9d,0xc8,0x56,0xf2,0x6f,0xa2,0x29}};
static ICorProfilerInfo10* info = nullptr;
static ModuleID target_module = 0;
static mdMethodDef bridge_method = 0;
static mdMemberRef string_length = 0;
struct MethodTarget {
    mdMethodDef method = 0;
    std::vector<BYTE> original;
    BYTE text_argument = 0;
    int font_argument = -1;
    bool builder = false;
    bool frame_boundary = false;
};
static std::vector<MethodTarget> targets;
static bool monogame_profile = false;
static std::wstring target_module_filename = L"Glyphshift.CoreClr.SyntheticHost.dll";
static mdMethodDef font_bridge = 0, builder_bridge = 0;
static mdMethodDef fallback_resolver = 0;
static std::atomic<uint32_t> compiled_methods{0};
static std::atomic<uint32_t> compiled_mask{0};
static std::atomic<bool> preparation_done{false};
static std::atomic<bool> decisions_enabled{false};
static std::mutex lifecycle_lock;
static std::atomic<HRESULT> last_error{S_OK};
static std::atomic<bool> patched{false};
static std::atomic<bool> reverting{false};
static std::shared_mutex host_lock;
static Host host{};
static uint64_t active_features = 0;
static HMODULE own_module = nullptr;
static thread_local bool inside_decision = false;

#define CHECK(call) do { HRESULT checked_result = (call); if (FAILED(checked_result)) { last_error.store(checked_result); return checked_result; } } while (false)

extern "C" __declspec(dllexport) BYTE __cdecl glyphshift_fallback_byte(int32_t index) {
    return index >= 0 && static_cast<size_t>(index) < sizeof(fallback_assembly) ? fallback_assembly[index] : 0;
}

#include "monogame_profile.h"
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
#include "self_attach.h"
#endif

extern "C" __declspec(dllexport) wchar_t* __cdecl glyphshift_coreclr_translate(const wchar_t* source, int32_t supplied_length) noexcept {
    if (!source || supplied_length < 0 || supplied_length > 65536) return nullptr;
    if (!decisions_enabled.load(std::memory_order_acquire)) return nullptr;
    size_t length = static_cast<size_t>(supplied_length);
    // Only bounded, null-terminated UTF-16 strings are supported by this fixture.
    if (wcsnlen_s(source, length + 1) != length) return nullptr;
    const wchar_t* chosen = nullptr;
    uint32_t chosen_length = static_cast<uint32_t>(length);
    size_t leading = 0, trailing = 0;
    try {
        std::vector<uint16_t> translated(65536);
        if (!inside_decision) {
            std::shared_lock guard(host_lock);
            if (host.decide && active_features && decisions_enabled.load(std::memory_order_acquire)) {
                inside_decision = true;
                auto decision = host.decide(host.context, reinterpret_cast<const uint16_t*>(source),
                    chosen_length, translated.data(), static_cast<uint32_t>(translated.size()), nullptr, 0);
                inside_decision = false;
                // Capture catalogs trim the edges of observed text. MonoGame
                // layout helpers can add edge padding before DrawString. Keep
                // exact keys authoritative, then try the unpadded lookup and
                // preserve the caller's original padding around its translation.
                if ((active_features & 2) && decision.status == 0 && !(decision.bits & 1)) {
                    auto padding = [](wchar_t value) { return value == L' ' || value == L'\t' || value == L'\r' || value == L'\n'; };
                    size_t start = 0, end = length;
                    while (start < end && padding(source[start])) ++start;
                    while (end > start && padding(source[end - 1])) --end;
                    if (end > start && (start != 0 || end != length)) {
                        inside_decision = true;
                        decision = host.decide(host.context, reinterpret_cast<const uint16_t*>(source + start),
                            static_cast<uint32_t>(end - start), translated.data(), static_cast<uint32_t>(translated.size()), nullptr, 0);
                        inside_decision = false;
                        leading = start; trailing = length - end;
                    }
                }
                if ((active_features & 2) && decision.status == 0 && (decision.bits & 1)
                    && decision.text_len <= translated.size()) {
                    chosen = reinterpret_cast<const wchar_t*>(translated.data());
                    chosen_length = decision.text_len;
                    if (std::find(translated.begin(), translated.begin() + chosen_length, 0) != translated.begin() + chosen_length) return nullptr;
                }
            }
        }
        if (!chosen) return nullptr;
        size_t total = leading + chosen_length + trailing;
        if (total > 65536) return nullptr;
        auto result = static_cast<wchar_t*>(CoTaskMemAlloc((total + 1) * sizeof(wchar_t)));
        if (!result) return nullptr;
        memcpy(result, source, leading * sizeof(wchar_t));
        memcpy(result + leading, chosen, chosen_length * sizeof(wchar_t));
        memcpy(result + leading + chosen_length, source + length - trailing, trailing * sizeof(wchar_t));
        result[total] = 0;
        return result;
    } catch (...) {
        inside_decision = false;
        return nullptr;
    }
}

static HRESULT prepare_module() {
    ICorProfilerModuleEnum* modules = nullptr;
    CHECK(info->EnumModules(&modules));
    ModuleID module;
    ULONG fetched;
    bool fixture_present = false;
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
    fixture_present = true;
#endif
    while (modules->Next(1, &module, &fetched) == S_OK && fetched) {
        wchar_t name[32768];
        ULONG used;
        LPCBYTE address;
        AssemblyID assembly;
        if (FAILED(info->GetModuleInfo(module, &address, 32768, &used, name, &assembly))) continue;
        std::wstring path(name);
        auto slash = path.find_last_of(L"\\/");
        auto filename = path.substr(slash == std::wstring::npos ? 0 : slash + 1);
        if (filename == target_module_filename) fixture_present = true;
        if (filename == (monogame_profile ? L"MonoGame.Framework.dll" : target_module_filename)) {
            target_module = module;
            if (!monogame_profile) { fixture_present = true; break; }
        }
    }
    modules->Release();
    if (!target_module || !fixture_present) return E_INVALIDARG;
    IMetaDataImport* metadata = nullptr;
    CHECK(info->GetModuleMetaData(target_module, ofRead | ofWrite, IID_IMetaDataImport,
        reinterpret_cast<IUnknown**>(&metadata)));
    HRESULT result;
    if (monogame_profile) result = find_monogame_targets(metadata);
    else {
        mdTypeDef renderer;
        result = metadata->FindTypeDefByName(L"SyntheticRenderer", mdTokenNil, &renderer);
        const BYTE signature[] = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 1, ELEMENT_TYPE_STRING, ELEMENT_TYPE_STRING};
        mdMethodDef method = 0;
        if (SUCCEEDED(result)) result = metadata->FindMethod(renderer, L"Render", signature, sizeof(signature), &method);
        if (SUCCEEDED(result)) targets.push_back({method, {}, 0, -1, false});
    }
    mdTypeRef object_ref = 0, string_ref = 0;
    HCORENUM type_enum = nullptr;
    mdTypeRef type_ref;
    while (metadata->EnumTypeRefs(&type_enum, &type_ref, 1, &fetched) == S_OK && fetched) {
        wchar_t type_name[256]; ULONG type_length; mdToken scope;
        if (SUCCEEDED(metadata->GetTypeRefProps(type_ref, &scope, type_name, 256, &type_length))) {
            if (wcscmp(type_name, L"System.Object") == 0) object_ref = type_ref;
            if (wcscmp(type_name, L"System.String") == 0) string_ref = type_ref;
        }
    }
    metadata->CloseEnum(type_enum);
    if (!object_ref || !string_ref) result = E_FAIL;
    IMetaDataEmit* emit = nullptr;
    if (SUCCEEDED(result)) result = metadata->QueryInterface(IID_IMetaDataEmit, reinterpret_cast<void**>(&emit));
    metadata->Release();
    if (FAILED(result)) return result;
    mdModuleRef native_module = 0;
    wchar_t native_path[32768];
    if (!GetModuleFileNameW(own_module, native_path, 32768)) { emit->Release(); return E_FAIL; }
    result = emit->DefineModuleRef(native_path, &native_module);
    // Existing loaded types cannot acquire methods just by appending metadata.
    // Define a new type so its first CLR load sees the complete P/Invoke method.
    mdTypeDef bridge_type = 0;
    if (SUCCEEDED(result)) result = emit->DefineTypeDef(L"Glyphshift.SyntheticBridge", tdPublic | tdAbstract | tdSealed,
        object_ref, nullptr, &bridge_type);
    const BYTE bridge_signature[] = {IMAGE_CEE_CS_CALLCONV_DEFAULT, 2, ELEMENT_TYPE_STRING, ELEMENT_TYPE_STRING, ELEMENT_TYPE_I4};
    if (SUCCEEDED(result)) result = emit->DefineMethod(bridge_type, L"Translate",
        mdPublic | mdStatic | mdPinvokeImpl, bridge_signature, sizeof(bridge_signature), 0, miPreserveSig, &bridge_method);
    const BYTE length_signature[] = {IMAGE_CEE_CS_CALLCONV_HASTHIS, 0, ELEMENT_TYPE_I4};
    if (SUCCEEDED(result)) result = emit->DefineMemberRef(string_ref, L"get_Length", length_signature, sizeof(length_signature), &string_length);
    if (SUCCEEDED(result)) result = emit->DefinePinvokeMap(bridge_method,
        pmCallConvCdecl | pmCharSetUnicode | pmNoMangle, L"glyphshift_coreclr_translate", native_module);
    const BYTE marshal = NATIVE_TYPE_LPWSTR;
    for (ULONG sequence = 0; sequence <= 1 && SUCCEEDED(result); ++sequence) {
        mdParamDef parameter;
        result = emit->DefineParam(bridge_method, sequence, sequence ? L"source" : L"result", 0,
            ELEMENT_TYPE_VOID, nullptr, 0, &parameter);
        if (SUCCEEDED(result)) result = emit->SetFieldMarshal(parameter, &marshal, 1);
    }
    if (SUCCEEDED(result) && monogame_profile) result = prepare_font_helpers(emit, bridge_type, string_ref, native_module);
    emit->Release();
    if (FAILED(result)) return result;
    CHECK(info->ApplyMetaData(target_module));
    for (auto& target : targets) {
        LPCBYTE body; ULONG size;
        CHECK(info->GetILFunctionBody(target_module, target.method, &body, &size));
        target.original.assign(body, body + size);
    }
    return S_OK;
}

static HRESULT rewrite(const MethodTarget& target, ICorProfilerFunctionControl* control) {
    const auto& original = target.original;
    if (original.empty()) return E_FAIL;
    uint32_t code_size = 0, locals = 0;
    uint16_t flags = 0x3003, max_stack = 8;
    size_t header_size = 0;
    if ((original[0] & 3) == 2) {
        header_size = 1;
        code_size = original[0] >> 2;
    } else if ((original[0] & 3) == 3 && original.size() >= 12) {
        memcpy(&flags, original.data(), 2);
        if ((flags >> 12) != 3 || (flags & CorILMethod_MoreSects)) return E_NOTIMPL;
        header_size = 12;
        memcpy(&max_stack, original.data() + 2, 2);
        memcpy(&code_size, original.data() + 4, 4);
        memcpy(&locals, original.data() + 8, 4);
    } else return E_NOTIMPL;
    if (header_size + code_size > original.size()) return E_FAIL;
    // Keep null/unbounded/embedded-NUL input unchanged. A null bridge result is
    // a fail-open sentinel (including allocation failure), never a new argument.
    std::vector<BYTE> prefix;
    if (target.frame_boundary) {
        prefix = {0x02, 0x14, 0x17, 0x28};
        auto token = reinterpret_cast<const BYTE*>(&fallback_resolver);
        prefix.insert(prefix.end(), token, token + 4); prefix.push_back(0x26);
    } else if (target.font_argument >= 0) {
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
        prefix = {0x0f, static_cast<BYTE>(target.font_argument), 0x0f, target.text_argument,
            static_cast<BYTE>(target.font_argument == 1 ? 0x17 : 0x16), 0x28};
        const auto method = target.builder ? builder_bridge : font_bridge;
        auto token = reinterpret_cast<const BYTE*>(&method);
        prefix.insert(prefix.end(), token, token + 4);
#else
        prefix = {static_cast<BYTE>(0x02 + target.font_argument), static_cast<BYTE>(0x02 + target.text_argument), 0x28};
        const auto method = target.builder ? builder_bridge : font_bridge;
        auto token = reinterpret_cast<const BYTE*>(&method);
        prefix.insert(prefix.end(), token, token + 4);
        prefix.insert(prefix.end(), {0x10, target.text_argument});
#endif
    } else {
        prefix = {0x02, 0x2c, 0x00, 0x02, 0x02, 0x6f};
        auto length_token = reinterpret_cast<const BYTE*>(&string_length);
        prefix.insert(prefix.end(), length_token, length_token + 4);
        prefix.push_back(0x28);
        auto token = reinterpret_cast<const BYTE*>(&bridge_method);
        prefix.insert(prefix.end(), token, token + 4);
        prefix.insert(prefix.end(), {0x25, 0x2d, 0x02, 0x26, 0x02, 0x10, 0x00});
        prefix[2] = static_cast<BYTE>(prefix.size() - 3);
    }
    uint32_t total_code_size = code_size + static_cast<uint32_t>(prefix.size());
    max_stack = std::max<uint16_t>(max_stack, 3);
    std::vector<BYTE> body(12);
    memcpy(body.data(), &flags, 2);
    memcpy(body.data() + 2, &max_stack, 2);
    memcpy(body.data() + 4, &total_code_size, 4);
    memcpy(body.data() + 8, &locals, 4);
    body.insert(body.end(), prefix.begin(), prefix.end());
    body.insert(body.end(), original.begin() + header_size, original.begin() + header_size + code_size);
    return control->SetILFunctionBody(static_cast<ULONG>(body.size()), body.data());
}

class Profiler final : public CallbackBase {
    std::atomic<ULONG> refs{1};
public:
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void** output) override {
        if (!output) return E_POINTER;
        *output = nullptr;
        if (id == __uuidof(IUnknown) || id == __uuidof(ICorProfilerCallback)
            || id == __uuidof(ICorProfilerCallback2) || id == __uuidof(ICorProfilerCallback3)
            || id == __uuidof(ICorProfilerCallback4)) {
            *output = static_cast<ICorProfilerCallback4*>(this); AddRef(); return S_OK;
        }
        return E_NOINTERFACE;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return ++refs; }
    ULONG STDMETHODCALLTYPE Release() override { auto n = --refs; if (!n) delete this; return n; }
    HRESULT STDMETHODCALLTYPE Initialize(IUnknown*) override { return E_NOTIMPL; }
    HRESULT STDMETHODCALLTYPE InitializeForAttach(IUnknown* unknown, void* data, UINT length) override {
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
        const char expected[] = "glyphshift-monogame-v1";
        if (!data || length != sizeof(expected) || memcmp(data, expected, sizeof(expected))) return E_INVALIDARG;
        monogame_profile = true;
        HMODULE pinned;
        if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
            reinterpret_cast<LPCWSTR>(&glyphshift_coreclr_translate), &pinned)) return E_FAIL;
#else
        const char expected[] = "glyphshift-synthetic-only-v1";
        const char monogame[] = "glyphshift-monogame-synthetic-v1";
        if (data && length >= sizeof(monogame) - 1 && !memcmp(data, monogame, sizeof(monogame) - 1)) {
            monogame_profile = true;
            const char* payload = static_cast<const char*>(data) + sizeof(monogame) - 1;
            size_t payload_length = length - (sizeof(monogame) - 1);
            const char prefix[] = "|module=";
            if (payload_length >= sizeof(prefix) - 1 && !memcmp(payload, prefix, sizeof(prefix) - 1)) {
                std::string module(payload + sizeof(prefix) - 1, payload_length - (sizeof(prefix) - 1));
                while (!module.empty() && module.back() == '\0') module.pop_back();
                if (module.empty() || module.size() > 240 || module.find_first_of("\\/") != std::string::npos) return E_INVALIDARG;
                target_module_filename.assign(module.begin(), module.end());
            }
        }
        else if (length != sizeof(expected) || !data || memcmp(data, expected, sizeof(expected))) return E_INVALIDARG;
#endif
        CHECK(unknown->QueryInterface(__uuidof(ICorProfilerInfo10), reinterpret_cast<void**>(&info)));
        CHECK(info->SetEventMask2(COR_PRF_ENABLE_REJIT | COR_PRF_MONITOR_JIT_COMPILATION, 0));
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE ProfilerAttachComplete() override {
        // ReJIT must originate on a profiler-owned native thread, outside callbacks.
        try {
            std::thread([] {
                try {
                    HRESULT result = prepare_module();
                    if (SUCCEEDED(result)) {
                        std::vector<ModuleID> modules(targets.size(), target_module);
                        std::vector<mdMethodDef> methods;
                        for (const auto& target : targets) methods.push_back(target.method);
                        result = info->RequestReJITWithInliners(COR_PRF_REJIT_BLOCK_INLINING | COR_PRF_REJIT_INLINING_CALLBACKS,
                            static_cast<ULONG>(targets.size()), modules.data(), methods.data());
                    }
                    last_error.store(result);
                    preparation_done.store(true, std::memory_order_release);
                } catch (...) { last_error.store(E_FAIL); preparation_done.store(true); }
            }).detach();
        } catch (...) { last_error.store(E_FAIL); return E_FAIL; }
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE GetReJITParameters(ModuleID module, mdMethodDef method, ICorProfilerFunctionControl* control) override {
        if (module != target_module || reverting.load()) return S_OK;
        auto target = std::find_if(targets.begin(), targets.end(), [&](const auto& entry) { return entry.method == method; });
        if (target == targets.end()) return S_OK;
        HRESULT result;
        try { result = rewrite(*target, control); } catch (...) { result = E_FAIL; }
        if (FAILED(result)) last_error.store(result);
        return result;
    }
    HRESULT STDMETHODCALLTYPE ReJITCompilationFinished(FunctionID function, ReJITID, HRESULT result, BOOL) override {
        if (FAILED(result)) last_error.store(result);
        else {
            ClassID type; ModuleID module; mdToken method;
            if (SUCCEEDED(info->GetFunctionInfo(function, &type, &module, &method)) && module == target_module) {
                for (size_t index = 0; index < targets.size(); ++index) if (targets[index].method == method) {
                    compiled_mask.fetch_or(1u << index);
                    compiled_methods.fetch_add(1);
                }
                if (compiled_mask.load() == ((1u << targets.size()) - 1)) patched.store(true);
            }
        }
        return S_OK;
    }
    HRESULT STDMETHODCALLTYPE ReJITError(ModuleID, mdMethodDef, FunctionID, HRESULT result) override {
        last_error.store(result); return S_OK;
    }
    HRESULT STDMETHODCALLTYPE JITInlining(FunctionID, FunctionID, BOOL* should_inline) override {
        if (should_inline) *should_inline = TRUE;
        return S_OK;
    }
};

class Factory final : public IClassFactory {
    std::atomic<ULONG> refs{1};
public:
    HRESULT STDMETHODCALLTYPE QueryInterface(REFIID id, void** output) override {
        if (!output) return E_POINTER;
        *output = nullptr;
        if (id != __uuidof(IUnknown) && id != __uuidof(IClassFactory)) return E_NOINTERFACE;
        *output = this; AddRef(); return S_OK;
    }
    ULONG STDMETHODCALLTYPE AddRef() override { return ++refs; }
    ULONG STDMETHODCALLTYPE Release() override { auto n = --refs; if (!n) delete this; return n; }
    HRESULT STDMETHODCALLTYPE CreateInstance(IUnknown* outer, REFIID id, void** output) override {
        if (outer) return CLASS_E_NOAGGREGATION;
        auto object = new (std::nothrow) Profiler;
        if (!object) return E_OUTOFMEMORY;
        auto result = object->QueryInterface(id, output); object->Release(); return result;
    }
    HRESULT STDMETHODCALLTYPE LockServer(BOOL) override { return S_OK; }
};
STDAPI DllGetClassObject(REFCLSID id, REFIID iid, void** output) {
    if (id != profiler_id) return CLASS_E_CLASSNOTAVAILABLE;
    auto object = new (std::nothrow) Factory;
    if (!object) return E_OUTOFMEMORY;
    auto result = object->QueryInterface(iid, output); object->Release(); return result;
}
STDAPI DllCanUnloadNow() { return S_FALSE; }
BOOL WINAPI DllMain(HINSTANCE module, DWORD reason, LPVOID) {
    if (reason == DLL_PROCESS_ATTACH) { own_module = module; DisableThreadLibraryCalls(module); }
    return TRUE;
}

static Negotiation __cdecl negotiate(uint64_t requested, uint64_t granted) {
    if (requested & ~uint64_t{3}) return {1, 0};
    if (requested & ~granted) return {2, 0};
    return {0, requested};
}
static Negotiation __cdecl activate(const Host* supplied, uint64_t requested, uint64_t granted) {
    auto result = negotiate(requested, granted);
    if (result.status) return result;
    if (!supplied || supplied->size != sizeof(Host) || !supplied->decide || !supplied->context) return {4, 0};
    std::lock_guard lifecycle(lifecycle_lock);
#ifndef GLYPHSHIFT_SYNTHETIC_FIXTURE
    try {
        if (!preparation_done.load(std::memory_order_acquire)) {
            auto status = attach_current_runtime();
            if (FAILED(status)) { last_error.store(status); return {3, 0}; }
            auto deadline = GetTickCount64() + 2000;
            while (!preparation_done.load(std::memory_order_acquire) && GetTickCount64() < deadline) Sleep(5);
        }
        if (!preparation_done.load() || FAILED(last_error.load()) || reverting.load()) return {3, 0};
    } catch (...) { return {3, 0}; }
#endif
    std::unique_lock guard(host_lock);
    host = *supplied;
    active_features = result.active;
    decisions_enabled.store(true, std::memory_order_release);
    return result;
}
static int32_t __cdecl deactivate() {
    decisions_enabled.store(false, std::memory_order_release);
    std::unique_lock guard(host_lock);
    host = {};
    active_features = 0;
    return 0; // Identity bypass, NOT profiler detach. CLR keeps this fixture resident.
}
static void __cdecl refresh() {}
extern "C" __declspec(dllexport) Api __cdecl glyphshift_adapter_entry_v1() {
    Descriptor descriptor{};
    descriptor.size = sizeof(Descriptor);
#ifdef GLYPHSHIFT_SYNTHETIC_FIXTURE
    const char id[] = "windows.coreclr.synthetic-text";
#else
    const char id[] = "windows.monogame.sprite-batch-draw-string";
#endif
    descriptor.id.len = sizeof(id) - 1;
    memcpy(descriptor.id.bytes, id, sizeof(id) - 1);
    descriptor.major = descriptor.abi_major = 1;
    descriptor.apply_model = descriptor.placement = 1;
    descriptor.features = 3;
    descriptor.platforms = 1; descriptor.architectures = 2;
    return {sizeof(Api), descriptor, negotiate, activate, deactivate, refresh};
}
extern "C" __declspec(dllexport) uint32_t __cdecl glyphshift_adapter_source_policy_v1() {
#ifdef GLYPHSHIFT_SYNTHETIC_FIXTURE
    return 0;
#else
    return 1;
#endif
}
extern "C" __declspec(dllexport) int32_t __cdecl glyphshift_fixture_status() {
    HRESULT error = last_error.load();
    return FAILED(error) ? error : (patched.load() ? 0 : 1);
}
extern "C" __declspec(dllexport) int32_t __cdecl glyphshift_fixture_revert() {
    deactivate();
    if (!info || !target_module || targets.empty()) return E_UNEXPECTED;
    reverting.store(true);
    HRESULT result = E_FAIL;
    try {
        std::thread worker([&] {
            std::vector<ModuleID> modules(targets.size(), target_module);
            std::vector<mdMethodDef> methods;
            for (const auto& target : targets) methods.push_back(target.method);
            std::vector<HRESULT> statuses(targets.size(), E_FAIL);
            result = info->RequestRevert(static_cast<ULONG>(targets.size()), modules.data(), methods.data(), statuses.data());
            if (SUCCEEDED(result)) for (auto status : statuses) if (FAILED(status)) { result = status; break; }
        });
        worker.join();
    } catch (...) { return E_FAIL; }
    return result;
}
extern "C" __declspec(dllexport) uint32_t __cdecl glyphshift_fixture_compiled_count() { return compiled_methods.load(); }
