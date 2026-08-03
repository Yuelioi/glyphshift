# 运行时能力升级 Roadmap

Status: Open

## Goal

在不扩大 Dictionary 职责、不泄漏平台细节到核心匹配 Interface 的前提下，按真实软件证据逐步扩展
多进程接管、观察流复用、Windows 文字技术覆盖、原生隔离和布局适配能力。

## Current

外部评审和通用升级方案中可用的未来方向已经被重写为 GlyphShift 的 Capability Adapter 路线。
Roadmap 不承诺一次支持 DirectWrite、UIA、OCR、Direct2D、Direct3D、OpenGL 和 Vulkan；每项能力
必须先证明目标软件存在稳定信号、可验证写回或明确的 observe-only 价值。

当前所有目标进程仍遵守一个 Software 同时由一个 enabled Workflow 拥有、同一软件同一时刻只有一个
Probe Run 占用 Target Runtime 的规则。未来希望把这一限制深化为单一 Target Execution：最多一个
Publication Owner，允许多个只读 Observation Subscriber，共享同一套注入和 Hook。

LunaTranslator 固定源码版本调研进一步确认：Adapter 不能代替“选择哪一条候选文本流”。Roadmap
因此把 Observation Stream Identity / Binding 与 Process Family 并列为高优先级，并新增有界
Transform Profile；OCR、剪贴板、Overlay 和外部事件 API 保持较低优先级、按真实缺口立项。

## Next

- 本主题被提升为 Focus 后，先以合成 Controller inventory 冻结 Process Family 的授权、发现、退出
  和部分失败合同；从[稳定约束](context.md)、[阶段计划](plan.md)和
  [Controller Windows Module](../../../crates/glyphshift-controller-windows/src/lib.rs)开始。

## Progress

- 已完成 AI 评审路由，建立采纳门槛和明确的非目标；尚未授权实现 Roadmap 能力。
- 已完成 LunaTranslator 产品与运行时一手资料调研，冻结 Observation Stream、Transform Profile
  及可选输入/输出能力的路由边界。

## References

- [稳定约束](context.md)
- [阶段计划](plan.md)
- [AI 评审路由结论](references/ai-review-assessment.md)
- [产品领域语言](../../../CONTEXT.md)
- [Capability Adapter 调研](../glyphshift/references/adapter-registry-and-cross-platform-interception-research.md)
- [LunaTranslator 产品与运行时调研](../glyphshift/references/lunatranslator-product-runtime-research.md)
