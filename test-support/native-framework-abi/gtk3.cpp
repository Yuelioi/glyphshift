// Synthetic C ABI and ownership contract; this does not render GTK pixels.
#include <string>
#include <climits>
#define API extern "C" __declspec(dllexport)
struct Layout { std::string text; bool partial; };
static std::string drawn;
static int outstanding = 0;
API void __cdecl gtk_render_layout(void*, void*, double x, double y, Layout* layout) {
    drawn = (x == 12.5 && y == 23.25) ? layout->text : "bad coordinates";
}
API Layout* __cdecl pango_layout_copy(Layout* layout) { ++outstanding; return new Layout(*layout); }
API const char* __cdecl pango_layout_get_text(Layout* layout) { return layout->text.c_str(); }
API void __cdecl pango_layout_set_text(Layout* layout, const char* text, int size) { layout->text.assign(text, size); }
API void* __cdecl pango_layout_get_attributes(Layout* layout) { return layout->partial ? layout : nullptr; }
API void* __cdecl pango_attr_list_get_iterator(void* value) { return value; }
API void __cdecl pango_attr_iterator_range(void*, int* start, int* end) { *start = 1; *end = INT_MAX; }
API void __cdecl pango_attr_iterator_destroy(void*) {}
API void __cdecl g_object_unref(Layout* layout) { --outstanding; delete layout; }
API const char* __cdecl fixture_draw(int partial) {
    Layout layout{"Open", partial != 0};
    gtk_render_layout(nullptr, nullptr, 12.5, 23.25, &layout);
    if (layout.text != "Open" || outstanding) return "ownership violation";
    return drawn.c_str();
}
