# JUCE：下一个通用文字 Adapter 选取

结论：**当前 JUCE 6.1.6 canonical 三入口 profile 为 No-Go。** 静态可行性门槛已经完成，
`addCurtailedLineOfText` 与 `addFittedText` 都能唯一定位，但没有找到可证明的 canonical
`addJustifiedText`，也没有保留下来的 `addLineOfText` 薄包装。因此按预先约定的 fail-closed 规则，
本轮不进入 observation-only Hook，更不建立生产 Adapter。

本轮从 JUCE 6.1.6 x64 profile 开始，因为当前已有一个真实目标能证明“静态 JUCE 存在、现有通用文字
Adapter 0 命中”，同时 JUCE 的标准文字 API 在完整 `juce::String` 仍存在时进入布局。结果证明“部分
canonical 函数可稳定恢复”，但不足以满足计划中的三入口完整覆盖门槛。

## 本轮静态可行性结果

使用官方 JUCE 6.1.6 源码按 Windows x64 MSVC Release 形态建立对照，再对真实静态 PE 使用版本标记、
RTTI、`.pdata` 函数边界和归一化指令语义做 fail-closed 匹配。可移植结果如下：

- 精确 `JUCE v6.1.6` 标记、`LowLevelGraphicsContext` 与 `WindowsDirectWriteTypeface` RTTI 均存在；
- `GlyphArrangement::addCurtailedLineOfText`：**1 个**唯一语义候选；
- `GlyphArrangement::addFittedText`：**1 个**唯一语义候选；
- canonical `GlyphArrangement::addJustifiedText`：**0 个**候选；
- `GlyphArrangement::addLineOfText` 薄包装：**0 个**候选；
- 能找到包含 CR/LF 分行、32-byte glyph 记录和 curtailed 调用的代码，但其接收的是调用者在栈上创建的
  `GlyphArrangement`，属于上层函数中的内联 / 组合逻辑，不能冒充稳定的 canonical JUCE seam；
- 非 JUCE PE 会在版本 / RTTI gate 直接拒绝。

因此静态判定为 `curtailed=1 / fitted=1 / justified=0 / line-wrapper=0 -> NO_GO`。由于第一道门槛失败，
本轮没有 attach、注入或 observation-only Hook。

## 为什么当时先验证 JUCE

进入本轮验证前，候选按“通用性 + 完整字符串 seam + 真实目标 + 可验证性”排序如下；JUCE 已由本页前述
静态结果从候选 Go 降为当前 profile No-Go：

1. **JUCE：验证前 Go，当前 6.1.6 canonical profile No-Go。** 有真实授权目标，JUCE 版本可识别，标准
   Graphics / GlyphArrangement 路线持有完整 `String`；实际静态链接产物没有保留计划要求的全部 callable
   canonical 入口。
2. **SDL3_ttf：保留 Roadmap。** `TTF_Text` + `TTF_Draw*Text` 技术 smoke 已 Go，但项目此前没有找到满足
   动态公开绘制链的代表性第三方真实目标；现在建设生产 Adapter 会缺真实收益门槛。
3. **Slug：继续研究，不先实现。** 已确认文字技术存在，但公开 `CompileString` / `BuildSlug` 等符号 seam
   在目标和官方 Demo 中没有形成可调用边界，当前仍需要静态语义识别。
4. **ZBrush：后置。** 已观测到的是 glyph / 纹理缓存下游；完整字符串上游 seam 尚未找到。

Blender 官方 Python 翻译 API 有明确能力，但它是 Blender 自身集成接口，不符合当前阶段“只新增通用框架 /
渲染文字 Adapter”的策略，因此不参加这一轮通用 Adapter 排序。

## JUCE 6.1.6 的完整字符串路径

JUCE 6.1.6 的 `Graphics` 标准文字函数在布局前都仍持有完整 `const String&`：

- `Graphics::drawSingleLineText` 创建 `GlyphArrangement` 后调用 `addLineOfText`；
- `Graphics::drawMultiLineText` 调用 `addJustifiedText`；
- `Graphics::drawText(Rectangle<float>)` 直接调用 `addCurtailedLineOfText`；
- `Graphics::drawFittedText` 调用 `addFittedText`。

同版本 `GlyphArrangement::addLineOfText` 再进入 `addCurtailedLineOfText`，`addJustifiedText` 与
`addFittedText` 也继续在内部复用完整字符串布局流程。这比 Hook `LowLevelGraphicsContext::drawGlyph` 更合适：
后者已经只剩 glyph number，不再能可靠恢复完整原文。

一手源码：

- JUCE 6.1.6 `Graphics`：<https://github.com/juce-framework/JUCE/blob/6.1.6/modules/juce_graphics/contexts/juce_GraphicsContext.cpp>
- JUCE 6.1.6 `GlyphArrangement`：<https://github.com/juce-framework/JUCE/blob/6.1.6/modules/juce_graphics/fonts/juce_GlyphArrangement.cpp>
- JUCE 当前 `Graphics` 文档：<https://docs.juce.com/master/classjuce_1_1Graphics.html>
- JUCE 当前 `GlyphArrangement` 文档：<https://docs.juce.com/master/classjuce_1_1GlyphArrangement.html>

### 6.1.6 的首选 Hook 家族

第一版 profile 优先发现三个 canonical 入口：

1. `GlyphArrangement::addCurtailedLineOfText`：覆盖 `drawText`、`drawSingleLineText` 与直接
   `addLineOfText` 的常见单行路径；
2. `GlyphArrangement::addJustifiedText`：在多行拆分前保留完整源字符串；
3. `GlyphArrangement::addFittedText`：在 fitted / 换行处理前保留完整源字符串。

后两个入口需要重入保护：若外层已经对完整字符串做过观察 / 替换，内部再次进入
`addLineOfText -> addCurtailedLineOfText` 时只走原布局，避免重复观察和二次翻译。

不把 `Font::getGlyphPositions` 作为替换 seam。它虽然仍收到 `String`，但调用方随后还会继续按原始
`text` 读取字符、空白和长度；只替换 glyph position 输入会让“字形数组”和“原字符串”失配，长译文尤其
不安全。

## 为什么必须做版本 profile

当前 JUCE 主线仍保留 `Graphics` / `GlyphArrangement` 的公开完整字符串 API，但内部实现已经改为
`ShapedText`。当前源码中 `addCurtailedLineOfText`、`addJustifiedText`、`addFittedText` 分别创建 / 使用
`ShapedText`，不再像 6.1.6 那样都汇入同一个旧布局链。

因此 Adapter ID 可以保持 JUCE 技术语义，但 locator 必须按精确版本 / ABI 建 profile；不能把 6.1.6 的
机器码形态或内部调用图通配到 JUCE 7/8/后续版本。

当前 `GlyphArrangement` 一手源码：
<https://github.com/juce-framework/JUCE/blob/master/modules/juce_graphics/fonts/juce_GlyphArrangement.cpp>

## 静态链接下的 fail-closed 条件

JUCE 通常静态链接，没有稳定 DLL 导出可直接解析。仓库已有 CatSystem2 的先例：从 PE、RTTI 与指令结构
恢复窄 profile，并在歧义时拒绝激活。JUCE 应沿用同一原则，但 fingerprint 必须只描述 JUCE，不包含软件名、
进程名或样本固定地址。

JUCE 6.1.6 首个 profile 至少要求同时满足：

- Windows x64 与支持的 MSVC ABI；
- 精确 `JUCE v6.1.6` 版本标记；
- 预期 JUCE RTTI / 类型族存在，例如 `LowLevelGraphicsContext`，并与实际图形实现组成一致的交叉证据；
- 三个 canonical 布局入口只能各解析出一个候选；
- 候选间调用结构与 6.1.6 一手源码一致；
- 任一入口缺失、被 LTO 完全内联、出现多个无法消歧的候选或 `String` ABI 无法证明时，整个 profile
  fail-closed。

单独看到 `DWrite.dll`、`WindowsDirectWriteTypeface` 或 `LowLevelGraphicsSoftwareRenderer` 不能判定文字
一定经过某个 seam；真实入口命中仍必须由 observation-only 探针证明。

## 替换所有权与验证门槛

第一阶段先做只观察 locator / hook，要求真实界面稳定采到完整标签，并确认没有把文件路径、内部状态或逐 glyph
数据误当正文。进入替换阶段后：

- 不修改调用方持有的 `juce::String`；为一次布局调用构造 profile 已验证的临时译文 `String`，调用原实现后
  立即按 JUCE 所有权规则释放；
- 首代译文、同进程第二代长短不同译文、停用后的原文恢复都必须可见；
- 中文必须验证字体 fallback / shaping，不能只证明 ASCII 替换；
- 单行、多行、裁剪、fitted 至少各有一个确定性合同；
- 复杂 `AttributedString` / `TextLayout`、自定义 glyph、OpenGL/自建纹理缓存不从这一 profile 自动推导，
  未证明时保持原路径。

## 下一动作

停止当前 JUCE 6.1.6 canonical 三入口路线并回到通用 Adapter 候选选择。只有出现以下任一新证据时才重开：

1. 能按 JUCE 技术结构而非软件名 / 固定地址唯一识别另一个仍持有完整字符串的可调用 seam；
2. 另一个精确 JUCE 版本 / ABI 的真实目标保留了可证明的 canonical 完整字符串入口；
3. 能证明当前内联布局路径存在跨 JUCE 软件稳定、可安全替换且可 fail-closed 的通用结构边界。

否则不为 Vovious 建软件专属地址 profile，也不把两个已找到的入口单独包装成“JUCE 已支持”。
