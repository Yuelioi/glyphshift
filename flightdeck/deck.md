# Flightdeck

## Open Work

- [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — 以“捕获原文、命中字典、
  在真实软件中立即写回译文”为主线；Qt 5/6 Widgets 绘制链已通过合成与动态链接真实目标验收并
  进入正式 Bundle；WinUI/MRT 已验证为加载时资源替换而非完整实时写回。GTK 3 / Pango 的公开
  `gtk_render_layout` Layout 副本方案也已通过像素、热更新、恢复与正式注入合同并进入 Bundle；GTK 4
  因标准控件只经过私有 snapshot 入口而暂不建设。动态 Tk 8.6 与 GDI 的隐藏双路对照也已证明结果
  等价并作 No-Go。SDL2_ttf 因缓存 Surface 无法满足实时恢复合同；SDL3_ttf 虽有绘制时 Text API，
  但等待真实授权目标后再进入 Roadmap 原型，不提前制造空置 Adapter。
- **Focus:** [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — Scintilla DirectWrite 编辑区
  底层已完成真实 Go，并覆盖 Compatible Bitmap 缓冲绘制；开发 App 也已用含 DirectWrite 的新 Bundle
  重启。同一目标进程中的候选 Hook 现已支持部分成功，不适用技术不会再撤销健康写回技术，且
  Controller 只回传真实激活集合。当前只差在既有 Probe 中显式加入 DirectWrite 并完成肉眼端到端
  验收，不能再用 UIA 观察冒充写回。WPF 仍只有 UIA 结构化观察收益，后续由外部译文呈现 Roadmap
  承接。
- [运行时能力升级 Roadmap](work/runtime-capability-roadmap/index.md) — Observation Stream、共享 Hook、区域
  Binding 和 Web 宿主协作等架构升级继续保留，但不再抢占当前通用写回主线。

## Project links

- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)
