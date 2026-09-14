// Synthetic x64 fixture for SideFX CV_PaintBuffer::textWrappedInBox.
// It validates the exported MSVC member-function ABI and text argument only.
#include <string>

#define API extern "C" __declspec(dllexport)

class UT_DimRectImpl {};

template <typename T>
class UT_Rect {};

enum CV_HorizontalAlignment {
    CV_HorizontalAlignment_Left = 1,
};

enum CV_VerticalAlignment {
    CV_VerticalAlignment_Center = 1,
};

class UT_Color {};

static std::string drawn;

class __declspec(dllexport) CV_PaintBuffer {
public:
    void textWrappedInBox(
        const UT_Rect<UT_DimRectImpl>& rect,
        CV_HorizontalAlignment horizontal,
        CV_VerticalAlignment vertical,
        const char* text,
        UT_Color* color,
        double* scale,
        UT_Rect<UT_DimRectImpl>* bounds);
};

void CV_PaintBuffer::textWrappedInBox(
    const UT_Rect<UT_DimRectImpl>&,
    CV_HorizontalAlignment horizontal,
    CV_VerticalAlignment vertical,
    const char* text,
    UT_Color*,
    double*,
    UT_Rect<UT_DimRectImpl>*) {
    drawn = horizontal == CV_HorizontalAlignment_Left
            && vertical == CV_VerticalAlignment_Center
        ? text
        : "ABI violation";
}

API const char* fixture_draw() {
    CV_PaintBuffer buffer;
    UT_Rect<UT_DimRectImpl> rect;
    buffer.textWrappedInBox(
        rect,
        CV_HorizontalAlignment_Left,
        CV_VerticalAlignment_Center,
        "Open",
        nullptr,
        nullptr,
        nullptr);
    return drawn.c_str();
}
