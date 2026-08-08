# Flightdeck

## Open Work

- **Focus:** [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — 正式 Adapter 已补齐
  可验证的官方技术文档入口；Unity / UE 可实施性已完成分层复核，不从引擎名承诺覆盖。生产实现前
  必须先通过每个分层两个外部 Shipping 真实目标加一个负例的 gate；两份 Unity Mono 候选已完成
  x64 盘点与无注入启动；Unity 2021 LTS / Unity 6 的两份 IL2CPP 正式包也已通过 x64、metadata、导出
  API、UI 标记、符号和保护组件的静态门禁。首个 Unity 6 候选因 Development Build 被否决后，备用
  Unity 6 release 已通过静态盘点与两次无注入启动；IL2CPP 现有跨 Unity 2021 LTS / Unity 6 的两个
  Shipping-like 运行候选和可重复动态输入、运行时格式化源码合同。另有一份 Mono NGUI Mesh/atlas
  负例可补 Standard UI 通用拒绝证据，但不能代替 IL2CPP 同后端负例；一次有界公开候选复核没有找到
  合格 IL2CPP 自绘文字成品，attach gate 因此仍未通过。Unity Mono Standard UI Observe/TextReplace 已在
  单一 Adapter 内完成并进入生产 Catalog/Runtime Bundle：合成宿主、两个跨代 Standard UI 正例和 NGUI
  同后端负例均通过，首代、第二代与停用恢复有可见证据；Debug/Release Bundle、中文技术边界与 Unity
  官方文档入口也已验证。下一覆盖项继续等待新的授权真实缺口；仍不扩大到 IL2CPP、UI Toolkit、NGUI
  或 TMP `SetText(...)`，IL2CPP attach 仍缺同后端自绘文字负例。
- [运行时能力升级 Roadmap](work/runtime-capability-roadmap/index.md) — Runtime 兼容性路由与
  一键快速探针、Desktop Acquisition Command 与 UIA 取词翻译产品入口已完成。生产 OCR 的技术评审
  、候选 Worker、Region 调度及 UIA 无文本后的显式 OCR 产品入口已完成。精简 Tesseract 支持集、
  完整许可/notice、体积、延迟与峰值内存门禁也已通过；负坐标、受保护窗口及退出边界已由真实 WGC
  Fixture 固定，两个独立授权目标的同点 UIA/OCR 缺口证据也已完成。默认开发/Release Bundle 与
  光标附近悬浮翻译气泡现已交付；软件快速捕获与取词翻译快捷键设置也已实现真实按键录制、占用/内部
  冲突探测和原子重绑。本轮按用户要求未运行自动化验证；当前剩余提交/发布前定向门禁与物理多显示器/
  混合缩放 smoke。OCR 与取词翻译不再抢占近期产品工作，只作为最后阶段的可选兜底继续保留。
- [探针创建来源统一](work/probe-creation-sources/index.md) — 已完成：Probe 创建统一 Software/Dictionary
  来源，临时资产可保留或清理，Probe/Dictionary 联动与元数据表单已补齐，Desktop API 已推进到
  `/26`。Rust 定向门禁与前端生产构建通过；Playwright CLI 单文件启动超时作为测试基础设施风险留存。
- [全仓代码结构梳理与优化](work/repository-structure-optimization/index.md) — Adapter 实现二级目录纠正已
  完成：23 个 crate 归入 `native/framework/accessibility/fallback`；分组只负责导航，没有新增运行时
  seam，也没有创建空的未来类别。
  Observation Stream、共享 Hook、区域 Binding 和 Web 宿主协作继续不抢占当前路径。

## Project links

- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)

## 验证纪律

- 开发循环只运行受影响 package、合同与页面的定向测试，并在约 20 秒没有新输出时主动回报状态。
- 大 Slice 自审运行相关边界回归，不默认升级为全仓测试。
- 全仓测试只用于提交前、发布前，或 workspace 拓扑、公共 ABI / wire contract 发生实质变化时；运行前
  明确说明必要性。
