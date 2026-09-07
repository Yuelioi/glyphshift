// Synthetic MSVC class exports: checks calling conventions and decorated names,
// not Qt rendering or application compatibility. Compile without inlining.
#include <string>
#include <cstdint>
#define API __declspec(dllexport)
class QChar { public: unsigned short value; };
class API QString {
    std::u16string* value;
public:
    QString(const QChar* text, int length);
    ~QString();
    int size() const;
    const unsigned short* utf16() const;
};
QString::QString(const QChar* text, int length) : value(new std::u16string(reinterpret_cast<const char16_t*>(text), length)) {}
QString::~QString() { delete value; }
int QString::size() const { return static_cast<int>(value->size()); }
const unsigned short* QString::utf16() const { return reinterpret_cast<const unsigned short*>(value->c_str()); }
class QPointF { public: double x=0, y=0; };
class QRect { public: int x=0, y=0, width=0, height=0; };
class QRectF { public: double x=0, y=0, width=0, height=0; };
class QTextOption {};
static std::u16string drawn;
static int calls=0;
static void record(const QString& text) { drawn.assign(reinterpret_cast<const char16_t*>(text.utf16()), text.size()); ++calls; }
class API QPainter {
public:
    void drawText(const QPointF&, const QString&, int, int);
    void drawText(const QRect&, int, const QString&, QRect*);
    void drawText(const QRectF&, int, const QString&, QRectF*);
    void drawText(const QRectF&, const QString&, const QTextOption&);
};
void QPainter::drawText(const QPointF&, const QString& text, int from, int length) {
    record(text); drawn = drawn.substr(from, length < 0 ? std::u16string::npos : length);
}
void QPainter::drawText(const QRect&, int, const QString& text, QRect*) { record(text); }
void QPainter::drawText(const QRectF&, int, const QString& text, QRectF*) { record(text); }
void QPainter::drawText(const QRectF&, const QString& text, const QTextOption&) { record(text); }
extern "C" API const unsigned short* __cdecl fixture_draw(int kind, const unsigned short* text, int length) {
    QString source(reinterpret_cast<const QChar*>(text), length);
    QPainter painter;
    switch(kind) {
    case 0: painter.drawText(QPointF{}, source, 0, -1); break;
    case 1: painter.drawText(QRect{}, 17, source, nullptr); break;
    case 2: painter.drawText(QRectF{}, 29, source, nullptr); break;
    default: painter.drawText(QRectF{}, source, QTextOption{}); break;
    }
    return reinterpret_cast<const unsigned short*>(drawn.c_str());
}
extern "C" API int __cdecl fixture_calls() { return calls; }
