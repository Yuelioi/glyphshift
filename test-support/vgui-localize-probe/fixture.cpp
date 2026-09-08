#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <cstdint>
#include <cstring>
#include <iostream>
#include <string>

struct Queries {
    virtual const wchar_t* find(const char*) = 0;
    virtual std::uint32_t index(const char*) = 0;
    virtual const wchar_t* value(std::uint32_t) = 0;
};
struct Fixture final : Queries {
    const std::wstring original = L"Open";
    const wchar_t* find(const char* token) override { return value(index(token)); }
    std::uint32_t index(const char* token) override {
        return !std::strcmp(token, "menu.open") ? 0 : ~0u;
    }
    const wchar_t* value(std::uint32_t index) override { return index == 0 ? original.c_str() : nullptr; }
} fixture;
__declspec(noinline) const wchar_t* named(Queries* queries) { return queries->find("menu.open"); }
__declspec(noinline) const wchar_t* indexed(Queries* queries) { return queries->value(queries->index("menu.open")); }
static void text(const wchar_t* value) {
    char buffer[2048]{};
    WideCharToMultiByte(CP_UTF8, 0, value ? value : L"", -1, buffer, sizeof(buffer), nullptr, nullptr);
    std::cout << buffer << std::endl;
}
int main() {
    std::cout << reinterpret_cast<std::uintptr_t>(&fixture) << std::endl;
    std::string command;
    std::wstring cached;
    const wchar_t* held = nullptr;
    while (std::getline(std::cin, command) && command != "quit") {
        if (command == "cached") { text(cached.c_str()); continue; }
        if (command == "held") { text(held); continue; }
        if (command == "table") { text(fixture.original.c_str()); continue; }
        if (command == "mutate-held") { const_cast<wchar_t*>(held)[0] = L'X'; text(held); continue; }
        if (command == "copy-name" || command == "copy-index" || command == "hold") {
            const wchar_t* result = command == "copy-name" ? named(&fixture) : indexed(&fixture);
            cached = result ? result : L"";
            if (command == "hold") held = result;
            text(cached.c_str());
            continue;
        }
        SetLastError(123);
        const wchar_t* result = command == "name" ? named(&fixture) : indexed(&fixture);
        const bool preserved = GetLastError() == 123;
        std::cout << (preserved && result && std::wstring(result) == L"Open" ? "original" : "CHANGED") << std::endl;
    }
    std::cout << "fixture_exit" << std::endl;
}
