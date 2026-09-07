// Synthetic Mono export/JIT seam. Does not emulate Unity rendering or a GC.
#include <string>
#include <cstring>
#include <vector>
#include <memory>
#include <mutex>
#define API extern "C" __declspec(dllexport)
struct Class { const char* space; const char* name; };
static Class canvas{"UnityEngine.UI","CanvasUpdateRegistry"}, objectClass{"UnityEngine","Object"}, textClass{"UnityEngine.UI","Text"};
static std::u16string text=u"Open";
static int object,domain,frames;
static void* array[1]={&object};
static std::mutex textLock;
static std::vector<std::unique_ptr<std::u16string>> strings;
API void __cdecl fixture_dispatch(void*) { ++frames; }
API void __cdecl fixture_set(void* instance, void* value) {
    if(instance == &object) { std::lock_guard guard(textLock); text=*static_cast<std::u16string*>(value); }
}
API void* __cdecl mono_get_root_domain() { return &domain; }
API void* __cdecl mono_thread_attach(void* value) { return value; }
API void __cdecl mono_thread_detach(void*) {}
API void __cdecl mono_assembly_foreach(void(__cdecl *callback)(void*,void*), void* data) { callback(&domain,data); }
API void* __cdecl mono_assembly_get_image(void* value) { return value; }
API void* __cdecl mono_class_from_name_case(void*,const char* space,const char* name) {
    for(auto value:{&canvas,&objectClass,&textClass}) if(!strcmp(space,value->space)&&!strcmp(name,value->name)) return value;
    return nullptr;
}
API const char* __cdecl mono_class_get_name(Class* value) { return value->name; }
API const char* __cdecl mono_class_get_namespace(Class* value) { return value->space; }
API int __cdecl mono_class_is_assignable_from(void* left,void* right) { return left==right; }
API void* __cdecl mono_class_get_method_from_name(void* klass,const char* name,int) {
    if(klass==&canvas&&!strcmp(name,"PerformUpdate")) return reinterpret_cast<void*>(&fixture_dispatch);
    if(klass==&objectClass&&!strcmp(name,"FindObjectsOfType")) return &domain;
    if(klass==&textClass&&!strcmp(name,"set_text")) return reinterpret_cast<void*>(&fixture_set);
    return nullptr;
}
API void* __cdecl mono_compile_method(void* method) { return method; }
API void* __cdecl mono_class_get_field_from_name(void* klass,const char* name) { return klass==&textClass&&!strcmp(name,"m_Text") ? &object : nullptr; }
API void* __cdecl mono_runtime_invoke(void*,void*,void**,void** exception) { *exception=nullptr; return array; }
API void* __cdecl mono_class_get_type(void* klass) { return klass; }
API void* __cdecl mono_type_get_object(void*,void* type) { return type; }
API size_t __cdecl mono_array_length(void*) { return 1; }
API void* __cdecl mono_array_addr_with_size(void*,int size,size_t index) { return size==sizeof(void*)&&index==0 ? array : nullptr; }
API void* __cdecl mono_object_get_class(void*) { return &textClass; }
API void __cdecl mono_field_get_value(void*,void*,void* output) { *static_cast<void**>(output)=&text; }
API const char16_t* __cdecl mono_string_chars(std::u16string* value) { return value->data(); }
API int __cdecl mono_string_length(std::u16string* value) { return static_cast<int>(value->size()); }
API void* __cdecl mono_string_new_utf16(void*,const char16_t* value,int length) {
    strings.push_back(std::make_unique<std::u16string>(value,length)); return strings.back().get();
}
API size_t __cdecl mono_gchandle_new_v2(void* value,int) { return reinterpret_cast<size_t>(value); }
API size_t __cdecl mono_gchandle_new_weakref_v2(void* value,int) { return reinterpret_cast<size_t>(value); }
API void* __cdecl mono_gchandle_get_target_v2(size_t value) { return reinterpret_cast<void*>(value); }
API void __cdecl mono_gchandle_free_v2(size_t) {}
API int __cdecl fixture_text(char16_t* output,int capacity) {
    std::lock_guard guard(textLock);
    if(capacity < static_cast<int>(text.size())) return -1;
    std::memcpy(output,text.data(),text.size()*sizeof(char16_t)); return static_cast<int>(text.size());
}
