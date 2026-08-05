# Flightdeck

## Open Work

- [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — 以“捕获原文、命中字典、
  在真实软件中立即写回译文”为主线；Qt 5/6 Widgets 绘制链已通过合成与动态链接真实目标验收并
  进入正式 Bundle；WinUI/MRT 已验证为加载时资源替换而非完整实时写回。GTK 3 / Pango 的公开
  `gtk_render_layout` Layout 副本方案也已通过像素、热更新、恢复与正式注入合同并进入 Bundle；GTK 4
  因标准控件只经过私有 snapshot 入口而暂不建设。动态 Tk 8.6 与 GDI 的隐藏双路对照也已证明结果
  等价并作 No-Go。SDL2_ttf 因缓存 Surface 无法满足实时恢复合同；SDL3_ttf 虽有绘制时 Text API，
  但等待真实授权目标后再进入 Roadmap 原型，不提前制造空置 Adapter。
- **Focus:** [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — Web/CDP 已确认不能连接绝大多数
  未主动开放宿主接口的现有 Web 桌面软件，降级为后续 Host-assisted Integration，不计通用实时翻译
  覆盖。首个授权 WPF 真实目标已证明 UIA 可观察 72 条公开文本，但六条 Native 写回入口全部零命中；
  当前需要在 Managed WPF Agent 与 UIA 驱动的外部翻译呈现之间选择下一 Apply Model，不按模块名称
  制造空置 Adapter。
- [运行时能力升级 Roadmap](work/runtime-capability-roadmap/index.md) — Observation Stream、共享 Hook、区域
  Binding 和 Web 宿主协作等架构升级继续保留，但不再抢占当前通用写回主线。

## Project links

- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)
