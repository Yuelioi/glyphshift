# raylib 之后的下一 Adapter 第一方资料复核

Status: Complete

Date: 2026-08-06

## 结论

本轮没有候选同时满足“可重复的外部 Windows 真实目标、目标实际加载公开动态文字入口、每次绘制取得
完整原文、临时替换后不改变目标持久状态、停用可立即恢复”五项门槛，因此不进入本机 Hook smoke，也
不新增生产 crate、Catalog 或 Runtime Bundle 项。

FLTK 是三个候选中最值得等待真实目标触发的入口；SFML 与 Dear ImGui 在当前真实候选中均由静态或
缓存几何路径否决。UI Automation 仍只有结构化观察价值，不属于本评审的写回候选。

## 候选对照

| 候选 | 完整原文入口 | 动态 ABI / 真实目标 | 写回与恢复 | 本轮判定 |
| --- | --- | --- | --- | --- |
| FLTK 1.4 `fl_draw` | 公开 UTF-8 draw overload 可取得完整字符串；带边界框的入口还处理换行、折行与标签 | Windows shared build 是可选项且默认关闭；MSVC DLL 必须与目标编译器版本匹配；当前没有可重复外部动态目标 | 绘制时临时替换原则上可行，但必须覆盖目标实际使用的 overload，并验证字体与换行 | **Wait**：最优后续候选，但不进入空置原型 |
| SFML 3 `sf::Text` | `setString` 持有原文；最终 draw 只提交缓存后的顶点 | Windows 二进制要求编译器完全匹配；当前 Open Hexagon 候选显式静态构建 SFML | Hook setter 会改变 retained object；下游 draw 已无字符串。临时改写对象还要两次重建 geometry 与字体 atlas | **No-Go**：没有窄的绘制时临时字符串 seam |
| Dear ImGui | `ImDrawList::AddText` 可同步消费 UTF-8，但 renderer backend 只收到顶点和索引 | 官方是源码集成；当前 Tracy 目标把 `TracyImGui` 构建为静态库 | 入口不是可依赖的公共 DLL ABI；下游无原文，CJK 仍受目标 font atlas 约束 | **No-Go**：真实目标没有动态文字入口 |

## FLTK

- FLTK 1.4.5 的公开绘制文档把 `fl_draw(const char*, ...)` 定义为 UTF-8 文字入口；简单 overload 接收
  NUL 结尾完整字符串，边界框 overload 用于所有 label，并支持多行与布局处理。[绘制 API](https://www.fltk.org/doc-1.4/group__fl__drawings.html)
- 头文件把简单和边界框 overload 标记为 `FL_EXPORT`，但带显式 byte length 的 overload 是 inline，
  会直接分派到当前 graphics driver；单挂一个符号不能声称覆盖所有 FLTK 文字路径。
  [fl_draw.H](https://github.com/fltk/fltk/blob/release-1.4.5/FL/fl_draw.H#L977-L1060)
- 边界框实现会先展开、换行和分段，再调用更低层绘制函数；如果未来真实目标确实导入该 overload，
  Hook 能在预处理前看到完整 label，但仍需实测目标使用的是哪一个导出。
  [fl_draw.cxx](https://github.com/fltk/fltk/blob/release-1.4.5/src/fl_draw.cxx#L181-L215)
- 官方 CMake 的 `FLTK_BUILD_SHARED_LIBS` 默认是 `OFF`。官方 Windows 说明还明确指出，因编译器名称
  修饰差异，FLTK DLL 只能与生成它的同版本编译器配合使用。
  [构建选项](https://github.com/fltk/fltk/blob/release-1.4.5/CMake/options.cmake#L438-L442)、
  [Windows DLL 说明](https://www.fltk.org/doc-1.4/intro.html)

因此 FLTK 的 API 形状为条件 Go，但当前缺少能证明 PE import、运行模块、实际导出与固定可见词条命中
的外部真实目标。下一次只有在这样的目标出现后才恢复评审；不先建设无人能激活的 Adapter。

## SFML

- SFML 3.1 的 `sf::Text` 是导出的 C++ class，公开 `setString` / `getString`，内部保留 `m_string` 与
  vertex arrays。[Text.hpp](https://github.com/SFML/SFML/blob/3.1.0/include/SFML/Graphics/Text.hpp#L60-L181)、
  [成员状态](https://github.com/SFML/SFML/blob/3.1.0/include/SFML/Graphics/Text.hpp#L789-L810)
- `Text::draw` 先确保 geometry 已更新，之后只把 outline 与 fill vertices 交给 RenderTarget；如果
  geometry 和字体纹理没有变化，更新函数会直接返回。渲染下游因此不再携带原始字符串。
  [Text.cpp](https://github.com/SFML/SFML/blob/3.1.0/src/SFML/Graphics/Text.cpp#L741-L777)
- 官方 Windows 下载页要求预编译库与编译器版本完全匹配，这不是可跨任意目标依赖的稳定 C ABI。
  [SFML 3.1.0 下载说明](https://www.sfml-dev.org/download/sfml/3.1.0/)
- 当前可重复外部候选 Open Hexagon 在自己的 CMake 中设置 `BUILD_SHARED_LIBS false` 与
  `SFML_STATIC_LIBRARIES true`，随后把 SFML targets 链入静态项目库。
  [Open Hexagon CMake](https://github.com/vittorioromeo/SSVOpenHexagon/blob/master/CMakeLists.txt#L91-L107)

CSFML 虽提供官方 C binding，但当前候选集没有实际加载该动态库的外部真实目标；它对 retained Text
对象的包装也不会改变上述 source-to-geometry 生命周期。本轮不因存在一套 C API 就跳过真实目标门槛。

## Dear ImGui

- Dear ImGui 官方头文件的 `IMGUI_API` 默认不产生平台导出；其公共说明面向把源码直接加入 C++ 项目，
  不是稳定的共享库文字 ABI。[imgui.h](https://github.com/ocornut/imgui/blob/v1.92.5/imgui.h)
- `ImDrawList::AddText` 是最接近完整 UTF-8 draw-time seam 的入口，但 font atlas 负责字形，renderer
  backend 最终只消费 draw lists 的顶点与索引；越过该层便失去原文。
  [ImDrawList API](https://github.com/ocornut/imgui/blob/v1.92.5/imgui.h#L3144-L3157)、
  [Renderer backend contract](https://github.com/ocornut/imgui/blob/v1.92.5/backends/imgui_impl_dx11.cpp)
- 当前可重复 Windows 候选 Tracy 0.13.1 固定 Dear ImGui docking 版本，并把 `TracyImGui` 声明为
  `STATIC`；正式发布因此不能提供可挂接的 ImGui 文字 DLL。
  [Tracy vendor build](https://github.com/wolfpld/tracy/blob/v0.13.1/cmake/vendor.cmake#L124-L150)、
  [Tracy 0.13.1](https://github.com/wolfpld/tracy/releases/tag/v0.13.1)

## 恢复条件

只在出现新的、明确授权且可重复的 Windows 目标时恢复本切片。优先顺序为：

1. 先查 PE import 和运行模块，确认目标实际加载 FLTK shared library 或另一条尚未覆盖的公开动态入口。
2. 再确认目标导入的具体文字符号能在一次调用中提供完整原文，而不是缓存后的 glyph/vertex/texture。
3. 只有固定词条真实命中后，才做第一代、第二代、停用恢复与字体资源生命周期的有界 smoke。
