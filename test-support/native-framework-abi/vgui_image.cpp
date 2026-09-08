#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <string>
#include "vgui_surface.h"
static Surface* surface=nullptr;
namespace vgui {
struct IImage { virtual void Paint()=0;virtual ~IImage(){}; };
class TextImage:public IImage {
public:
#ifdef DIFFERENT_OBJECT_LAYOUT
    unsigned char padding[61]{};
#endif
    std::wstring text;int mode;
    TextImage(const wchar_t* value,int valueMode):text(value),mode(valueMode){}
    __declspec(noinline) void Paint() override {
        surface->SetFont(7);
        if(mode==4)surface->SetColorValue(0xff21160b);
        else surface->SetColor(11,22,33,255);
        int x=5;
        for(size_t index=0;index<text.size();index++) {
            if(mode==1 && index==2) {
                SetLastError(123);
                if(surface->Barrier(1.25,2.5)!=3.75 || GetLastError()!=123) {surface->Print(L"BAD ABI",7,0);return;}
            }
            if(mode==2 && index==2)x+=20;
            if(mode==3 && index==2)surface->SetColor(50,60,70,255);
            surface->SetPos(x,20);surface->Character(text[index],0);
            x+=surface->Width(7,text[index]);
        }
    }
};
}
class DecoratedTextImage final:public vgui::TextImage {
    unsigned char additionalState[37]{};
public:
    using TextImage::TextImage;
};
extern "C" __declspec(dllexport) void* __cdecl CreateInterface(const char*,int*) { return nullptr; }
extern "C" __declspec(dllexport) void __cdecl fixture_paint(const wchar_t* text,int mode) {
    if(!surface) {
        auto module=GetModuleHandleW(L"vguimatsurface.dll");
        auto factory=reinterpret_cast<void*(__cdecl*)(const char*,int*)>(GetProcAddress(module,"CreateInterface"));
        surface=static_cast<Surface*>(factory("VGUI_Surface031",nullptr));
    }
    if(mode>=8 && mode<=13) {
        surface->PushMakeCurrent(42,true);
        if(mode==10)surface->PushMakeCurrent(43,true);
        if(mode==11)for(unsigned int i=0;i<34;i++)surface->PushMakeCurrent(100+i,true);
        surface->SetFont(7);surface->SetColorValue(0xff21160b);
        if(mode==12){surface->SetPos(5,0);surface->Print(L"Label",5,0);}
        int x=5;
        if(mode==13)surface->SetPos(x,20);
        for(size_t i=0;i<wcslen(text);i++) {
            if(mode!=13)surface->SetPos(x,20);
            surface->Character(text[i],0);x+=surface->Width(7,text[i]);
        }
        if(mode==10)surface->PopMakeCurrent(43);
        if(mode==11)for(unsigned int i=34;i>0;i--)surface->PopMakeCurrent(99+i);
        surface->PopMakeCurrent(mode==9?99:42);
        return;
    }
    if(mode==5 || mode==6) {
        surface->SetFont(7);surface->SetColorValue(0xff21160b);surface->SetPos(5,20);
        if(mode==6)surface->Barrier(1.25,2.5);
        surface->Print(text,static_cast<int>(wcslen(text)),0);
        return;
    }
    if(mode==7){DecoratedTextImage image(text,0);vgui::IImage* instance=&image;instance->Paint();return;}
    vgui::TextImage image(text,mode);vgui::IImage* instance=&image;instance->Paint();
}
