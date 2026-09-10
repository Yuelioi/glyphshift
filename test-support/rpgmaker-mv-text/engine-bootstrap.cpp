// Fixed engine bootstrap embedded into the fixture artifact. The production
// activation path must never depend on the test driver's arbitrary evaluator.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <string>
#include <vector>
#include <cstring>
#include <atomic>
#include "engine_adapter.h"
extern "C" int __cdecl glyphshift_mv_session_start_v1() noexcept;
extern "C" int __cdecl glyphshift_mv_session_stop_v1() noexcept;
extern "C" int __cdecl glyphshift_mv_session_evaluate_v1(const char*, uint32_t, char*, uint32_t, uint32_t*) noexcept;
namespace {
std::atomic<bool> engineStarted{false};
bool execute(const std::string& source) {
    char result[4096]{}; uint32_t length = 0;
    const auto status = glyphshift_mv_session_evaluate_v1(source.data(), static_cast<uint32_t>(source.size()), result, sizeof(result) - 1, &length);
    return status == 0 && std::string(result, length) == "{\"value\":true}";
}
std::string modulePath() {
    HMODULE self = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        reinterpret_cast<LPCWSTR>(&modulePath), &self)) return {};
    std::vector<wchar_t> path(32768);
    const auto length = GetModuleFileNameW(self, path.data(), static_cast<DWORD>(path.size()));
    if (!length || length >= path.size()) return {};
    const auto bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, path.data(), static_cast<int>(length), nullptr, 0, nullptr, nullptr);
    if (!bytes) return {};
    std::string utf8(static_cast<size_t>(bytes), '\0');
    if (!WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, path.data(), static_cast<int>(length), utf8.data(), bytes, nullptr, nullptr)) return {};
    std::string quoted = "\"";
    for (const unsigned char ch : utf8) {
        if (ch < 32) return {};
        if (ch == '\\' || ch == '"') quoted += '\\';
        quoted += static_cast<char>(ch);
    }
    return quoted + "\"";
}
}
bool startEngine() noexcept {
    try {
        if (glyphshift_mv_session_start_v1() != 0) return false;
        const auto path = modulePath();
        if (path.empty()) return false;
        const auto source = std::string("return {value:(function(){var key=Symbol.for('glyphshift.mv.fixture.engine.v1');var state=window[key];if(!state){var nativeModule={exports:{}};process.dlopen(nativeModule,") + path +
            ");var exports=(function(){var module={exports:{}};" + embedded_adapter +
            ";return module.exports;})();state={native:nativeModule.exports,exports:exports,session:null};Object.defineProperty(window,key,{value:state,configurable:false});}"
            "if(!state.session)state.session=state.exports.installMessageAdapter(Window_Message,{resolve:function(event){return state.native.resolve(event.source,event.usage==='draw'?1:3);}});return true;})()};";
        const bool ready = execute(source);
        if (ready) engineStarted = true;
        return ready;
    } catch (...) { return false; }
}
bool stopEngine() noexcept {
    try {
        if (!engineStarted) return glyphshift_mv_session_stop_v1() == 0;
        const bool stopped = execute("return {value:(function(){var state=window[Symbol.for('glyphshift.mv.fixture.engine.v1')];if(state&&state.session){state.session.stop();state.session=null;}return true;})()};");
        if (stopped) engineStarted = false;
        const auto nativeStatus = glyphshift_mv_session_stop_v1();
        return stopped && nativeStatus == 0;
    } catch (...) { return false; }
}
