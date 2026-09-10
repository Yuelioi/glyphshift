// Exact-version authored-fixture transport. Not a shipping arbitrary-script API.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <MinHook.h>
#include <atomic>
#include <condition_variable>
#include <memory>
#include <mutex>
#include <string>
#include <cstring>
#include <vector>

extern "C" int32_t __cdecl glyphshift_mv_register_node_v1();
namespace {
using Current = void* (__cdecl *)();
using Constructor = void (__thiscall *)(void*, void*);
using Destructor = void (__thiscall *)(void*);
using NewString = void* (__cdecl *)(void**, void*, const char*, int, int);
using Compile = void* (__cdecl *)(void**, void*, void*, void*);
using Run = void* (__thiscall *)(void*, void**, void*);
using Utf8 = void (__thiscall *)(void*, void*, void*);
using Call = void* (__thiscall *)(void*, void**, void*, void*, int, void*);
using Load = int (__cdecl *)(const char*, void*);
Current current;
Constructor scopeNew, catcherNew;
Destructor scopeDelete, catcherDelete, utf8Delete;
NewString stringNew;
Compile compile;
Run run;
Utf8 utf8New;
Call originalCall;
Load originalLoad;
void* callAddress;
void* loadAddress;
std::wstring bridgePath;
std::mutex lifecycle, queueGate;
std::condition_variable completion;
std::atomic<bool> enabled{false}, pending{false};
bool created = false;
thread_local bool busy = false;
struct Request {
    std::string source, result;
    ULONGLONG deadline;
    bool done = false;
    int status = 0;
};
std::shared_ptr<Request> request;

template<class T> bool symbol(HMODULE module, const char* name, T& out) {
#pragma warning(push)
#pragma warning(disable:4191)
    out = reinterpret_cast<T>(GetProcAddress(module, name));
#pragma warning(pop)
    return out != nullptr;
}

// Handles live only inside the intercepted game callback. No V8 handle crosses
// the queue boundary, and neither AI nor filesystem I/O runs on this thread.
std::string execute(void* isolate, void* context, const std::string& body) {
    alignas(16) unsigned char scope[64]{}, catcher[256]{}, text[32]{};
    scopeNew(scope, isolate);
    catcherNew(catcher, isolate);
    struct Cleanup {
        void* scope; void* catcher;
        ~Cleanup() { catcherDelete(catcher); scopeDelete(scope); }
    } cleanup{scope, catcher};
    const auto source = std::string("JSON.stringify((function(){try{if(typeof Utils==='undefined'||Utils.RPGMAKER_NAME!=='MV'||typeof document==='undefined'||document.title!=='Glyphshift MV Contract'||typeof process==='undefined'||process.versions.node!=='9.7.1')return {error:'Wrong fixture context'};") + body + "}catch(e){return {error:String(e)}}})())";
    void* string = nullptr; void* script = nullptr; void* result = nullptr;
    stringNew(&string, isolate, source.c_str(), 0, -1);
    if (!string) return "{\"error\":\"No string\"}";
    compile(&script, context, string, nullptr);
    if (!script) return "{\"error\":\"Compile failed\"}";
    run(script, &result, context);
    if (!result) return "{\"error\":\"Run failed\"}";
    utf8New(text, isolate, result);
    struct UtfCleanup { void* value; ~UtfCleanup() { utf8Delete(value); } } utfCleanup{text};
    const auto data = *reinterpret_cast<const char**>(text);
    const auto length = *reinterpret_cast<const int*>(text + sizeof(void*));
    if (!data || length < 0 || length > 4 * 1024 * 1024) return "{\"error\":\"Invalid result size\"}";
    return std::string(data, static_cast<size_t>(length));
}

void pump(void* context) noexcept {
    if (!enabled.load() || !pending.load() || busy || !context) return;
    const auto isolate = current();
    if (!isolate) return;
    std::shared_ptr<Request> task;
    {
        std::unique_lock<std::mutex> lock(queueGate, std::try_to_lock);
        if (!lock || !request || request->done || GetTickCount64() >= request->deadline) return;
        task = request;
        pending = false;
    }
    busy = true;
    std::string value;
    int status = 0;
    try { value = execute(isolate, context, task->source); }
    catch (...) { status = 6; }
    busy = false;
    std::lock_guard<std::mutex> lock(queueGate);
    if (request != task || task->done) return;
    if (value == "{\"error\":\"Wrong fixture context\"}") { pending = true; return; }
    task->result = std::move(value);
    task->status = status;
    task->done = true;
    completion.notify_all();
}

// Explicit hidden structure-return pointer matches the verified x86 V8 ABI.
void* __fastcall callHook(void* self, void*, void** result, void* context, void* receiver, int argc, void* argv) {
    pump(context);
    return originalCall(self, result, context, receiver, argc, argv);
}

int __cdecl loadHook(const char* path, void* library) {
    const int result = originalLoad(path, library);
    if (result || !enabled.load() || !path || !current()) return result;
    try {
        const auto length = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, nullptr, 0);
        if (length <= 0 || length > 32768) return result;
        std::vector<wchar_t> wide(static_cast<size_t>(length));
        if (!MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, wide.data(), length)) return result;
        for (auto& ch : wide) if (ch == L'/') ch = L'\\';
        if (_wcsicmp(wide.data(), bridgePath.c_str()) == 0) glyphshift_mv_register_node_v1();
    } catch (...) { /* Registration failure lets Node reject this module. */ }
    return result;
}

bool prepare() {
    const auto nw = GetModuleHandleW(L"nw.dll"), node = GetModuleHandleW(L"node.dll");
    if (!nw || !node) return false;
    const char* (__cdecl *version)() = nullptr;
    if (!symbol(nw, "?GetVersion@V8@v8@@SAPBDXZ", version) || std::strcmp(version(), "6.5.254.31")) return false;
    if (!symbol(nw, "?GetCurrent@Isolate@v8@@SAPAV12@XZ", current) ||
        !symbol(nw, "??0HandleScope@v8@@QAE@PAVIsolate@1@@Z", scopeNew) ||
        !symbol(nw, "??1HandleScope@v8@@QAE@XZ", scopeDelete) ||
        !symbol(nw, "??0TryCatch@v8@@QAE@PAVIsolate@1@@Z", catcherNew) ||
        !symbol(nw, "??1TryCatch@v8@@QAE@XZ", catcherDelete) ||
        !symbol(nw, "?NewFromUtf8@String@v8@@SA?AV?$MaybeLocal@VString@v8@@@2@PAVIsolate@2@PBDW4NewStringType@2@H@Z", stringNew) ||
        !symbol(nw, "?Compile@Script@v8@@SA?AV?$MaybeLocal@VScript@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VString@v8@@@2@PAVScriptOrigin@2@@Z", compile) ||
        !symbol(nw, "?Run@Script@v8@@QAE?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@@Z", run) ||
        !symbol(nw, "??0Utf8Value@String@v8@@QAE@PAVIsolate@2@V?$Local@VValue@v8@@@2@@Z", utf8New) ||
        !symbol(nw, "??1Utf8Value@String@v8@@QAE@XZ", utf8Delete) ||
        !symbol(nw, "?Call@Function@v8@@QAE?AV?$MaybeLocal@VValue@v8@@@2@V?$Local@VContext@v8@@@2@V?$Local@VValue@v8@@@2@HQAV52@@Z", callAddress) ||
        !symbol(node, "uv_dlopen", loadAddress)) return false;
    HMODULE self = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN,
        reinterpret_cast<LPCWSTR>(&prepare), &self)) return false;
    std::vector<wchar_t> path(32768);
    const auto count = GetModuleFileNameW(self, path.data(), static_cast<DWORD>(path.size()));
    if (!count || count >= path.size()) return false;
    bridgePath.assign(path.data(), count);
    if (MH_Initialize() != MH_OK) return false;
    if (MH_CreateHook(callAddress, reinterpret_cast<void*>(&callHook), reinterpret_cast<void**>(&originalCall)) != MH_OK) {
        MH_Uninitialize();
        return false;
    }
    if (MH_CreateHook(loadAddress, reinterpret_cast<void*>(&loadHook), reinterpret_cast<void**>(&originalLoad)) != MH_OK) {
        MH_RemoveHook(callAddress);
        MH_Uninitialize();
        return false;
    }
    return true;
}
}

extern "C" __declspec(dllexport) int __cdecl glyphshift_mv_session_start_v1() noexcept {
    try {
        std::lock_guard<std::mutex> lock(lifecycle);
        if (enabled) return 0;
        if (!created) { if (!prepare()) return 1; created = true; }
        if (MH_QueueEnableHook(callAddress) != MH_OK || MH_QueueEnableHook(loadAddress) != MH_OK || MH_ApplyQueued() != MH_OK) {
            MH_DisableHook(callAddress);
            MH_DisableHook(loadAddress);
            return 2;
        }
        enabled = true;
        return 0;
    } catch (...) { return 6; }
}

extern "C" __declspec(dllexport) int __cdecl glyphshift_mv_session_stop_v1() noexcept {
    try {
        std::lock_guard<std::mutex> lock(lifecycle);
        enabled = false;
        {
            std::lock_guard<std::mutex> queueLock(queueGate);
            if (request) { request->status = 5; request->done = true; request.reset(); }
            pending = false;
            completion.notify_all();
        }
        if (!created) return 0;
        if (MH_QueueDisableHook(callAddress) != MH_OK || MH_QueueDisableHook(loadAddress) != MH_OK || MH_ApplyQueued() != MH_OK) return 2;
        // Trampolines and DLL stay resident: an intercepted call may still be
        // returning through them. A future session can enable the same hooks.
        return 0;
    } catch (...) { return 6; }
}

extern "C" __declspec(dllexport) int __cdecl glyphshift_mv_session_evaluate_v1(
    const char* source, uint32_t length, char* output, uint32_t capacity, uint32_t* written) noexcept {
    if (!source || !length || length > 131072 || !output || !written || capacity > 4 * 1024 * 1024) return 3;
    *written = 0;
    try {
        std::unique_lock<std::mutex> lock(queueGate);
        if (!enabled || request) return 4;
        auto task = std::make_shared<Request>();
        task->source.assign(source, length);
        task->deadline = GetTickCount64() + 5000;
        request = task;
        pending = true;
        const bool done = completion.wait_for(lock, std::chrono::seconds(5), [&] { return task->done; });
        if (request == task) { request.reset(); pending = false; }
        if (!done) return 7;
        if (task->status) return task->status;
        if (task->result.size() > capacity) return 8;
        *written = static_cast<uint32_t>(task->result.size());
        std::memcpy(output, task->result.data(), *written);
        return 0;
    } catch (...) { return 6; }
}
