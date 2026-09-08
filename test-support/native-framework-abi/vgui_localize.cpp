// Synthetic query owner. No game libraries, resources, addresses, or text.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <cstdint>
#include <cstring>

constexpr std::uint32_t invalid = ~std::uint32_t{0};
#ifdef DIFFERENT_OBJECT_LAYOUT
struct Entry { std::uint32_t unused[4]; std::uint32_t offset; };
#else
struct Entry { std::uint32_t unused[2]; std::uint32_t offset; };
#endif
wchar_t original[] = L"Open\0Close\0Count %s1\0";
Entry entries[3]{};

class Queries {
#ifdef DIFFERENT_OBJECT_LAYOUT
    std::uint32_t padding[13]{};
#endif
    Entry* records = entries;
    wchar_t* pool = original;
public:
#ifdef DIFFERENT_OBJECT_LAYOUT
    virtual void reserved_a() {}
    virtual void reserved_b() {}
#endif
    virtual wchar_t* find(const char* token) {
        const auto selected = index(token);
        if (selected == invalid) return nullptr;
        return pool + records[selected].offset;
    }
#ifdef DIFFERENT_OBJECT_LAYOUT
    virtual void reserved_c() {}
#endif
    virtual std::uint32_t index(const char* token) {
        if (!token) return invalid;
        if (*token == '#') ++token;
        if (!std::strcmp(token, "menu.open")) return 0;
        if (!std::strcmp(token, "menu.close")) return 1;
        if (!std::strcmp(token, "count")) return 2;
        return invalid;
    }
#ifdef WRONG_QUERY_ABI
    virtual wchar_t* value(std::uint32_t selected, std::uint32_t extra = 0) {
#else
    virtual wchar_t* value(std::uint32_t selected) {
#endif
        if (selected == invalid) return nullptr;
#ifdef WRONG_VALUE_EXPRESSION
        return pool + records[selected].offset + 1;
#else
        return pool + records[selected].offset;
#endif
    }
};
Queries queries;
Queries other;
struct Initialize {
    Initialize() { entries[0].offset = 0; entries[1].offset = 5; entries[2].offset = 11; }
} initialize;

extern "C" __declspec(dllexport) void* CreateInterface(const char* name, int* status) {
    const bool match = !std::strcmp(name, "VGUI_Localize005");
    if (status) *status = match ? 0 : 1;
    return match ? &queries : nullptr;
}
// Indirection prevents compile-time devirtualization at the call site.
__declspec(noinline) wchar_t* call_find(Queries* owner, const char* token) { return owner->find(token); }
__declspec(noinline) wchar_t* call_value(Queries* owner, std::uint32_t selected) { return owner->value(selected); }
extern "C" __declspec(dllexport) wchar_t* fixture_name(const char* token) { return call_find(&queries, token); }
extern "C" __declspec(dllexport) wchar_t* fixture_other_name(const char* token) { return call_find(&other, token); }
extern "C" __declspec(dllexport) wchar_t* fixture_value(std::uint32_t selected) { return call_value(&queries, selected); }
extern "C" __declspec(dllexport) const wchar_t* fixture_source() { return original; }
extern "C" __declspec(dllexport) void** fixture_table() { return *reinterpret_cast<void***>(&queries); }
