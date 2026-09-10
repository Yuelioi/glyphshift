// x86 fixture-only process transport. The caller supplies executable identity,
// creation time and DLL path explicitly; it never searches for games by name.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <tlhelp32.h>
#include <fcntl.h>
#include <io.h>
#include <cstdio>
#include <string>
#include <vector>
#include <stdexcept>
#include "remote-packet.h"
struct Handle {
    HANDLE value;
    ~Handle() { if (value && value != INVALID_HANDLE_VALUE) CloseHandle(value); }
};
static void require(bool value, const char* message) { if (!value) throw std::runtime_error(message); }
static uintptr_t moduleBase(DWORD pid, const wchar_t* filename) {
    Handle snapshot{CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid)};
    require(snapshot.value != INVALID_HANDLE_VALUE, "Module snapshot failed");
    MODULEENTRY32W entry{}; entry.dwSize = sizeof(entry);
    if (Module32FirstW(snapshot.value, &entry)) do {
        if (_wcsicmp(entry.szModule, filename) == 0) return reinterpret_cast<uintptr_t>(entry.modBaseAddr);
    } while (Module32NextW(snapshot.value, &entry));
    throw std::runtime_error("Expected target module missing");
}
static DWORD invoke(HANDLE process, uintptr_t function, void* data) {
    Handle thread{CreateRemoteThread(process, nullptr, 0, reinterpret_cast<LPTHREAD_START_ROUTINE>(function), data, 0, nullptr)};
    require(thread.value != nullptr, "Remote thread failed");
    require(WaitForSingleObject(thread.value, 15000) == WAIT_OBJECT_0, "Remote thread timeout; allocation retained");
    DWORD code = 0; require(GetExitCodeThread(thread.value, &code) != 0, "Remote status unavailable");
    return code;
}
static void writeRemote(HANDLE process, void* remote, const void* local, size_t size) {
    SIZE_T written = 0;
    require(WriteProcessMemory(process, remote, local, size, &written) && written == size, "Remote write failed");
}
int wmain(int argc, wchar_t** argv) {
    try {
        require(argc == 5, "Expected target, DLL, executable, creation time");
        const auto pid = static_cast<DWORD>(std::stoul(argv[1]));
        Handle process{OpenProcess(PROCESS_CREATE_THREAD | PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE, FALSE, pid)};
        require(process.value != nullptr, "Target unavailable");
        wchar_t image[32768]{}; DWORD imageLength = 32768;
        require(QueryFullProcessImageNameW(process.value, 0, image, &imageLength) && _wcsicmp(image, argv[3]) == 0, "Executable identity changed");
        FILETIME created{}, exited{}, kernel{}, user{};
        require(GetProcessTimes(process.value, &created, &exited, &kernel, &user) != 0, "Creation time unavailable");
        const uint64_t started = (uint64_t{created.dwHighDateTime} << 32) | created.dwLowDateTime;
        require(started == std::stoull(argv[4]), "Target instance changed");
        USHORT machine = 0, native = 0;
        require(IsWow64Process2(process.value, &machine, &native) && (machine ? machine : native) == IMAGE_FILE_MACHINE_I386, "Expected x86 target");

        // Resolve the actual owner of forwarded LoadLibraryW, then use its RVA
        // in the matching remote module; never assume equal ASLR addresses.
        const auto loader = GetProcAddress(GetModuleHandleW(L"kernel32.dll"), "LoadLibraryW");
        HMODULE owner = nullptr;
        require(loader && GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            reinterpret_cast<LPCWSTR>(loader), &owner), "Loader owner missing");
        wchar_t ownerPath[32768]{};
        require(GetModuleFileNameW(owner, ownerPath, 32768) != 0, "Loader name unavailable");
        const auto ownerName = wcsrchr(ownerPath, L'\\');
        require(ownerName != nullptr, "Loader path invalid");
        const auto remoteLoader = moduleBase(pid, ownerName + 1) + reinterpret_cast<uintptr_t>(loader) - reinterpret_cast<uintptr_t>(owner);
        const size_t pathBytes = (wcslen(argv[2]) + 1) * sizeof(wchar_t);
        auto remotePath = VirtualAllocEx(process.value, nullptr, pathBytes, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        require(remotePath != nullptr, "Path allocation failed");
        writeRemote(process.value, remotePath, argv[2], pathBytes);
        const auto remoteLibrary = invoke(process.value, remoteLoader, remotePath);
        VirtualFreeEx(process.value, remotePath, 0, MEM_RELEASE);
        require(remoteLibrary != 0, "Fixture DLL load failed");
        const auto local = LoadLibraryExW(argv[2], nullptr, DONT_RESOLVE_DLL_REFERENCES);
        require(local != nullptr, "Fixture export image unavailable");
        const auto control = GetProcAddress(local, "_glyphshift_mv_remote_control_v1@4");
        require(control != nullptr, "Fixture control export missing");
        const auto remoteControl = uintptr_t{remoteLibrary} + reinterpret_cast<uintptr_t>(control) - reinterpret_cast<uintptr_t>(local);
        FreeLibrary(local);

        _setmode(_fileno(stdin), _O_BINARY); _setmode(_fileno(stdout), _O_BINARY);
        uint32_t header[3]{};
        while (fread(header, sizeof(header), 1, stdin) == 1) {
            require(header[1] <= packetLimit && header[2] <= packetLimit, "Command exceeds fixture limit");
            const size_t bytes = sizeof(RemotePacket) + header[1] + header[2] + packetLimit;
            std::vector<char> buffer(bytes);
            auto& packet = *reinterpret_cast<RemotePacket*>(buffer.data());
            packet = {sizeof(RemotePacket), header[0], header[1], header[2], packetLimit, 0, 0};
            const size_t input = size_t{header[1]} + header[2];
            require(fread(buffer.data() + sizeof(packet), 1, input, stdin) == input, "Incomplete command");
            auto allocation = VirtualAllocEx(process.value, nullptr, bytes, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
            require(allocation != nullptr, "Command allocation failed");
            writeRemote(process.value, allocation, buffer.data(), sizeof(packet) + input);
            const auto code = invoke(process.value, remoteControl, allocation);
            require(code == 0, "Remote control rejected packet");
            SIZE_T read = 0;
            require(ReadProcessMemory(process.value, allocation, buffer.data(), bytes, &read) && read == bytes, "Remote read failed");
            VirtualFreeEx(process.value, allocation, 0, MEM_RELEASE);
            require(packet.written <= packetLimit, "Invalid output length");
            const uint32_t response[]{packet.status, packet.written};
            require(fwrite(response, sizeof(response), 1, stdout) == 1 &&
                fwrite(buffer.data() + sizeof(packet) + input, 1, packet.written, stdout) == packet.written, "Response write failed");
            fflush(stdout);
        }
        return 0;
    } catch (const std::exception& error) {
        fprintf(stderr, "%s\n", error.what());
        return 1;
    }
}
