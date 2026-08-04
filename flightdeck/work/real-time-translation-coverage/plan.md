# 通用实时翻译覆盖计划

## Stage 1：覆盖基线与下一 Adapter

- [x] 建立当前 Adapter 支持矩阵，分别记录观察、字典命中、写回、真实目标和停用恢复证据。
- [x] [选择下一种实时写回 Adapter](slices/next-writeback-adapter.md)，必须绑定一个明确可验证的通用
  文字路径和授权目标。

## Stage 2：端到端原型

- [x] 用确定性合成目标证明 DirectWrite Unicode 观察、Dictionary 替换、失败开放、停用恢复、重复
  激活和复杂布局安全放行。
- [ ] 在授权真实目标中证明路径命中和可见译文；零命中时否决该目标，不用更多抽象掩盖结果。
  - DirectWrite TextLayout 已完成真实测试，运行成功但零命中，已否决当前目标并停止生产接入。

## Stage 3：生产接入

- [x] 仅在真实写回验收通过后加入正式 Runtime Bundle、Adapter Catalog 与 Workflow 候选。
- [x] Probe 编辑绑定 Dictionary 后保存并发布下一代预览；GDI+ 确定性目标无需重启即可使用新代次。
- [ ] 在干净重启的授权真实目标上完成
  [Probe 到 GDI+ 实时更新](slices/probe-gdiplus-live-update.md)的用户可见验收。
- [ ] 对无信号、仅采集、字典未命中和写回失败分别返回清晰状态。

## Stage 4：重复扩展

- [x] 根据支持矩阵与第一方资料选择下一文字技术，不按软件品牌累计特例。
- [x] 完成 [Qt Painter 实时写回 Adapter](slices/qt-painter-writeback-adapter.md) 的有界原型：先证明
  每次绘制都能取得原文、Dictionary 命中可替换本次绘制、热更新生效且停用恢复。
- [x] 仅在显式配置的合成宿主合同通过后，选择一个授权 Qt Widgets 目标完成可见 smoke；未命中时
  记录覆盖边界，不退化为仅注入成功。
- [ ] 完成 [WinUI/MRT 文字路径诊断](slices/winui-mrt-path-diagnostic.md)：只有声明式 `x:Uid` 也命中
  可安全替换的入口，才进入 Adapter 实现。
- [ ] 把共享 Hook、区域 Binding、字体布局和 OCR 等阻塞项送回各自 Roadmap，不抢占覆盖主线。
