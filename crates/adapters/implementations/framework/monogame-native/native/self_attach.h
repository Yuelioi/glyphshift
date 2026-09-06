#pragma once
#include <psapi.h>

// Official Diagnostics IPC v1: Profiler command set 0x03, AttachProfiler 0x01.
// This endpoint belongs to this process. No caller-controlled process or DLL path.
static bool loaded_framework() {
    HMODULE modules[1024]; DWORD bytes = 0;
    if (!EnumProcessModules(GetCurrentProcess(), modules, sizeof(modules), &bytes) || bytes > sizeof(modules)) return false;
    for (DWORD i = 0; i < bytes / sizeof(HMODULE); ++i) {
        wchar_t name[256];
        if (GetModuleBaseNameW(GetCurrentProcess(), modules[i], name, 256) && !_wcsicmp(name, L"MonoGame.Framework.dll")) return true;
    }
    return false;
}
static bool pipe_transfer(HANDLE pipe, BYTE* buffer, DWORD length, bool writing, ULONGLONG deadline) {
    while (length) {
        OVERLAPPED operation{};
        operation.hEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);
        if (!operation.hEvent) return false;
        DWORD transferred = 0;
        BOOL result = writing ? WriteFile(pipe, buffer, length, &transferred, &operation) : ReadFile(pipe, buffer, length, &transferred, &operation);
        if (!result && GetLastError() == ERROR_IO_PENDING) {
            auto now = GetTickCount64();
            DWORD wait = now < deadline ? static_cast<DWORD>(deadline - now) : 0;
            if (WaitForSingleObject(operation.hEvent, wait) == WAIT_OBJECT_0) result = GetOverlappedResult(pipe, &operation, &transferred, FALSE);
            else { CancelIoEx(pipe, &operation); GetOverlappedResult(pipe, &operation, &transferred, TRUE); result = FALSE; }
        }
        CloseHandle(operation.hEvent);
        if (!result || !transferred || transferred > length) return false;
        buffer += transferred; length -= transferred;
    }
    return true;
}
template<class T> static void packet_value(std::vector<BYTE>& packet, const T& value) {
    auto bytes = reinterpret_cast<const BYTE*>(&value);
    packet.insert(packet.end(), bytes, bytes + sizeof(T));
}
static HRESULT attach_current_runtime() {
    if (!GetModuleHandleW(L"coreclr.dll") || !loaded_framework()) return E_NOINTERFACE;
    wchar_t path[32768]; DWORD path_length = GetModuleFileNameW(own_module, path, 32768);
    if (!path_length || path_length >= 32768) return E_FAIL;
    const char configuration[] = "glyphshift-monogame-v1";
    std::vector<BYTE> packet(20, 0);
    memcpy(packet.data(), "DOTNET_IPC_V1\0", 14);
    packet[16] = 3; packet[17] = 1;
    packet_value(packet, uint32_t{3});
    packet_value(packet, profiler_id);
    packet_value(packet, uint32_t{path_length + 1});
    auto path_bytes = reinterpret_cast<const BYTE*>(path);
    packet.insert(packet.end(), path_bytes, path_bytes + (path_length + 1) * sizeof(wchar_t));
    packet_value(packet, uint32_t{sizeof(configuration)});
    packet.insert(packet.end(), configuration, configuration + sizeof(configuration));
    if (packet.size() > UINT16_MAX) return E_INVALIDARG;
    uint16_t size = static_cast<uint16_t>(packet.size());
    memcpy(packet.data() + 14, &size, 2);
    std::wstring endpoint = L"\\\\.\\pipe\\dotnet-diagnostic-" + std::to_wstring(GetCurrentProcessId());
    HANDLE pipe = CreateFileW(endpoint.c_str(), GENERIC_READ | GENERIC_WRITE, 0, nullptr, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, nullptr);
    if (pipe == INVALID_HANDLE_VALUE) return HRESULT_FROM_WIN32(GetLastError());
    const auto deadline = GetTickCount64() + 4000;
    BYTE header[20]{};
    bool valid = pipe_transfer(pipe, packet.data(), static_cast<DWORD>(packet.size()), true, deadline)
        && pipe_transfer(pipe, header, sizeof(header), false, deadline)
        && !memcmp(header, "DOTNET_IPC_V1\0", 14) && header[16] == 0xff;
    uint16_t response_size = 0; memcpy(&response_size, header + 14, 2);
    HRESULT result = E_FAIL;
    if (valid && response_size == 24 && (header[17] == 0 || header[17] == 0xff)) {
        if (!pipe_transfer(pipe, reinterpret_cast<BYTE*>(&result), sizeof(result), false, deadline)) result = E_FAIL;
        if (header[17] == 0xff && SUCCEEDED(result)) result = E_FAIL;
    }
    CloseHandle(pipe);
    return result;
}
