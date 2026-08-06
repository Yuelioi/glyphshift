# Flightdeck

## Open Work

- [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — 正式 Adapter 已补齐
  可验证的官方技术文档入口；raylib 之后暂无满足门槛的真实动态目标，等待新缺口触发。
- **Focus:** [运行时能力升级 Roadmap](work/runtime-capability-roadmap/index.md) — Runtime 兼容性路由与
  一键快速探针、Desktop Acquisition Command 与 UIA 取词翻译产品入口已完成。生产 OCR 的技术评审
  已完成，当前以授权软件的可见文字作为工程样板，验证 WGC 单帧有界 ROI 与离线 OCR；两个“UIA
  无文本、像素文字可见”的成对样本改为产品提升门槛，不再阻塞技术实现。
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
