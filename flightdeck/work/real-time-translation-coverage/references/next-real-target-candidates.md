# 下一轮真实目标候选筛选

日期：2026-08-05

## 结论

本轮推荐的 **Notepad++ Scintilla 编辑区**已完成实机验收，不把菜单、
对话框等既有 Win32/GDI 覆盖算作新能力。官方源码显示 Notepad++ 会按配置向 Scintilla 发送
`SCI_SETTECHNOLOGY`；Scintilla 的 Windows Direct2D Surface 明确创建 DirectWrite `TextLayout`，再调用
`ID2D1RenderTarget::DrawTextLayout`。这是一条可审计、可重复触发、不同于现有 GDI/DrawText/GDI+/Qt
Widgets/GTK3-Pango 的真实文字路径，不是根据“使用某框架”猜测命中。

VS Code 与 Godot Editor 保留为边界对照：前者代表 Electron/Chromium DOM 渲染，后者代表引擎自有
TextServer/Canvas 文字栈。它们都有真实产品价值，但当前都没有像 Scintilla DirectWrite 那样窄、稳定、
公开且适合立即验证的系统绘制入口，因此本轮不并行实现 Adapter。

| 候选 | 代表路径 | 与现有 Adapter 的关系 | 下一步判断 |
| --- | --- | --- | --- |
| **Notepad++（已完成）** | Scintilla → DirectWrite TextLayout → Direct2D DrawTextLayout | 编辑区不走现有 Qt/GTK/GDI 文字入口；菜单可能仍被 GDI 命中，必须排除 | Go：进入正式 Bundle |
| VS Code | Electron renderer → HTML/CSS/DOM → Chromium 渲染进程 | Native Hook 最多偶然看到下游字形/系统壳，不等于拿到 DOM 原文 | 保留为 Web/结构化观察边界，不立 Native Adapter |
| Godot Editor | Godot Label/TextLine → TextServer shaped text → Canvas/RenderingServer | 引擎自有文字对象与 GPU Canvas，不等同 Qt、GTK 或 Win32 文本 API | 保留为 Engine-aware Roadmap 候选 |

> 共同约束：下面的源码证据只能证明软件拥有该文字路径，不能证明当前发布二进制在指定场景一定命中。
> 支持结论仍必须由运行模块、真实调用事件、可见替换、热更新与停用恢复共同证明。

## 1. Notepad++（唯一首选）

### 验收结果

结果为 Go。Scintilla 自身报告 DirectWrite 技术模式；真实编辑区取得完整 source，首版 Dictionary、
第二代热更新和停用恢复均通过。正式 Adapter 同时覆盖直接 Render Target 与缓冲绘制使用的 Compatible
Bitmap Render Target，只重建单一格式布局；局部格式与内联对象安全放行。软件通过包管理器稳定目录
链接启动时暴露的路径身份差异也已修复并复测。

### 官方维护与获取

- 官方仓库持续发布版本，仓库首页将其定义为 Windows 上的开源文本编辑器；最新发布页同时提供安装包、
  x64 Portable ZIP/7z、SHA-256 摘要与签名材料，适合不改系统安装状态的短期验收。
  [官方仓库](https://github.com/notepad-plus-plus/notepad-plus-plus)、
  [官方发布页](https://github.com/notepad-plus-plus/notepad-plus-plus/releases/latest)
- 项目为 GPL 授权，但开源授权不等于 Glyphshift 可以静默下载、安装或修改用户的软件；实机验收仍需
  用户明确选择官方包并授权目标进程。

### UI / 渲染技术证据

Notepad++ 的编辑器视图是 Scintilla。初始化时，源码在 DirectWrite 可用且用户选择了非默认文字技术时
调用 `SCI_SETTECHNOLOGY`；源码注释还明确指出该选择位于 Preferences 的 Scintilla technology 配置。
[Notepad++ 编辑器初始化源码](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/PowerEditor/src/ScintillaComponent/ScintillaEditView.cpp#L471-L496)

Scintilla 的 Windows Direct2D Surface 并非笼统“可能使用 DirectWrite”：它直接包含 D2D 1.1、D3D11.1
和 DirectWrite 1.1，创建 `IDWriteTextLayout`，并把布局交给
`ID2D1RenderTarget::DrawTextLayout`。该函数接收的仍是转换后的完整 Unicode 文本和布局对象，具备做
Dictionary 精确命中实验的语义位置。
[Scintilla Direct2D Surface 初始化](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/scintilla/win32/SurfaceD2D.cxx#L44-L128)、
[CreateTextLayout 与 DrawTextLayout 路径](https://github.com/notepad-plus-plus/notepad-plus-plus/blob/master/scintilla/win32/SurfaceD2D.cxx#L1205-L1229)

这不是既有 Qt/GDI 验证的重复：

- 菜单、工具栏或传统对话框可能继续经过 Win32/GDI，命中它们只能说明已有 Adapter 工作。
- 本轮只统计编辑区中显式选择 DirectWrite/DirectX 渲染后产生的 `TextLayout` / `DrawTextLayout` 调用。
- 即使进程加载 `dwrite.dll`、`d2d1.dll` 或注入成功，也不能算命中；必须取得编辑区测试文本事件。

### 有界触发场景

1. 使用官方 x64 Portable 包，在普通权限下启动，不安装插件。
2. 在偏好设置中显式选择 DirectWrite/DirectX 文字渲染技术；记录实际选项，不从 DLL 加载反推。
3. 新建未保存的纯文本页，放入 2–3 个互不重叠、无语法高亮的完整英文测试词条。
4. 只观察编辑区：首代 Dictionary 可见替换、同进程第二代热更新、停用后原文恢复。
5. 菜单或对话框命中全部单独记为“已有 GDI 覆盖”，不得计入本次增量。

### 进入实现的门槛

只有同时满足以下条件才恢复 DirectWrite TextLayout Adapter 工作：

- 目标二进制实际使用 DirectWrite 模式，且编辑区持续产生目标 `DrawTextLayout` 事件；
- Hook 取得完整、稳定的 source，而不是拆散字形、不可判定片段或仅测量调用；
- 译文能在当前重绘中可见，第二代 Dictionary 无需重启目标即可生效；
- 释放连接后由目标正常重绘恢复原文；复杂布局或无法安全重建的格式失败开放。

如果只有 GDI 菜单命中、只有观察没有编辑区写回，或切换渲染技术后仍零命中，本轮即作 No-Go，不用
签名扫描或 Notepad++ 专用补丁扩大范围。

### 安全与授权边界

- 只使用官方便携包及其官方摘要/签名，不代用户下载、安装或覆盖现有版本。
- Glyphshift 与目标保持相同普通权限；不得为了方便关闭系统保护或提升到管理员。
- 只打开未保存的合成测试文本，不读取用户文件、会话、插件数据或云内容。
- 授权范围仅限用户选定的本地目标进程和本次测试；不修改目标安装目录，不植入永久插件。

## 2. VS Code（Web 渲染边界对照）

### 官方维护与获取

Microsoft 官方仓库说明 VS Code 按月更新，并把桌面产品定位为 Code - OSS 的发行版；官方 Windows
安装文档提供 User Installer、System Installer 和 ZIP Archive，其中 User Installer 不要求管理员权限，
ZIP 可解压直接运行。
[官方仓库](https://github.com/microsoft/vscode)、
[Windows 官方安装文档](https://code.visualstudio.com/docs/setup/windows)

### UI / 渲染技术证据

官方源码组织文档明确：桌面 Workbench 使用 Electron，浏览器层使用 DOM/Web API，桌面入口包含
Electron main、renderer 与 shared process。Electron 官方进程模型进一步说明，每个 `BrowserWindow`
在独立 renderer 中加载网页，界面按 HTML、CSS 和 JavaScript 的 Web 范式渲染。
[VS Code 源码组织](https://github.com/microsoft/vscode/wiki/source-code-organization)、
[Electron 官方进程模型](https://www.electronjs.org/docs/latest/tutorial/process-model)

因此 VS Code 的设置页、命令面板、侧栏和编辑区首先是 DOM/Chromium 拥有的内容。下游可能加载
DirectWrite、Direct2D 或 GPU 模块，但加载模块、拦到字形绘制或捕获窗口标题都不能证明拿到了 DOM
原文。现有 GDI/DrawText/GDI+/Qt/GTK Adapter 预期只能覆盖少量原生壳或弹窗，而不能代表 Workbench
实时翻译。

### 可见触发场景与当前判断

适合复核的稳定场景是空窗口中的 Command Palette、Settings UI、Explorer 空状态和 About 内容；这些
文本能明确区分 Chromium Workbench 与原生系统对话框。但当前对普通第三方 Electron 实例没有安全、
默认开放的 DOM 接口，之前的 CDP/宿主协作结论仍成立，所以不把它作为下一 Native Adapter。

安全边界是：只用官方 ZIP 或用户级安装、空目录/空配置测试，不加载不可信 Workspace 或扩展；不得为
Glyphshift 关闭 Electron sandbox、添加远程调试端口或暴露 DevTools endpoint。Electron 官方说明 renderer
默认使用 Chromium sandbox，关闭 sandbox 会扩大不可信内容风险。
[Electron sandbox 文档](https://www.electronjs.org/docs/latest/tutorial/sandbox/)

## 3. Godot Editor（引擎自有文字栈边界对照）

### 官方维护与获取

Godot 官方仓库持续维护 4.x，Windows 官方下载页说明标准版“Extract and run”、无需安装且可自包含
运行；官方 Windows 可执行文件有代码签名。项目源码使用 MIT 许可证。
[官方仓库](https://github.com/godotengine/godot)、
[Windows 官方下载页](https://godotengine.org/download/windows/)、
[官方许可证](https://github.com/godotengine/godot/blob/master/LICENSE.txt)

### UI / 渲染技术证据

Godot GUI 不是 Qt/GTK 包装。官方 `Label` 源码把显示文本加入 TextServer 的 shaped-text RID，并在绘制
阶段调用 `font_draw_glyph`；`TextLine` 则把 shaped RID 直接交给 `TextServer::shaped_text_draw`，后者将
文字画进 Canvas Item。官方 TextServer 文档也把 `shaped_text_draw` 定义为“把 shaped text 绘制到
canvas item”。
[Godot Label 源码](https://github.com/godotengine/godot/blob/master/scene/gui/label.cpp#L135-L200)、
[Godot TextLine 绘制源码](https://github.com/godotengine/godot/blob/master/scene/resources/text_line.cpp#L362-L405)、
[TextServer 官方文档](https://docs.godotengine.org/en/stable/classes/class_textserver.html#class-textserver-method-shaped-text-draw)

该路径在文本成形阶段仍有 source，但随后转成 Godot 自有 RID、glyph 与 RenderingServer/Canvas 命令。
因此现有 Windows API Hook 预期只能偶然看到窗口壳，不会自然覆盖编辑器内部的菜单、Inspector、场景树
和属性标签；它代表 Engine-aware 入口，而不是 Qt、GTK 或 GDI 的另一个样本。

### 可见触发场景与当前判断

稳定场景可选 Project Manager 空状态，以及新建空 2D 项目后的 Scene、Inspector、FileSystem、菜单和
设置标签。官方包便携、无账户依赖，适合未来实机复核；但当前源码入口是引擎内部 C++ 对象协议，尚未
证明有稳定导出、跨版本 ABI 或安全停用恢复机制，因此不立即建设 Adapter。

安全边界是：只使用官方签名的标准版，在普通权限下创建空本地项目；不导入第三方项目、不执行项目
脚本、不安装 Editor Plugin，也不把任意 Godot 游戏与编辑器视为同一授权目标。只有先证明公开、稳定的
TextServer/RenderingServer seam，才进入 Engine-aware 原型。

## 下一动作

Scintilla DirectWrite 验收与生产接入已经结束。VS Code 与 Godot 仍不并行下载、不并行探针，也不因
它们使用 Electron 或 TextServer 就预先增加 Catalog 项；下一候选继续从用户真实缺口重新选择。

后续曾以体积小、知名且可由包管理器稳定获取为条件复核 SumatraPDF。官方源码显示主界面文字明确
使用 `DrawTextW` / `DrawTextExW`，部分自绘文字使用 GDI+ `DrawString`，完全落在现有 Adapter 覆盖内，
因此在下载安装前即作重复覆盖 No-Go。小型 Java Swing 候选则普遍以共享 `javaw.exe + JAR` 启动；在
当前“软件绑定可执行文件身份”的模型下无法把不同 JVM 应用安全区分，Java Agent 不能作为当前下一
Adapter 偷跑，需先由后续宿主型应用身份方案承接。
