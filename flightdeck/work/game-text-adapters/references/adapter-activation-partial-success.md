# 加勾 Windows 适配器后启动错误消失

Status: Investigating

用户反馈：重启应用和目标软件仍出现“没有成功启用任何兼容方式”，加勾任意 Windows 适配器后错误消失。用户补充原先勾选全部且能采集单字；关闭所有 Windows 适配器并重启后失败。用户另有同时启用后关闭 Windows 能改善的经历。仍未定位现场根因，不能排除适配器之间的相互影响。

运行时对照已验证：所有部署候选激活失败会返回 AdapterActivation；只要至少一个候选激活成功，整体启动允许成功，激活报告只包含成功候选。错误消失不证明原先候选生效，也不证明目标文字路径经过新增候选。

复跑 `glyphshift-target-runtime` 的 `trh_003_` 与 `trh_005_` Native 合同各 1 项通过（明确使用 `--ignored --test-threads=1`，仅活动 GDI/Qt Painter DLL）。未运行归档 UIA，未启动用户目标或重新打包 App。此为机制对照，不是现场复现。

错误映射还把 ProtocolRejected、RuntimeExportUnavailable 与 TargetRuntimeRejected 汇总为 component_incompatible，现有重启优先文案不能定位具体原因。不能仅凭截图断言 Qt 版本问题，也不能把加勾 Windows 当成通用修复。

后续先完成了[多适配器协同与停止修复](../slices/adapter-cooperation-stop.md)。现场下一步需要单字条目的来源适配器，以及剩余 Qt 适配器的具体激活失败原因；不能把已修复的停止短路当成现场 Qt 初始化失败的根因。
