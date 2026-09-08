#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <string>
#include <cstring>
#include <vector>
#include "vgui_surface.h"
struct Metrics {
    __declspec(noinline) int Tall(unsigned int) { return 16; }
    __declspec(noinline) int Ascent(unsigned int,wchar_t) { return 12; }
    __declspec(noinline) bool IsAdditive(unsigned int) { return false; }
    __declspec(noinline) void GetAbc(unsigned int,wchar_t,int& a,int& b,int& c) { a=0;b=10;c=0; }
    __declspec(noinline) int GetWidth(unsigned int,int) { return 10; }
    __declspec(noinline) void Size(unsigned int,const wchar_t* text,int& width,int& height) { width=static_cast<int>(wcslen(text))*10;height=16; }
} metrics;
__declspec(noinline) Metrics* GetMetrics() { return &metrics; }
static std::wstring output;
static unsigned long long trace=1469598103934665603ull;
static void Mix(unsigned int value) { trace=(trace^value)*1099511628211ull; }
struct Renderer final: Surface {
    unsigned int font=0; int r=0,g=0,b=0,a=0,x=0,y=0;
void Opaque0() override {}
void Opaque1() override {}
void Opaque2() override {}
void Opaque3() override {}
void Opaque4() override {}
void Opaque5() override {}
void Opaque6() override {}
void Opaque7() override {}
void PushMakeCurrent(unsigned int,bool) override {}
void PopMakeCurrent(unsigned int) override {}
void Opaque10() override {}
void Opaque11() override {}
void Opaque13() override {}
void Opaque14() override {}
void Opaque15() override {}
void Opaque16() override {}
void Opaque25() override {}
void Opaque26() override {}
void Opaque27() override {}
void Opaque28() override {}
void Opaque29() override {}
void Opaque30() override {}
void Opaque31() override {}
void Opaque32() override {}
void Opaque33() override {}
void Opaque34() override {}
void Opaque35() override {}
void Opaque36() override {}
void Opaque37() override {}
void Opaque38() override {}
void Opaque39() override {}
void Opaque40() override {}
void Opaque41() override {}
void Opaque42() override {}
void Opaque43() override {}
void Opaque44() override {}
void Opaque45() override {}
void Opaque46() override {}
void Opaque47() override {}
void Opaque48() override {}
void Opaque49() override {}
void Opaque50() override {}
void Opaque51() override {}
void Opaque52() override {}
void Opaque53() override {}
void Opaque54() override {}
void Opaque55() override {}
void Opaque56() override {}
void Opaque57() override {}
void Opaque58() override {}
void Opaque59() override {}
void Opaque60() override {}
void Opaque61() override {}
void Opaque62() override {}
void Opaque63() override {}
void Opaque64() override {}
void Opaque65() override {}
void Opaque66() override {}

    double Barrier(double first,double second) override { output+=L'|';Mix(0xffffffffu);return first+second; }
    void SetFont(unsigned int value) override { font=value; }
    void SetColorValue(unsigned int color) override { SetColor(color&255,(color>>8)&255,(color>>16)&255,(color>>24)&255); }
    void SetColor(int red,int green,int blue,int alpha) override { r=red;g=green;b=blue;a=alpha; }
    void SetPos(int left,int top) override { x=left;y=top; }
    void GetPos(int& left,int& top) override { left=x;top=y; }
    void Print(const wchar_t* text,int length,int kind) override { for(int i=0;i<length;i++) Character(text[i],kind); }
    void Character(wchar_t unit,int) override {
        Mix(unit);Mix(static_cast<unsigned int>(x));Mix(static_cast<unsigned int>(y));Mix(font);Mix(r);Mix(g);Mix(b);Mix(a);
        output+=unit;auto dc=CreateCompatibleDC(nullptr);
        if(dc){ExtTextOutW(dc,0,0,0,nullptr,&unit,1,nullptr);DeleteDC(dc);} x+=10;
    }
    void Flush() override {}
    int FontTall(unsigned int fontValue) override { return GetMetrics()->Tall(fontValue); }
    int FontAscent(unsigned int fontValue,wchar_t unit) override { return GetMetrics()->Ascent(fontValue,unit); }
    bool Additive(unsigned int fontValue) override { return GetMetrics()->IsAdditive(fontValue); }
    void Abc(unsigned int fontValue,wchar_t unit,int& left,int& middle,int& right) override { GetMetrics()->GetAbc(fontValue,unit,left,middle,right); }
    int Width(unsigned int fontValue,int unit) override { return GetMetrics()->GetWidth(fontValue,unit); }
    void Measure(unsigned int fontValue,const wchar_t* text,int& width,int& height) override { GetMetrics()->Size(fontValue,text,width,height); }
} renderer;
extern "C" __declspec(dllexport) void* __cdecl CreateInterface(const char* name,int*) { return !strcmp(name,"VGUI_Surface031") ? &renderer : nullptr; }
extern "C" __declspec(dllexport) void __cdecl fixture_reset() { output.clear();trace=1469598103934665603ull; }
extern "C" __declspec(dllexport) const wchar_t* __cdecl fixture_output() { return output.c_str(); }
extern "C" __declspec(dllexport) unsigned long long __cdecl fixture_trace() { return trace; }
static void** originalTable=nullptr;
static std::vector<void*> invalidTable;
extern "C" __declspec(dllexport) void __cdecl fixture_wrong_abi() {
    originalTable=*reinterpret_cast<void***>(&renderer);
    invalidTable.assign(originalTable-1,originalTable+73);
    invalidTable[72]=originalTable[67]; // Width now has a one-argument callee.
    *reinterpret_cast<void***>(&renderer)=invalidTable.data()+1;
}
extern "C" __declspec(dllexport) void __cdecl fixture_restore_abi() { *reinterpret_cast<void***>(&renderer)=originalTable; }
