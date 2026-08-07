# Flightdeck

## Open Work

- [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — 正式 Adapter 已补齐
  可验证的官方技术文档入口；raylib 之后暂无满足门槛的真实动态目标，等待新缺口触发。
- **Focus:** [运行时能力升级 Roadmap](work/runtime-capability-roadmap/index.md) — Runtime 兼容性路由与
  一键快速探针、Desktop Acquisition Command 与 UIA 取词翻译产品入口已完成。生产 OCR 的技术评审
  、候选 Worker、Region 调度及 UIA 无文本后的显式 OCR 产品入口已完成。精简 Tesseract 支持集、
  完整许可/notice、体积、延迟与峰值内存门禁也已通过；负坐标、受保护窗口及退出边界已由真实 WGC
  Fixture 固定，两个独立授权目标的同点 UIA/OCR 缺口证据也已完成。默认开发/Release Bundle 与
  光标附近悬浮翻译气泡现已交付；软件快速捕获与取词翻译快捷键设置也已实现真实按键录制、占用/内部
  冲突探测和原子重绑。本轮按用户要求未运行自动化验证；当前剩余提交/发布前定向门禁与物理多显示器/
  混合缩放 smoke。OCR 与取词翻译不再抢占近期产品工作，只作为最后阶段的可选兜底继续保留。
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
