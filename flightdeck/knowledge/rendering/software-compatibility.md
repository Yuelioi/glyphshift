# 软件兼容状态

用于快速判断已经调查或实机展示过的软件当前能否使用，以及下一步应复用、扩展哪一条通用适配路线；当前找不到可复用通用入口的软件保持“尚不支持”，暂不建立软件专属适配器。
这里只记录可移植结论；本机截图、路径和原始日志仍留在 `local-test/`，不进入项目知识。

| 软件 | 当前状态 | 推荐路线 | 简介 |
| --- | --- | --- | --- |
| Cavalry | **已验证可用（实机截图）** | 继续使用现有 Qt Widgets 路线 | 实机展示中主界面已有多处中文，Glyphshift 能持续采集并替换 Qt Widgets 文字。当前证据证明展示区域可用，不等于每个面板都已完整验收。 |
| Fiddler Classic | **已验证可用（实机截图）** | 继续使用现有 Windows 文字适配器 | 实机展示中菜单和界面文字已有中文，且能持续采集大量词条。当前结论以展示区域为边界。 |
| [Houdini 22.0.429](../../work/software-text-coverage/references/software/houdini.md) | **部分支持，建议在首次生成 UI / help 前启用 Workflow** | **Qt Painter + Qt Text Document + SideFX CV PaintBuffer** | shelf 与 Network Editor 菜单走 Qt Painter；`Add / Edit / Go / View / Tools / Layout / Help` 的首尾布局空白已由通用 Runtime source-policy key 回退解决，真实决策与中文像素均通过。hover / help 完整正文进入 `QTextDocument::setHtml`，Qt 6.8.3 x64 的纯文本与单可见文本节点 HTML 首代可见替换通过。Network Editor 的 `Non-Commercial Edition`、`Objects` 与 `Empty Network\nPress Tab to Add Nodes` 已确认进入动态 `libCV.dll!CV_PaintBuffer::textWrappedInBox`，动态实验中文像素和合成 ABI 生命周期合同通过；production Adapter 已进入 16-Adapter Bundle，重启目标后的当前界面已获用户手动确认完全汉化；第二代更新 / 停用恢复仍缺独立实机验收，保留缓存生命周期边界。 |
| IDA | **已验证可用（实机截图）** | 继续使用现有通用适配器并按版本观察 | 实机展示中菜单和主要界面已有中文。当前只记录已展示版本与区域可用，不扩大为所有 IDA 版本完整支持。 |
| NEKOPARA | **已验证可用（实机截图）** | 继续使用已验证的游戏文字适配器 | 实机展示中游戏正文已出现中文，可作为游戏正文路线的成功样本。不同作品、引擎版本和历史页仍按各自适配器边界判断。 |
| Notepad++ | **已验证可用（实机截图）** | 继续使用现有 Windows 文字适配器 | 实机展示中菜单和编辑器外围界面已有中文，适合作为传统桌面软件通用路线的成功样本。 |
| RizomUV | **已验证可用（实机截图）** | 继续使用现有通用适配器 | 实机展示中主界面已有中文，可继续按现有通用路线使用；未展示区域不自动视为完整覆盖。 |
| SpeedTree | **已验证可用（实机截图）** | 继续使用现有通用适配器 | 实机展示中主界面已有中文，可继续按现有通用路线使用；复杂面板和特殊文字区域仍以实际采集结果为准。 |
| [Autograph 2026](../../work/software-text-coverage/references/software/autograph.md) | **已验证可用（实机 Runtime）** | **复用 Qt Quick 标准文字适配器** | 已精确加入 Qt 6.11.1 MSVC x64 ABI。真实生产 Runtime 捕获 47 条唯一文字且 0 丢弃，全部来自 Qt Quick；`File` 已完成两代热更新并在停止后恢复原文。当前结论限于 QQuickText 标准标签，输入框、富文本和自绘文字仍按适配器边界排除。 |
| [Silhouette 2026](../../work/software-text-coverage/references/software/silhouette.md) | **部分支持，节点图待复验** | **继续扩展通用 Qt Painter 适配器** | Qt 6.5.4 `isl0` 主界面标准区域已通过生产 Runtime，但节点图仍漏 `Unproject`、`Right`、`Left`、`Time Shift` 与节点 `Output`。已确认主程序直接导入两个此前未 Hook 的标准 QPainter 重载，并已把它们作为通用可选能力补入；等待目标重启后完成实机验收。 |
| [ZBrush 2026](../../work/software-text-coverage/references/software/zbrush.md) | **尚不支持** | **继续寻找可抽象的通用文字入口** | 界面文字已确认绕过现有 13 个通用文字适配器；常规重绘复用 glyph / 纹理缓存。当前规则禁止软件专属适配器，只有找到可复用的完整字符串上游 seam 后才进入通用适配器实现。 |
| [Blender](../../work/software-text-coverage/references/software/blender.md) | **尚未适配** | 优先官方 Python 翻译接口 | 自绘界面。官方翻译注册、注销和区域重绘接口比底层像素 Hook 更适合作为入口，但尚未完成完整覆盖验证。 |
| [TouchDesigner](../../work/software-text-coverage/references/software/touchdesigner.md) | **尚不支持** | 推荐 Slug / 内部文字布局适配器 | 新版主界面大量文字使用 Slug / Text COMP 路线，现有 Windows / Qt 适配器没有覆盖。公开导出入口未找到，需继续定位稳定的完整字符串入口。 |
| SlugD3D11 Demo | **尚不支持** | 继续研究可复用的 Slug GPU 文字入口 | Slug 7.5 D3D11 Demo 的文字由 Slug 自有 shader / glyph mesh 管线绘制，二进制内直接携带 Slug 文本与 shader 资源；现有 GDI、DirectWrite、Qt 与 Unity 适配器均不经过该文字路径。后续只有找到可复用的完整字符串上游 seam 才进入通用适配器实现。 |
| [Vovious](../../work/software-text-coverage/references/software/vovious.md) | **尚未适配** | 推荐 JUCE 通用适配器 | 静态 JUCE，自带 TextEditor / Typeface 类型，但没有可用文字函数导出。需要找到接收完整 JUCE String 的绘制或布局入口。 |
| [Fluffy Mod Manager](../../work/software-text-coverage/references/software/fluffy-mod-manager.md) | **尚不支持** | 暂后置；寻找完整字符串入口 | OpenGL 自绘，当前没有找到可复用的文字函数入口。只拿到字形贴图或顶点不足以做可靠翻译。 |
| [PotPlayer](../../work/software-text-coverage/references/software/potplayer.md) | **可用但有使用限制** | 继续使用现有 GDI 路线 | 外挂 SRT 完整正文可命中 TextOutW / ExtTextOutW；预先准备字典并重新加载字幕可显示译文。实时缓存刷新未适配。 |
| [MTool](../../work/software-text-coverage/references/software/mtool.md) | **研究参考，不计入 Glyphshift 支持** | 转游戏引擎专属路线 | 用于参考按游戏引擎获取完整对白和菜单的产品方向。MTool 官方支持列表不能直接算作 Glyphshift 能力。 |

## 状态含义

- **已验证可用（实机截图）**：真实软件已经出现可见汉化效果；只证明截图展示的版本和区域，不自动代表全软件、全窗口或全部刷新/恢复行为均已验收。
- **已验证可用（实机 Runtime）**：真实软件已通过生产 Runtime 的捕获、替换、更新和停止恢复闭环；仍只覆盖记录中明确验收的版本、ABI 和文字类型。
- **部分支持**：现有适配器能处理一部分文字，同时存在已确认的漏采区域。
- **可用但有使用限制**：已有可重复的使用方式，但实时刷新、恢复或覆盖范围仍有限制。
- **尚未适配**：方向已找到，但还没有完成生产适配与实机验收。
- **尚不支持**：已有明确证据表明现有生产适配器无法覆盖主要文字路径。

新增软件调查时，先更新本页的一行状态；需要保留技术证据时，再在对应 Work 下建立软件独立记录。
