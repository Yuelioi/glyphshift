# 适配器与通用文字完整性

Status: Open

## Goal

保持一个 x64 App 和可按架构加载的逻辑 Adapter，交付跨软件复用的文字完整性与替换资格机制。框架差异留在 Adapter，不按游戏名、样本地址或专用词句实现补丁。

## Current

**用户确认 VGUI 已可用，但明确尚未完成整体验收。** 保留实验标识与覆盖限制；后续由用户准备 Garry’s Mod 扩展验证。本次先提交当前适配器工作，再调查已启动的 NEKOPARA Vol. 3 当前对话捕获，菜单已由用户确认可用。

**已实现标准 Label 自动刷新，并完成原生 DLL 与生产 Runtime 的同菜单更新/停用验证。** `request_refresh` 推进代次，在目标 UI 回调使用目标自身 setter 更新已验证标签；寿命句柄、原本地化索引及最后应用值保护对象归属。隐藏对象也在停用时恢复，无 UI 回调则超时失败，不误报恢复。描述符改为 `RetainedObject`；未知形态保留查询回退。见 [实现说明](../../../crates/adapters/implementations/framework/vgui-localize-native/README.md)。

用户“配置译文后重启才生效”的判断已证实为本轮标准标签缓存问题。一个既存主菜单按钮先后显示两版英文，再恢复中文，全程未重启或重开菜单。原生 ABI 实验四帧像素掩码通过，原始/恢复交并比为 1.0。生产 Desktop Backend / Runtime 字典发布合同两次通过，两版与恢复画面人工核对；部分生产截图被其他窗口遮挡，不能把对应自动像素检查报为通过。实验数据全部留在本地。

新增 6 项 x86 识别/生命周期测试通过，查询层 7 个进程合同通过，严格 Clippy 通过。生命周期覆盖旧代次、外部改字、句柄复用、隐藏恢复、停用超时；补充长输出/错误新译文退回原文，避免继续显示旧译文。刷新主路径经同步 Release Bundle 验证，最后的输出回退修正经定向测试，已通过指定脚本同步重建并启动最新 Release App，生产加载器验证 12 个适配器通过，窗口响应与新刷新说明已核对。当前游戏已停止实验替换，保持原文；首次切换新工件需要正常重启已加载旧工件的目标；后续字典换代无需重启已支持的控件。

**用户已确认单字来自其他适配器，本轮停止处理单字，目标改为尽量扩大实际翻译覆盖。** 最新探针含 318 条 VGUI 完整文本及 6 条单字；用户已看到部分中文转英文并反馈释放连接可恢复，但部分已配译文的完整标签仍不变。此前 3 条完整文本的快照仅代表更早采样，不再作为当前采集范围。当前需要逐词区分未命中、Adapter 回退、控件旧副本及绕过查询的写入路径。

**《求生之路》的 VGUI 查询原生实现已通过单个设置标签的可见替换、字典更新和停用重建恢复。** 新包为 `windows.vgui.localize-query`，本轮使用生产 Desktop Backend / Runtime / Controller 发布链路，没有使用 Frida 驱动替换；用户要求直接在 App 测试，现已将查询 Adapter 加入默认 Bundle/Catalog，标为实验且限定可重建短标签；不能推导整个游戏已支持。

已提交的双架构基础为 `29c74ac`，活动适配器扩展为 `ea56733`，App 仍为 0.3.0。默认 Bundle 配置现为 12 个逻辑适配器、11 个 x86 工件。共用文字完整性及 Qt/DirectWrite 接入纳入本次提交；此前 VGUI 逐字重放实验冻结。

原生 Adapter 从公开工厂取得候选接口，通过函数数据流识别配对的查询路径；未固定样本槽位、对象偏移或代码前缀。完整原文复用 V1 Host/Core 字典与诊断流程，未修改公共 ABI。返回池最多 1 MiB / 4096 项，按原文/译文复用，旧指针跨换代及启停保留；没有可证明的控件刷新入口时仍要求正常重建。

`scripts/test-vgui-localize.ps1` 的 7 个独立进程合同通过，新包 x86 Clippy 严格检查通过；`scripts/test.ps1` 的活动包回归为 505 项通过、0 失败、25 忽略。同 checkout 重新构建的本地实验 Bundle 经生产加载器验证，真实目标中加载的 Runtime / Adapter 与该 Bundle 匹配。

实机在新目标实例中完成两个字典代次，各有 2 次替换命中；首代、次代、恢复三张图人工核对，固定文字区域的像素掩码断言通过，原始/恢复相似度约 0.996。生产 Runtime 合同正常完成，设置页已取消并回到主菜单，游戏保持响应。模块及已返回存储按设计留驻到进程退出，替换已停用。

恢复时已重跑生命周期模型，x86/x64 各 6 项通过。之前的实验工具、可见结果及共用文字完整性改动仍保留，本次提交前重跑活动包入口：509 项通过、0 失败、25 忽略；6 项 x86 VGUI 单元合同和 7 个查询进程合同此前定向通过。

## Next

当前转到 NEKOPARA Vol. 3 的对话捕获：以当前画面的完整对话为断言目标，区分系统菜单文字与游戏正文入口。先检查已运行探针的实际观测，再定位引擎文字所有者；不以截图 OCR 或菜单条目冒充正文捕获。机器输入和捕获原文仅本地保存。

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
