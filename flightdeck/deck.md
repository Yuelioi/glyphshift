# Flightdeck

## Open Work

- **Focus:** [通用实时翻译覆盖](work/real-time-translation-coverage/index.md) — Unity Mono Standard UI 已
  完成生产接入。未发布 IL2CPP Standard UI observe-only 原型也已完成：Unity 2021 LTS 动态分数与
  Unity 6 搜索更新分别取得 14/72 条 Observation，均正常停用退出。当前处于生产准入门槛：仍缺外部
  Shipping-like 同后端自绘负例，不能建立可靠拒绝或进入 Catalog/Runtime Bundle。当前不安装 Unity、
  不做第五轮无界搜索，也不回退到 AE 等旧验收尾项。

## Project links

- [领域语言](../CONTEXT.md)
- [产品契约](../PRODUCT.md)
- [设计系统](../DESIGN.md)

## 验证纪律

- 开发循环只运行受影响 package、合同与页面的定向测试，并在约 20 秒没有新输出时主动回报状态。
- 大 Slice 自审运行相关边界回归，不默认升级为全仓测试。
- 全仓测试只用于提交前、发布前，或 workspace 拓扑、公共 ABI / wire contract 发生实质变化时；运行前
  明确说明必要性。
