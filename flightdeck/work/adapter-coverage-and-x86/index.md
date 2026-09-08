# 适配器与通用文字完整性

Status: Open

## Goal

保持一个 x64 App 和可按架构加载的逻辑 Adapter，交付跨软件复用的文字完整性与替换资格机制。框架差异留在 Adapter，不按游戏名、样本地址或专用词句实现补丁。

## Current

**用户复测发现历史页仍不刷新，用户已授权提交。** 已确认游戏加载了最新工件。历史页无标记正文直接提交显示，绕过普通正文对象回调；上轮仅修复带标记当前句不覆盖此路径。现补充显示入口捕获与历史容器回调，复用对象归属、线程、换代和恢复检查。连接时已打开的历史页仍需重开以观察显示入口。见 [CatSystem2 原生 Slice](slices/catsystem2-native.md)。

历史行局部实验已显示完整英文替换并恢复；未替换与替换模式的文字区域检查分别为红/绿。旧 DLL 在历史页独立显示缓存合同失败，新 DLL 在两种布局下通过首代、次代、同译文重试、释放恢复和 Host 重入；3 项单元、三种错误形态拒绝与严格 x86 Clippy 通过。生产 App 端到端待用户复测，中文缺字未验收。最新 Release App 已经指定脚本同步构建并启动，生产加载器验证 13 个适配器通过。

刷新常驻成功提示已移除，说明只放在按钮 hover title，保留请求失败提示。相关 Playwright 24 项通过，已核对无常驻提示的画面。未创建 Git 提交。

**VGUI 保持“可用、未完整验收”。** 标准 Label 缓存刷新和恢复已验证，用户后续准备 Garry’s Mod 扩展覆盖；不推广为全部 Source 游戏支持。既有适配器提交为 `7c7b986`。随后 CatSystem2 已加入默认 Bundle/Catalog，现为 13 个逻辑适配器、12 个 x86 工件。此前活动回归 511 通过、25 忽略，探针后端 16 项、相关 Playwright 23 项通过；这些为此前结果，不替代此次刷新修复后的真机验收。

## Next

同步构建与界面验证已完成。用户正常重启游戏后先连接再打开历史页，检查首代、次代、重复刷新和释放恢复；按 [CatSystem2 原生 Slice](slices/catsystem2-native.md) 收口。适配器继续标记实验，等待另 2～3 个游戏验证后再考虑正式验收，不把局部实验扩大成完整验收。

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
