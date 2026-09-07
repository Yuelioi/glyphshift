// Independent C++ layout/struct-return fixture for raylib 5.5's public C ABI.
// Exercises ownership and call arguments; it is not a graphics renderer.
#include <string>
#include <cstdlib>
#define API extern "C" __declspec(dllexport)
struct Vector2 { float x,y; };
struct Color { unsigned char r,g,b,a; };
struct Rectangle { float x,y,width,height; };
struct Image { void* data; int width,height,mipmaps,format; };
struct Texture { unsigned int id; int width,height,mipmaps,format; };
struct GlyphInfo { int value,offsetX,offsetY,advanceX; Image image; };
struct Font { int baseSize,glyphCount,glyphPadding; Texture texture; Rectangle* recs; GlyphInfo* glyphs; };
static std::string drawn;
static int fonts=0,frames=0;
API void __cdecl DrawTextEx(Font font, const char* text, Vector2 point, float size, float spacing, Color tint) {
    drawn = font.texture.id && point.x == 12.5f && point.y == 23.25f && size == 32.f && spacing == 1.5f && tint.r == 11 && tint.a == 255 ? text : "ABI violation";
}
API void __cdecl EndDrawing() { ++frames; }
API void __cdecl CloseWindow() { ++frames; }
API unsigned char* __cdecl LoadFileData(const char*, int* size) { *size=1; return static_cast<unsigned char*>(malloc(1)); }
API void __cdecl UnloadFileData(void* data) { free(data); }
API GlyphInfo* __cdecl LoadFontData(const unsigned char*, int, int, int* points, int count, int) {
    auto glyphs = static_cast<GlyphInfo*>(calloc(count,sizeof(GlyphInfo)));
    for (int i=0;i<count;++i) { glyphs[i].value=points[i]; glyphs[i].image={malloc(1),1,1,1,1}; }
    return glyphs;
}
API void __cdecl UnloadFontData(GlyphInfo* glyphs, int count) {
    for (int i=0;i<count;++i) free(glyphs[i].image.data);
    free(glyphs);
}
API Image __cdecl GenImageFontAtlas(const GlyphInfo*, Rectangle** recs, int count, int, int, int) {
    *recs=static_cast<Rectangle*>(calloc(count,sizeof(Rectangle)));
    for(int i=0;i<count;++i) (*recs)[i]={static_cast<float>(i*16),0,8,8};
    return {malloc(1),count*16,16,1,1};
}
API Texture __cdecl LoadTextureFromImage(Image image) { ++fonts; return {42,image.width,image.height,1,1}; }
API void __cdecl UnloadImage(Image image) { free(image.data); }
API void __cdecl GenTextureMipmaps(Texture* texture) { texture->mipmaps=1; }
API void __cdecl SetTextureFilter(Texture, int) {}
API void __cdecl UnloadFont(Font font) { --fonts; free(font.recs); UnloadFontData(font.glyphs,font.glyphCount); }
API void __cdecl MemFree(void* data) { free(data); }
API const char* __cdecl fixture_draw(int) {
    Rectangle rect{}; GlyphInfo glyph{};
    Font font{16,1,0,{7,16,16,1,1},&rect,&glyph};
    DrawTextEx(font,"Open",{12.5f,23.25f},32.f,1.5f,{11,22,33,255});
    EndDrawing();
    return drawn.c_str();
}
API int __cdecl fixture_fonts() { return fonts; }
