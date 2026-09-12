# AE 25.0／2025 界面与文字捕获：官方来源研究

检索日期：2026-09-12。本页仅依据 Adobe 官方发布、帮助、开发者资料、可确认身份的员工声明及 Microsoft API 文档；本轮实机结果与根因另见[采集诊断](capture-diagnosis.md)。

**已证实：改版及其范围**

Adobe 在 2024-04-11 的发布文中明确说明 AE Beta 采用 Spectrum 设计系统，且该 UI 在 Windows 上使用 GPU 加速；2024 年 10 月发布汇总将现代化界面列为 AE 25.0 新功能。[NAB 官方发布](https://blog.adobe.com/en/publish/2024/04/11/adobe-after-effects-beta-introduces-new-3d-workflows-user-requested-features-time-for-nab-2024)、[25.0 官方发布汇总](https://blog.adobe.com/jp/publish/2024/10/16/cc-creativecloud-october-2024-update-list)。

**员工声明**：Peter Antal 于 2024-12-16 明确称 AE 的硬件加速 UI 经公开 Beta 后在 v25 正式发布。同页对 Direct2D 资源使用及回退的具体说明针对 Premiere，不能外推为 AE 全部文字的实现。[员工原文](https://community.adobe.com/bug-reports-728/p-hundreds-of-direct2d-drawbot-error-hresult-2005270494-1331150/index3.html)。

Spectrum 是跨应用、跨平台的设计系统；Adobe 开发者门户则介绍插件 SDK 的内置和自定义参数控件。**研究推论**：设计系统、插件绘制 API、主程序底层渲染应分别取证，前两者不能单独确定第三者，更不能证明捕获故障因果。[Spectrum 官方说明](https://blog.adobe.com/en/publish/2018/05/15/introducing-spectrum-adobe-building-design-system-scale)、[AE 开发者门户](https://developer.adobe.com/after-effects/)。

**官方旧 UI 回退范围**

官方帮助列出 Darkest、Dark、Light 主题及对比度选项。[外观设置](https://helpx.adobe.com/after-effects/desktop/get-started/preferences-and-settings/preferences.html)。**员工声明**：Brian C Carter 于 2024-11-25 说明 theme coloring 从未正式受支持，AE2025 新皮肤不使用它，当时主题颜色选项均位于 Appearance；其资料列明 Adobe 高级质量工程师身份。[声明](https://community.adobe.com/feature-requests-530/can-no-longer-change-theme-ui-color-in-after-affects-1214917/index2.html)、[身份资料](https://community.adobe.com/members/brian-c-carter-1897772)。

本轮未找到 AE25 全局切回旧 UI 的官方入口；这不等于证明技术上绝无可能。**员工声明**：Tim Kurkoski 在 2014 年解释的 `Use Legacy UI` 仅作用于单个 ScriptUI 面板，不能作为 AE2025 全局回退依据。[历史原文](https://community.adobe.com/questions-529/2014-1-appearance-of-scripts-is-incorrect-30208)。官方另提供 Creative Cloud 的 `Other versions` 安装入口，但限列表中可用版本；这是更换应用版本，不能视为 AE25 内部切换 UI。[官方安装说明](https://helpx.adobe.com/download-install/apps/download-install-apps/creative-cloud-apps/install-previous-versions-creative-cloud-apps.html)。

**未证实及检索边界**

在上述发布、帮助、开发者材料及 Adobe 官方技术源码检索范围内，未找到足够版本对照，证明 AE25 引入、移除或替换了 dvaui、Drawbot 或 Skia。本轮也未取得可核对版本的官方 Drawbot 头文件／API 原文；不使用 docsforadobe 社区镜像、身份不明的论坛回答、用户崩溃日志或同名项目补足证据。不能宣称“迁移到 Skia”“全面改用 DirectWrite”或“停用 GDI/GDI+”，也不能将界面改版直接认定为文字捕获失败原因。未找到证据不代表这些组件不存在。

**Windows API 边界：不证明 AE2025 已采用**

- DirectWrite 排版与绘制解耦：自定义 `IDWriteTextRenderer::DrawGlyphRun` 可转交 Direct2D，也可通过 `IDWriteBitmapRenderTarget` 绘到 GDI 位图。因此这些 API 可以共存；模块加载不能定位某块界面的文字入口。[架构说明](https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-and-directwrite)、[GDI 绘制路径](https://learn.microsoft.com/en-us/windows/win32/directwrite/render-to-a-gdi-surface)。
- GDI+ 的 `Graphics::DrawString`／`DrawDriverString` 分别对应 `GdipDrawString`／`GdipDrawDriverString`。[官方映射](https://learn.microsoft.com/en-us/windows/win32/gdiplus/-gdiplus-text-flat)。后者的 UINT16 数据在设置 `DriverStringOptionsCmapLookup` 时为 Unicode，否则为 glyph index；GDI `ExtTextOutW` 也有 `ETO_GLYPH_INDEX`。[DrawDriverString](https://learn.microsoft.com/en-us/windows/win32/api/gdiplusgraphics/nf-gdiplusgraphics-graphics-drawdriverstring)、[ExtTextOutW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-exttextoutw)。
- glyph 与字符可能多对多，不能将 glyph ID 直接解码为文本。[字符与字形](https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-and-directwrite)。若目标实际使用文本布局回调，可检查 `DWRITE_GLYPH_RUN_DESCRIPTION` 的 `string`、`stringLength`、`clusterMap`、`textPosition`；仍须验证目标是否提供这些数据。[结构定义](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ns-dwrite-dwrite_glyph_run_description)、[回调契约](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextrenderer-drawglyphrun)。

**公开资料不能代替的实机检查**

以下是依据上述 API 边界提出的验证建议，不是已确认的 AE 实现：使用合成文字，固定版本、语言、缩放和重绘操作，按主面板、菜单、插件面板、提示分别追踪实际调用及数据语义，区分字符、字形、轮廓和位图；不要从一块 UI 或旧版本的覆盖外推整个 AE2025。[DirectWrite 分层及轮廓／位图输出](https://learn.microsoft.com/en-us/windows/win32/directwrite/introducing-directwrite)。

主题可作为对照变量；硬件加速对照须先确认该 AE 版本实际提供的选项及其作用范围，不能套用前述 Premiere 回退建议，也不能预设它们会恢复旧文字入口。补齐调用链和捕获结果的对应关系后再判断因果。[AE 外观设置](https://helpx.adobe.com/after-effects/desktop/get-started/preferences-and-settings/preferences.html)、[员工对 AE／Premiere 的不同说明](https://community.adobe.com/bug-reports-728/p-hundreds-of-direct2d-drawbot-error-hresult-2005270494-1331150/index3.html)。
