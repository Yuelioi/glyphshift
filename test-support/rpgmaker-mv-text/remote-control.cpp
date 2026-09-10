#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <string>
#include <mutex>
#include "remote-packet.h"
extern "C" int __cdecl glyphshift_mv_session_start_v1() noexcept;
extern "C" int __cdecl glyphshift_mv_session_stop_v1() noexcept;
extern "C" int __cdecl glyphshift_mv_session_evaluate_v1(const char*, uint32_t, char*, uint32_t, uint32_t*) noexcept;
namespace {
std::mutex runtimeGate;
HMODULE runtime = nullptr;
std::wstring runtimePath;
using CommandCall = uint32_t (WINAPI *)(void*);
struct RuntimeCommand { uint32_t size; const char* json; uint32_t length; };
struct Query { uint32_t size; char* output; uint32_t capacity, written; };
uint32_t command(RemotePacket* packet) {
    const auto first = reinterpret_cast<char*>(packet + 1);
    const auto second = first + packet->firstLength;
    const auto output = second + packet->secondLength;
    if (packet->operation == 1) return glyphshift_mv_session_start_v1();
    if (packet->operation == 2) return glyphshift_mv_session_stop_v1();
    if (packet->operation == 3) return glyphshift_mv_session_evaluate_v1(first, packet->firstLength, output, packet->capacity, &packet->written);
    const char* name = nullptr;
    switch (packet->operation) {
    case 4: name = "glyphshift_runtime_activate_v1"; break;
    case 5: name = "glyphshift_runtime_update_v1"; break;
    case 6: name = "glyphshift_runtime_observation_query_v1"; break;
    case 7: name = "glyphshift_runtime_deactivate_v1"; break;
    default: return 3;
    }
    if (!packet->firstLength || packet->firstLength > 32768) return 3;
    const auto length = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, first, static_cast<int>(packet->firstLength), nullptr, 0);
    if (length <= 0) return 3;
    std::wstring path(static_cast<size_t>(length), L'\0');
    if (!MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, first, static_cast<int>(packet->firstLength), path.data(), length)) return 3;
    if (path.find(L'\0') != std::wstring::npos) return 3;
    std::lock_guard<std::mutex> lock(runtimeGate);
    if (runtime && path != runtimePath) return 3;
    if (!runtime) { runtime = LoadLibraryW(path.c_str()); if (!runtime) return 3; runtimePath = path; }
#pragma warning(push)
#pragma warning(disable:4191)
    const auto call = reinterpret_cast<CommandCall>(GetProcAddress(runtime, name));
#pragma warning(pop)
    if (!call) return 3;
    if (packet->operation == 7) return call(nullptr);
    if (packet->operation == 6) {
        Query query{sizeof(Query), output, packet->capacity, 0};
        const auto status = call(&query);
        if (status == 0 && query.written <= packet->capacity) packet->written = query.written;
        return status;
    }
    RuntimeCommand argument{sizeof(RuntimeCommand), second, packet->secondLength};
    return call(&argument);
}
}
extern "C" __declspec(dllexport) DWORD WINAPI glyphshift_mv_remote_control_v1(RemotePacket* packet) noexcept {
    if (!packet || packet->size != sizeof(RemotePacket) || packet->firstLength > packetLimit ||
        packet->secondLength > packetLimit || packet->capacity > packetLimit) return 3;
    packet->written = 0;
    try { packet->status = command(packet); }
    catch (...) { packet->status = 6; }
    return 0;
}
