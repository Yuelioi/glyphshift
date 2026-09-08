// Deterministic MSVC x86 retained-text owner. No game code, data or addresses.
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <atomic>
#include <cassert>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <thread>
#include <vector>
#ifndef OBJECT_PADDING
#define OBJECT_PADDING 5
#endif
struct Object { void** table; uint32_t padding[OBJECT_PADDING]; char* text; int limit, dirty, glyphs, finish; std::string* visible; };
enum { TextMember = 4 + OBJECT_PADDING * 4, LimitMember = TextMember + 4,
    DirtyMember = TextMember + 8, GlyphMember = TextMember + 12, FinishMember = TextMember + 16 };
extern "C" void __cdecl set_text();
extern "C" void __cdecl append_text();
extern "C" void __cdecl get_text();
extern "C" void __cdecl destroy_text();
extern "C" void __cdecl deleting();
extern "C" void __cdecl tick_text();
extern "C" void __cdecl reserved();
extern "C" void __cdecl history_tick_text();
struct Type { void* table; void* spare; char name[64]; };
#ifdef WRONG_TYPE
const Type type = {nullptr, nullptr, ".?AVUnrelatedTextOwner@@"};
#else
const Type type = {nullptr, nullptr, ".?AVkcFEScriptObjStringUTF8@@"};
#endif
const uint32_t hierarchy[4]{};
struct Locator { uint32_t signature, offset, constructor; const Type* type; const void* hierarchy; };
const Locator locator = {0, 0, 0, &type, hierarchy};
struct Table { const Locator* locator; void* slots[4]; };
extern "C" const Table object_table = {&locator, {deleting, reserved, tick_text, reserved}};
const Type history_type = {nullptr, nullptr, ".?AVkcFEScriptObjLogStringUTF8@@"};
const Locator history_locator = {0, 0, 0, &history_type, hierarchy};
extern "C" const Table history_table = {&history_locator, {reserved, reserved, history_tick_text, reserved}};
std::vector<Object*> history_rows;

extern "C" __declspec(naked) void reserved() { __asm { ret } }
extern "C" void __cdecl render(Object* object) {
    if (object->visible && object->dirty && object->finish) {
        *object->visible = object->text ? object->text : "";
        object->dirty = object->finish = 0;
    }
}
extern "C" void __cdecl render_history() { for (auto* row : history_rows) render(row); }
extern "C" __declspec(naked) void history_tick_text() {
    __asm { call render_history }
    __asm { mov eax, 1 }
    __asm { ret 4 }
}
extern "C" __declspec(naked) void tick_text() {
    __asm { push ecx }
    __asm { call render }
    __asm { add esp, 4 }
    __asm { mov eax, 1 }
    __asm { ret 4 }
}
extern "C" __declspec(naked) void deleting() {
    __asm { push ebp }
    __asm { mov ebp, esp }
    __asm { call destroy_text }
    __asm { pop ebp }
    __asm { ret 4 }
}
extern "C" __declspec(naked) void destroy_text() {
    __asm { push esi }
    __asm { mov esi, ecx }
    __asm { mov eax, [esi + TextMember] }
    __asm { mov dword ptr [esi], offset object_table + 4 }
    __asm { push eax }
    __asm { call free }
    __asm { add esp, 4 }
    __asm { mov dword ptr [esi + TextMember], 0 }
    __asm { pop esi }
    __asm { ret }
}
extern "C" __declspec(naked) void set_text() {
    __asm {
        push ebp
        mov ebp, esp
        push ebx
        push esi
        push edi
        mov ebx, ecx
        mov esi, [ebp + 8]
        xor edi, edi
        test esi, esi
        jz sized
        cmp byte ptr [esi], 0
        je sized
    length_loop:
        inc edi
        cmp byte ptr [esi + edi], 0
        jne length_loop
    sized:
        push dword ptr [ebx + TextMember]
        call free
        add esp, 4
        inc edi
        push edi
        call malloc
        add esp, 4
        mov [ebx + TextMember], eax
        mov edi, eax
        test esi, esi
        jz empty
    copy_loop:
        mov al, [esi]
        mov [edi], al
        inc esi
        inc edi
        test al, al
        jne copy_loop
        jmp done
    empty:
        mov byte ptr [edi], 0
    done:
        mov dword ptr [ebx + GlyphMember], -1
        pop edi
        pop esi
        pop ebx
        pop ebp
        ret 4
    }
}
extern "C" char* __cdecl prefix(char* old, const char* input) {
    const size_t n = old ? std::strlen(old) : 0;
    auto* result = static_cast<char*>(std::malloc(n + (input ? std::strlen(input) : 0) + 1));
    if (n) std::memcpy(result, old, n);
    result[n] = 0; std::free(old); return result;
}
extern "C" __declspec(naked) void append_text() {
    __asm {
        push ebp
        mov ebp, esp
        push ebx
        push esi
        push edi
        mov ebx, ecx
        mov esi, [ebp + 8]
        push esi
        push dword ptr [ebx + TextMember]
        call prefix
        add esp, 8
        mov [ebx + TextMember], eax
        mov edi, eax
    find_end:
        cmp byte ptr [edi], 0
        je found
        inc edi
        jmp find_end
    found:
        test esi, esi
        jz done
    copy_loop:
        mov cl, [esi]
        mov [edi], cl
        inc esi
        inc edi
        test cl, cl
        jne copy_loop
    done:
        pop edi
        pop esi
        pop ebx
        pop ebp
        ret 4
    }
}
extern "C" __declspec(naked) void get_text() {
#ifdef WRONG_MEMBER
    __asm { mov eax, [ecx + TextMember + 4] }
#else
    __asm { mov eax, [ecx + TextMember] }
#endif
    __asm { ret }
}
extern "C" const unsigned char end_marker[] = {255, 255, 0};
extern "C" __declspec(naked) void append_marker() {
    __asm { push ebp }
    __asm { mov ebp, esp }
    __asm { push ecx }
    __asm { push offset end_marker }
    __asm { call append_text }
    __asm { mov esp, ebp }
    __asm { pop ebp }
    __asm { ret }
}
extern "C" __declspec(naked) void show_text() {
    __asm { push ebp }
    __asm { mov ebp, esp }
    __asm { mov eax, [ebp + 8] }
    __asm { mov [ecx + LimitMember], eax }
    __asm { mov dword ptr [ecx + DirtyMember], 1 }
#ifdef WRONG_DISPLAY
    __asm { mov dword ptr [ecx + GlyphMember], 2 }
#else
    __asm { mov dword ptr [ecx + GlyphMember], 1 }
#endif
    __asm { pop ebp }
    __asm { ret 4 }
}
extern "C" __declspec(naked) void finish_text() {
    __asm { mov dword ptr [ecx + FinishMember], 1 }
    __asm { ret }
}
// Four-argument script handlers expose a genuine receiver and stack-local string.
#define HANDLER(name, method) \
extern "C" __declspec(naked) void name() { \
    __asm { push ebp } __asm { mov ebp, esp } __asm { sub esp, 16 } \
    __asm { push esi } __asm { mov esi, [ebp + 8] } \
    __asm { mov byte ptr [ebp - 16], 0 } __asm { lea eax, [ebp - 16] } \
    __asm { mov ecx, esi } __asm { push eax } __asm { call method } \
    __asm { pop esi } __asm { mov esp, ebp } __asm { pop ebp } __asm { ret 16 } \
}
HANDLER(set_command, set_text)
HANDLER(append_command, append_text)
#define NO_ARGUMENT_HANDLER(name, method) \
extern "C" __declspec(naked) void name() { \
    __asm { mov ecx, [esp + 4] } __asm { call method } __asm { ret 16 } \
}
NO_ARGUMENT_HANDLER(marker_command, append_marker)
NO_ARGUMENT_HANDLER(skip_command, finish_text)
struct Command { char name[16]; void* function; };
extern "C" __declspec(dllexport) const Command commands[] = {
    {"str", set_command}, {"reset", reserved}, {"apend", append_command}, {"apendmark", marker_command},
    {"clear", reserved}, {"skip", skip_command}, {"refex", reserved}
};

struct Fixed { uint16_t len; char bytes[96]; };
struct Descriptor { uint32_t size; Fixed id; uint16_t major, minor, patch, abi_major, abi_minor; uint32_t model, placement; uint64_t features; uint32_t platforms, architectures; };
struct Decision { int32_t status; uint64_t generation; uint32_t bits, text_len, font_len; };
struct Negotiation { int32_t status; uint64_t features; };
using Decide = Decision(__cdecl*)(void*, const wchar_t*, uint32_t, wchar_t*, uint32_t, wchar_t*, uint32_t);
struct Host { uint32_t size; void* context; Decide decide; uint32_t(__cdecl* characters)(void*, wchar_t*, uint32_t); };
struct Api { uint32_t size; Descriptor descriptor; Negotiation(__cdecl* negotiate)(uint64_t, uint64_t); Negotiation(__cdecl* activate)(const Host*, uint64_t, uint64_t); int32_t(__cdecl* deactivate)(); void(__cdecl* refresh)(); };
using Text = void(__thiscall*)(Object*, const char*);
using Tick = size_t(__thiscall*)(Object*, size_t);
using Destroy = void(__thiscall*)(Object*);
using Show = void(__thiscall*)(Object*, int);
void set(Object& o, const char* s) { reinterpret_cast<Text>(set_text)(&o, s); }
void append(Object& o, const char* s) { reinterpret_cast<Text>(append_text)(&o, s); }
void tick(Object& o) { reinterpret_cast<Tick>(o.table[2])(&o, 0); }
void destroy(Object& o) { reinterpret_cast<Destroy>(destroy_text)(&o); }
Object create(const char* s) { Object o{}; o.table = const_cast<void**>(object_table.slots); set(o, s); return o; }
std::wstring source, translation;
std::vector<std::wstring> observed;
uint64_t generation = 1;
int output_mode = 0;
Object* reenter = nullptr;
Decision __cdecl decide(void*, const wchar_t* input, uint32_t len, wchar_t* output, uint32_t capacity, wchar_t*, uint32_t) {
    observed.emplace_back(input, len);
    if (reenter) tick(*reenter);
    if (observed.back() != source || translation.empty()) return {0, generation, 0, 0, 0};
    if (output_mode == 1) return {5, generation, 1, capacity + 1, 0};
    if (output_mode == 2) { output[0] = 0xd800; return {0, generation, 1, 1, 0}; }
    assert(capacity >= translation.size());
    std::memcpy(output, translation.data(), translation.size() * sizeof(wchar_t));
    return {0, generation, 1, static_cast<uint32_t>(translation.size()), 0};
}
void publication(Api& api, std::wstring from, std::wstring to) { source = std::move(from); translation = std::move(to); ++generation; api.refresh(); }
int main(int argc, char** argv) {
    assert(argc == 2);
    const auto module = LoadLibraryA(argv[1]); assert(module);
    const auto entry = reinterpret_cast<Api(__cdecl*)()>(GetProcAddress(module, "glyphshift_adapter_entry_v1")); assert(entry);
    Api api = entry(); Host host{sizeof(Host), nullptr, decide, nullptr};
    assert(api.size == sizeof(Api)); assert(api.descriptor.architectures == 1);
    assert(api.negotiate(4, 7).status == 1); assert(api.negotiate(3, 1).status == 2);
    assert(api.activate(nullptr, 3, 3).status == 4);
    auto object = create("A complete synthetic sentence.");
    const auto activated = api.activate(&host, 1, 1);
#if defined(WRONG_TYPE) || defined(WRONG_MEMBER) || defined(WRONG_DISPLAY)
    assert(activated.status == 3); destroy(object); std::puts("unsupported shape rejected"); return 0;
#endif
    if (activated.status != 0) { std::printf("activation failed: %d\n", activated.status); return 2; }
    tick(object); assert(observed.back() == L"A complete synthetic sentence.");
    assert(api.deactivate() == 0);
    assert(api.activate(&host, 3, 3).status == 0);
    const std::string suffix = std::string(reinterpret_cast<const char*>(end_marker)) + reinterpret_cast<const char*>(end_marker);
    const std::string marked_source = std::string("Visible dialogue.") + suffix;
    auto marked = create(marked_source.c_str());
    std::string display = marked_source;
    marked.visible = &display;
    publication(api, L"Visible dialogue.", L"First visible translation."); tick(marked);
    assert(display == std::string("First visible translation.") + suffix);
    assert(observed.back() == L"Visible dialogue.");
    display = "Stale display cache"; api.refresh(); tick(marked);
    assert(display == std::string("First visible translation.") + suffix);
    publication(api, source, L"Second visible translation."); tick(marked);
    assert(display == std::string("Second visible translation.") + suffix);
    publication(api, source, L"合成对话刷新"); tick(marked);
    assert(display == std::string(u8"合成对话刷新") + suffix);
    publication(api, source, std::wstring(8192, L'x')); tick(marked);
    assert(display == marked_source); // Marker bytes also count against the limit.
    publication(api, source, L"Before release"); tick(marked);
    std::atomic<int> release_marked{-1};
    std::thread release_control([&] { release_marked = api.deactivate(); });
    while (release_marked < 0) { tick(marked); Sleep(1); }
    release_control.join(); tick(marked); assert(release_marked == 0); assert(display == marked_source);
    assert(api.activate(&host, 3, 3).status == 0);
    // History displays plain strings and pumps only its container, never row.tick.
    Object history{}; history.table = const_cast<void**>(history_table.slots);
    auto history_row = create("History source"); std::string history_display = "History source";
    history_row.visible = &history_display; history_rows.push_back(&history_row);
    publication(api, L"History source", L"History first translation");
    reenter = &history;
    reinterpret_cast<Show>(show_text)(&history_row, -1);
    reenter = nullptr; tick(history);
    assert(history_display == "History first translation");
    publication(api, source, L"History second translation"); tick(history);
    assert(history_display == "History second translation");
    history_display = "Stale history cache"; api.refresh(); tick(history);
    assert(history_display == "History second translation");
    std::atomic<int> history_stopped{-1};
    std::thread history_stop([&] { history_stopped = api.deactivate(); });
    while (history_stopped < 0) { tick(history); Sleep(1); }
    history_stop.join(); tick(history);
    assert(history_stopped == 0 && history_display == "History source");
    history_rows.clear(); destroy(history_row);
    assert(api.activate(&host, 3, 3).status == 0);
    publication(api, source, L""); tick(marked); assert(display == marked_source);
    destroy(marked);
    publication(api, L"A complete synthetic sentence.", L"First translated sentence.");
    reenter = &object; tick(object); reenter = nullptr;
    assert(std::string(object.text) == "First translated sentence.");
    publication(api, source, L"Second translated sentence."); tick(object);
    assert(std::string(object.text) == "Second translated sentence.");
    publication(api, source, L""); tick(object); assert(std::string(object.text) == "A complete synthetic sentence.");
    publication(api, source, L"Changed"); tick(object);
    output_mode = 1; api.refresh(); tick(object); assert(std::string(object.text) == "A complete synthetic sentence.");
    output_mode = 2; api.refresh(); tick(object); assert(std::string(object.text) == "A complete synthetic sentence."); output_mode = 0;
    set(object, "Hello"); publication(api, L"Hello", L"Bonjour"); tick(object);
    assert(std::string(object.text) == "Bonjour");
    append(object, " world"); assert(std::string(object.text) == "Hello world");
    publication(api, L"Hello world", L"Complete greeting"); tick(object); assert(std::string(object.text) == "Complete greeting");
    set(object, "External write"); tick(object); assert(std::string(object.text) == "External write");
    set(object, "[ruby/reading]"); publication(api, L"[ruby/reading]", L"Unsafe"); tick(object); assert(std::string(object.text) == "[ruby/reading]");
    set(object, "Hidden original"); publication(api, L"Hidden original", L"Hidden translation"); tick(object);
    auto visible = create("Visible original"); tick(visible);
    std::atomic<int> stopped{-1};
    std::thread control([&] { stopped = api.deactivate(); });
    while (stopped < 0) { tick(visible); Sleep(1); }
    control.join(); assert(stopped == 0); assert(std::string(object.text) == "Hidden original");
    assert(api.activate(&host, 3, 3).status == 0);
    publication(api, L"Hidden original", L"Changed"); tick(object); destroy(object);
    object = create("Reused object"); tick(object); assert(std::string(object.text) == "Reused object");
    publication(api, L"Reused object", L"Timeout translation"); tick(object);
    assert(api.deactivate() == 3); // No UI callback: restoration cannot be claimed.
    tick(visible); assert(std::string(object.text) == "Reused object"); assert(api.deactivate() == 0);
    assert(api.activate(&host, 3, 3).status == 0);
    publication(api, L"Budget original", std::wstring(8192, L'x'));
    std::vector<Object> many;
    many.reserve(300);
    for (int i = 0; i < 300; ++i) { many.push_back(create("Budget original")); tick(many.back()); }
    for (auto& item : many) tick(item);
    stopped = -1;
    std::thread bounded_stop([&] { stopped = api.deactivate(); });
    while (stopped < 0) { tick(visible); Sleep(1); }
    bounded_stop.join(); assert(stopped == 0);
    for (auto& item : many) { assert(std::string(item.text) == "Budget original"); destroy(item); }
    destroy(object); destroy(visible);
    std::puts("capture, visible marker refresh/retry/restore, Unicode, generations, append, reentrancy, external writes, controls, hidden restore, destruction and timeout passed");
}
