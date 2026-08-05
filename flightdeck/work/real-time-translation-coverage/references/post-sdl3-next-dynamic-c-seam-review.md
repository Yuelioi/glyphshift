# SDL3_ttf 之后的动态 C 文字入口复核

Status: Complete

Date: 2026-08-06

## 结论

本轮比较 Allegro 5 的 `al_draw_text` / `al_draw_justified_text` / `al_draw_multiline_text` 与 raylib 的
`DrawText` / `DrawTextEx`。**当前都不进入生产 Runtime Bundle，也不新增用户可选 Adapter。**

- **raylib：字符串 seam 条件 Go；中文可见性与生产 Adapter No-Go。** 固定版本的公开动态 C ABI 能在每次
  绘制取得完整 UTF-8，并可只替换本次调用的字符串；Musializer 也提供项目自有、可重复的 Windows
  hot-reload 动态构建。但该真实目标只把 95 个 ASCII codepoints 烘焙进 `Font` atlas，中文译文必然回退成
  `?`。仅挂字符串入口不能完成中文实时写回，必须先做独立 fallback Font/atlas 原型并闭环渲染线程、
  GPU context 和销毁生命周期。
- **Allegro 5：No-Go。** 公开入口与临时替换语义成立，Open Surge Engine 是唯一值得保留的外部真实候选；
  但本轮一手资料尚不能证明其 Windows 发布产物实际导入并加载 `allegro_font*.dll`，也未证明所选可见文字
  一定经过 TTF driver。源码中的默认动态构建意图不能替代实际产物与 hook 命中。

至此停止继续枚举同类框架。**下一议程优先选择 Allegro/Open Surge 的一次有界本机 Go/No-Go**：先检查
发布包或显式动态构建的 PE imports、运行模块与实际 hook hit，再验证一个明确走 TTF driver 的词条及可见
字形。它通过后才值得实现 Allegro Adapter；失败即收口。raylib/Musializer 因只有开发态动态构建且现有
atlas 仅含 ASCII，排在第二位，必须先做 fallback Font 原型，不能先写字符串 Adapter。

## 硬门槛与结果

| 门槛 | Allegro 5 | raylib 5.5 |
| --- | --- | --- |
| 稳定公开动态 C ABI | 通过：公开头文件、`extern "C"`、Windows DLL import/export 与 `__cdecl` | 通过：`RLAPI`、`extern "C"`，官方 CMake 支持 shared library |
| 每次绘制取得完整 UTF-8 | 通过，但 Open Surge 取得的是预处理后的完整单行/单色 segment，不保证是整条富文本消息 | 通过：`const char *` 在入口仍完整，`DrawTextEx` 随后逐 UTF-8 codepoint 解码 |
| 临时替换且不改业务状态 | 通过：同步调用期间可换成独立 NUL 结尾缓冲 | 字符串层通过：同步调用期间可换指针；中文还必须临时改用自有 `Font` |
| 首代、同进程二代、停用恢复 | 语义成立，尚未做真实目标可见验收 | 字符串语义成立；中文 fallback Font/cache 生命周期尚未证明 |
| 外部开源真实软件 | Open Surge Engine v0.6.1.3 | Musializer 固定 commit `4d7d2f` |
| 可重复 Windows 构建/发布 | 有 Windows 构建与发布；默认配置意图动态链接 | hot-reload 源码构建明确生成 `raylib.dll`；公开分发构建明确静态链接 |
| 实际动态链接闭环 | 未通过：缺发布 EXE 的 PE import、运行模块和 export hook 命中 | 源码配置闭环；仍需本机 PE/module/export 验证，公开发布包不适用 |
| 中文可见性 | 未闭环 | 未通过：目标 Font atlas 只有 U+0020–U+007E，缺字回退 `?` |
| 本轮总判定 | **No-Go** | **字符串 seam 条件 Go；生产 Adapter No-Go** |

## raylib：调用时字符串 seam 成立

raylib 5.5 的公开头文件导出 `DrawText`、`DrawTextEx` 和 `DrawTextPro`。Windows shared build 通过
`BUILD_LIBTYPE_SHARED` 使用 DLL export/import；C++ 包装在 `extern "C"` 中。官方 CMake 的
`BUILD_SHARED_LIBS` 分支建立 shared library、定义对应宏，并只公开 `RLAPI` 标记的符号。这个结论只覆盖
固定的公开 ABI；`Font` 结构按值传递，因此实现必须与所支持 raylib ABI 精确匹配，不能猜测任意版本。

官方实现显示：

1. `DrawText` 把原始 `const char *text` 原样转给 `DrawTextEx`；
2. `DrawTextEx` 先计算完整字节长度，再在循环中调用 `GetCodepointNext(&text[i], ...)` 解码 UTF-8；换行也在
   此入口内部处理；
3. `DrawTextPro` 同样把完整字符串交给 `DrawTextEx`。

因此对直接调用这些公开入口的动态目标，Adapter 可在一次同步调用期间以独立 UTF-8 缓冲调用原函数，保留
位置、字号、间距、颜色和旋转参数，返回后释放缓冲；不修改应用持有的字符串。每帧重新查询不可变
Dictionary generation，理论上自然支持第一代、第二代与停用透传。内部转调要求实现重入保护；绕过这些
入口、直接调用 `DrawTextCodepoint(s)` 或复制实现的目标不在覆盖范围。

第一方资料：

- [raylib 5.5 公开头文件](https://github.com/raysan5/raylib/blob/5.5/src/raylib.h)
- [raylib 5.5 文字实现](https://github.com/raysan5/raylib/blob/5.5/src/rtext.c#L1131-L1199)
- [raylib 5.5 shared-library CMake](https://github.com/raysan5/raylib/blob/5.5/src/CMakeLists.txt#L57-L91)

## raylib 真实候选：Musializer 的动态构建成立，但字体门槛失败

Musializer 是外部开源音乐可视化应用，不是 raylib 官方示例。固定源码中存在稳定、用户可见的
`DrawTextEx` 标签，例如文件选择提示与错误/渲染状态文字；因此它能证明真实应用代码确实使用目标入口。

项目自己的 Windows 构建脚本给出了清楚边界：

- 启用 `MUSIALIZER_HOTRELOAD` 时，MSVC 与 MinGW 路径都生成 `raylib.dll`，并让 `libplug.dll`/主程序链接
  该动态库；这是可重复的项目自有动态源码构建。
- 普通构建使用静态 `raylib.lib` / `libraylib.a`；distribution 分支明确拒绝 hot reload。因此公开发布包
  不能作为动态 Adapter 目标，探测时必须依据进程实际加载模块与导出表 fail closed。

真正阻断中文的是字体。Musializer 调用：

```c
LoadFontFromMemory(..., FONT_SIZE, NULL, 0)
```

raylib 5.5 在 `codepoints == NULL && codepointCount == 0` 时固定生成 95 个连续 codepoints，从 U+0020 开始，
即 U+0020–U+007E。`GetGlyphIndex` 对 atlas 中不存在的字符回退到 `?`。所以即便 `DrawTextEx` hook 正确把
英文换成中文，Musializer 原 `Font` 也只能画出问号；字体文件本身是否包含中文字形已经不影响这个结论。

这使“字符串 hook 单独 Go”不成立。重新打开 raylib 生产立项前，fallback Font 原型至少必须证明：

1. 在有效 GPU context 的渲染线程创建仅含所需中文 codepoints 的自有 `Font`/atlas；
2. `DrawText`、`DrawTextEx` 和需要覆盖的 `DrawTextPro` 能在不修改应用 `Font` 的情况下，按本次调用临时
   选择 fallback Font，并保留布局参数；缺字或 ABI 不符时原样放行；
3. 字典第二代新增字符时能安全增量/重建 cache，旧帧不引用已释放 atlas；
4. 停用立即恢复原字符串与原 Font，context 关闭或重建前能在正确线程安全销毁 GPU 资源；
5. 在 Musializer hot-reload 动态构建中同时验证 PE import、`raylib.dll` 实际加载、导出命中和可见中文像素。

在这些条件通过前，不建立 raylib crate、Catalog 项或 Runtime Bundle 条目。

候选项目一手资料：

- [Musializer 项目与构建说明](https://github.com/tsoding/musializer/tree/4d7d2fa849ef66e94ce03a53a2e7aa3e36aa2392)
- [真实 `DrawTextEx` 调用](https://github.com/tsoding/musializer/blob/4d7d2fa849ef66e94ce03a53a2e7aa3e36aa2392/src/plug.c#L1572-L1592)
- [Musializer 字体加载与资源生命周期](https://github.com/tsoding/musializer/blob/4d7d2fa849ef66e94ce03a53a2e7aa3e36aa2392/src/plug.c#L1821-L1859)
- [MSVC 动态/静态与 distribution 分支](https://github.com/tsoding/musializer/blob/4d7d2fa849ef66e94ce03a53a2e7aa3e36aa2392/src_build/nob_win64_msvc.c)
- [MinGW 动态/静态与 distribution 分支](https://github.com/tsoding/musializer/blob/4d7d2fa849ef66e94ce03a53a2e7aa3e36aa2392/src_build/nob_win64_mingw.c)
- [raylib 默认 95 glyph 的实现](https://github.com/raysan5/raylib/blob/5.5/src/rtext.c#L527-L576)
- [raylib 默认 codepoint 范围](https://github.com/raysan5/raylib/blob/5.5/src/rtext.c#L635-L645)
- [raylib 缺字回退 `?`](https://github.com/raysan5/raylib/blob/5.5/src/rtext.c#L1339-L1369)

## Allegro 5：API 合同可用，真实动态目标尚未闭环

Allegro 5.2.11.3 的 `al_draw_text`、`al_draw_justified_text` 与 `al_draw_multiline_text` 位于公开头文件，
不在 unstable guard 中；Windows DLL 宏提供 import/export 与 `__cdecl`，C++ 调用方仍使用 C linkage。
官方实现中三个 C-string 入口都先收到完整 NUL 结尾字符串：普通入口创建临时 `ALLEGRO_USTR` 引用；
justified 与 multiline 也是在公开入口之后才拆词或拆行。由此可推断，hook 可只替换当前调用的字符串指针，
同步调用返回后释放，不写入业务对象；按帧重新决策可支持代际更新和停用透传。

覆盖必须保持克制：`*_ustr` 是另一组公开入口，`*textf` 也不保证再次经过 `al_draw_text`；仅挂本轮三个
C-string 函数不能宣称覆盖所有 Allegro 文字。

第一方资料：

- [Allegro 5.2.11.3 font addon 公开头文件](https://github.com/liballeg/allegro5/blob/5.2.11.3/addons/font/allegro5/allegro_font.h)
- [Allegro 5.2.11.3 文字实现](https://github.com/liballeg/allegro5/blob/5.2.11.3/addons/font/text.c)
- [Allegro 5.2.11.3 font addon 文档](https://liballeg.org/a5docs/5.2.11.3/font.html)

### 唯一保留候选：Open Surge Engine v0.6.1.3

Open Surge 是外部开源游戏引擎/游戏。其 `fontdrv_ttf_textout` 在阴影和正文路径直接调用
`al_draw_text`；`font_render` 每帧遍历预处理后的 segment 并交给 driver 绘制。TTF driver 不缓存整段文字
texture，因而绘制时入口具备热更新语义。

项目构建资料显示 `ALLEGRO_STATIC` 与 `ALLEGRO_MONOLITH` 默认关闭，Windows executable 链接独立
`allegro_font` / `allegro_ttf` 等库，只有静态分支才追加 `-static`；项目也提供 Windows 构建说明和正式发布。
这些证据足以把它保留为 smoke 候选，但不足以越过生产门槛：

1. 未检查发布 EXE 的 PE import，未证明运行时实际加载 `allegro_font*.dll`，也未取得 export hook 命中；
2. Open Surge 在 Allegro 入口之前已把复杂消息预处理成单行、单色 segment。每次调用的 UTF-8 segment 是
   完整的，但不等于完整富文本业务消息；首测必须选择精确匹配的单行单色词条；
3. 项目同时存在 bitmap 与 TTF font driver，本轮未证明选定的默认可见 UI 词条一定走 TTF driver。

因此 Allegro 保持 No-Go。若后续获准进行一次有界本机 smoke，只检查：显式
`ALLEGRO_STATIC=OFF; ALLEGRO_MONOLITH=OFF` 构建或项目发布包的 PE import、运行模块/导出，以及一个明确
TTF 单行词条的首代、二代、停用可见结果；不满足任一项即收口。

候选项目一手资料：

- [Open Surge Engine v0.6.1.3](https://github.com/alemart/opensurge/tree/v0.6.1.3)
- [Open Surge 真实字体绘制路径](https://github.com/alemart/opensurge/blob/v0.6.1.3/src/core/font.c)
- [Open Surge 构建选项](https://github.com/alemart/opensurge/blob/v0.6.1.3/CMakeLists.txt)
- [Open Surge Allegro 链接选择](https://github.com/alemart/opensurge/blob/v0.6.1.3/src/misc/cmake/libs.cmake)
- [Open Surge Windows 构建说明](https://github.com/alemart/opensurge/blob/v0.6.1.3/README.md)
- [Open Surge v0.6.1.3 发布](https://github.com/alemart/opensurge/releases/tag/v0.6.1.3)

## 明确排除

- 静态链接 raylib/Allegro 的目标；
- 文字已进入业务 texture、bitmap font 或其他缓存，绘制入口不再取得 UTF-8 的路径；
- 依赖软件品牌、EXE 名称、版本签名或私有地址表的适配；
- 只证明模块存在、导出存在或 Dictionary 命中，却没有真实入口与可见像素的结果；
- 以官方示例替代外部真实软件门槛。
