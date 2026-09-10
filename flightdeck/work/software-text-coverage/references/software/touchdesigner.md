# TouchDesigner

当前决定：保留研究，暂不继续适配。重新开展时需用户明确恢复此项。

日期：2026-09-10。范围：官方资料与仓库入口对照；本页没有把资料调查当成实机替换验收。未注入、未修改目标，也未增加适配器。

## 目前能确定什么

TouchDesigner 主界面存在一条现有 Glyphshift 适配器没有实现的 **Slug 文字排版与渲染路径**。这比“它用了 Vulkan，所以不能翻译”更准确：GPU 最后负责画出来，但应寻找上游仍持有完整字符串的入口。

Derivative 的 2021.38110 发布说明直接指出：内置参数面板、节点名称和查看器、菜单及弹出窗口改用 Slug；其余界面内部改用 Text COMP。同一版把图形 API 从 OpenGL 换成 Vulkan。这是官方对主界面实现的描述，不是从某个 DLL 名称猜出来的。该说明属于版本迁移证据；本机具体版本及具体区域是否走相同分支仍需实测。[官方发布说明：High DPI Panel Rendering / Vulkan API](https://derivative.ca/release/experimental-202138110/65595)

当前官方图形文档仍将 2022 年前的 OpenGL 与之后的 Vulkan 分开说明。不能拿很老的 OpenGL 调用路径直接覆盖新版本。[Shader](https://derivative.ca/UserGuide/Shader)

## 必须分开验证的区域

| 区域 | 官方证据 | 对适配的意义 |
| --- | --- | --- |
| 编辑器主界面 | 上述发布说明明确列出 Slug 和内部 Text COMP | 优先调查共同文字排版入口，而不是只测用户工程里的一个文字节点。 |
| Text COMP | 用 Slug 画面板上的二维文字和图标 | 是建立小型复现的候选；复现成功不自动代表主界面全部成功。 |
| Geo Text COMP | 用 Slug 画三维场景中的文字与图标 | 需要单独考虑三维布局和可见刷新。 |
| Text TOP | 输出文字图像；Scalable 使用 Slug，另有 Bitmap、Polygon、Stroke、Automatic | 不同显示方式可能走不同上游入口，不能把所有 Text TOP 当作一个分支。 |
| Text SOP | 从字符串生成几何体，支持 Unicode | 不能仅凭名称相近就认定它也走 Slug；要独立验证。 |

表中 Text COMP、Geo Text COMP 的实现依据：[Slug Library](https://docs.derivative.ca/Slug_Library)。Text TOP 各显示方式依据：[Text TOP](https://docs.derivative.ca/Text_TOP)。Text SOP 的输入与输出依据：[Text SOP](https://docs.derivative.ca/Text_SOP)。

## 与现有适配器的差距

仓库原生入口包括 Win32 `TextOutW` / `ExtTextOutW` / `DrawTextW`、GDI+、Direct2D `DrawText` 和 DirectWrite layout；Qt Painter 拦截 `QPainter::drawText`，Qt Quick 保留并恢复标签文字。它们都要求目标实际调用各自的文字入口，不能因为目标最终显示在 Windows 窗口里就自动覆盖。

源码对照：[原生适配器](../../../../../crates/adapters/implementations/native/)、[Qt Painter](../../../../../crates/adapters/implementations/framework/qt-painter-native/src/lib.rs)、[Qt Quick](../../../../../crates/adapters/implementations/framework/qt-quick-native/src/lib.rs)。当前适配器实现目录未检出 Slug / CompileString / BuildSlug 入口。

没有找到足以证明 TouchDesigner 主界面使用 Qt Painter 或 Qt Quick 的官方材料。即使本机进程加载 Qt，也必须用调用证据确认它服务于哪个区域；不能据此宣称打开 Qt 适配器即可解决。类似地，系统文字 API 捕获到文件对话框或外围文字，也不能证明主界面已覆盖。

## 下一步候选：完整字符串进入 Slug 之前

Slug 官方手册公开了 `CompileString`、`CountSlug`、`BuildSlug` 等 API：存在先编译文字、再计算几何容量和生成几何的流程。手册也记录过 API 与中间结构的版本变化。因此候选应是**文字编译或布局入口**，不能只在最后生成顶点时替换原文；译文长度变化必须同步影响容量计算与生成结果。公开手册的签名不能直接当成本机已确认 ABI。[Slug User Manual](https://sluglibrary.com/SlugManual.pdf)

以下是研究方向，不是已经可用的实现：

1. 核对目标版本、模块及公开符号，确认能否定位 Slug 的完整字符串入口；若静态链接或缺少符号，先评估可维护性，不按软件名和固定地址硬补。
2. 用合成文字分别覆盖主界面、Text COMP、Text TOP 的显示方式与 Text SOP，建立实际调用对应关系。只记录确实经过入口的分支。
3. 检查编译结果和布局缓存归属：译文更新是否重新编译，启用前已生成的文字如何刷新，停止时如何恢复。捕获成功不等于实时替换成功。
4. 验证完整字符串、两代长短不同的译文、中文字体、换行、内嵌格式与停止恢复。显示标签与节点标识、表达式、代码必须区分，不能改写程序语义。

如果能建立稳定的 Slug 入口，应按文字技术做成通用能力，再评估其他采用同一库的目标。若只能通过 TouchDesigner 工程节点参数修改文字，那属于工程级集成，不能宣传成通用主界面适配。此轮不走 GPU 图像提取，也不把归档 UIA/OCR 引回产品。

## 首轮入口可行性检查

用户已授权初步测试。只读检查目标已加载模块的 PE 导出表及官方 Demo 二进制：没有找到 Slug 命名空间的 CompileString / CountSlug / BuildSlug 导出入口。命中的 Python CompileString 已排除，不能作为 Slug 证据。目标核心库内识别到 Slug 着色器以及 GX_SlugText、UI_TextMultiSlugView 等 RTTI 类型，支持“目标集成了 Slug”的判断；这些封装类型不等于库级通用接口。

官方 Demo 的多个图形后端也没有导出该文字编译入口，未附公开调试符号。当前测试只判定“按公开符号定位的路径不成立”，不判定静态特征识别或进一步逆向必然不可行。本地可重跑的入口检查返回 NO_EXPORTED_SEAM；可复用的字符串捕获、中文替换、两代刷新和停止恢复均尚未通过。

下一阶段若继续，需要从编译后的静态库识别语义和调用约定，在官方 Demo 与目标中交叉验证；TouchDesigner 自有 GX/UI 封装只能辅助理解，不能直接当作通用 Slug 适配器。未调用签名不明的函数，也未修改目标工程或正式适配器。二进制、官方 Demo 下载包、扫描脚本和原始结果仅保存在本地忽略目录。
