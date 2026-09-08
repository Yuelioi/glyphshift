# 适配器与通用文字完整性

Status: Open

## Goal

保持一个 x64 App 和可按架构加载的逻辑 Adapter，交付跨软件复用的文字完整性与替换资格机制。框架差异留在 Adapter，不按游戏名、样本地址或专用词句实现补丁。

## Current

用户确认 CatSystem2 新版可用，仍保留实验标识。Houdini 及其他软件的新增覆盖调查已交接至 [下一项 Work](../software-text-coverage/index.md)，本轮不继续实机调查。探针列表停止按钮在探针 Work 中完成，相关回归通过。

用户授权实现首项及审计全体现有适配器。已补 [CatSystem2 经典分支](slices/catsystem2-classic.md)：原生 DLL 在授权目标的当前句完成英文、中文次代替换与停止恢复，合成 9 个变体及 5 项单元通过；继续实验，App 端到端与经典历史页待验收。[13 项字符串分支矩阵](references/string-branch-coverage.md) 已整理具体入口与缺口。最新 App 已经指定脚本同步构建并启动，生产加载器 13 项、catalog Playwright 1 项及 Release 四种布局合同通过。四个目标的路线见 [初查](slices/four-target-survey.md)。以下为既有验收记录。

**用户复测发现历史页仍不刷新，用户已授权提交。** 已确认游戏加载了最新工件。历史页无标记正文直接提交显示，绕过普通正文对象回调；上轮仅修复带标记当前句不覆盖此路径。现补充显示入口捕获与历史容器回调，复用对象归属、线程、换代和恢复检查。连接时已打开的历史页仍需重开以观察显示入口。见 [CatSystem2 原生 Slice](slices/catsystem2-native.md)。

历史行局部实验已显示完整英文替换并恢复；未替换与替换模式的文字区域检查分别为红/绿。旧 DLL 在历史页独立显示缓存合同失败，新 DLL 在两种布局下通过首代、次代、同译文重试、释放恢复和 Host 重入；3 项单元、三种错误形态拒绝与严格 x86 Clippy 通过。生产 App 端到端待用户复测，中文缺字未验收。最新 Release App 已经指定脚本同步构建并启动，生产加载器验证 13 个适配器通过。

刷新常驻成功提示已移除，说明只放在按钮 hover title，保留请求失败提示。相关 Playwright 24 项通过，已核对无常驻提示的画面。该适配器与刷新改动随后已提交 `9f670c0`。

**VGUI 保持“可用、未完整验收”。** 标准 Label 缓存刷新和恢复已验证，用户后续准备 Garry’s Mod 扩展覆盖；不推广为全部 Source 游戏支持。既有适配器提交为 `7c7b986`。随后 CatSystem2 已加入默认 Bundle/Catalog，现为 13 个逻辑适配器、12 个 x86 工件。此前活动回归 511 通过、25 忽略，探针后端 16 项、相关 Playwright 23 项通过；这些为此前结果，不替代此次刷新修复后的真机验收。

## Next

新增软件与字符串分支工作转至 [Houdini 与其他软件文字覆盖](../software-text-coverage/index.md)。本 Work 保留既有适配器验收记录；CatSystem2 经典历史页与 VGUI 跨游戏覆盖仍待独立验收，不扩大当前结论。

VGUI 保持“可用、未完整验收”，不扩大到全部 Source 游戏。Garry’s Mod 的 VGUI 机制已由 [Facepunch 文档](https://wiki.facepunch.com/gmod/vgui) 确认，实际分支、位数及自定义控件仍需验证。原有方案与边界见 [VGUI Slice](slices/vgui-localize-native.md)。

## References

- [活动适配器扩展矩阵与验收](references/expanded-adapters.md)
- [双架构实现与验收边界](references/dual-architecture-implementation.md)
- [研究范围](context.md)
- [字体与适配器审计](references/adapter-audit.md)
- [x86 可行性与阶段](references/x86-feasibility.md)
- [成熟项目对比与方案权衡](references/architecture-comparison.md)
- [Windhawk 与 OBS 源码证据](references/comparison-windhawk-obs.md)
- [Frida 与 Detours 源码证据](references/comparison-frida-detours.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
