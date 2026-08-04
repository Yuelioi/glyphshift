# 通用实时翻译覆盖计划

## Stage 1：覆盖基线与下一 Adapter

- [ ] 建立当前 Adapter 支持矩阵，分别记录观察、字典命中、写回、真实目标和停用恢复证据。
- [ ] [选择下一种实时写回 Adapter](slices/next-writeback-adapter.md)，必须绑定一个明确可验证的通用
  文字路径和授权目标。

## Stage 2：端到端原型

- [ ] 用确定性合成目标证明 Unicode 观察、Dictionary 替换、失败开放、停用恢复和重复激活。
- [ ] 在授权真实目标中证明路径命中和可见译文；零命中时否决该目标，不用更多抽象掩盖结果。

## Stage 3：生产接入

- [ ] 仅在真实写回验收通过后加入正式 Runtime Bundle、Adapter Catalog 与 Workflow 候选。
- [ ] 从 Probe 采集原文、编辑 Dictionary、保存并发布到运行时，完成一次用户可见端到端合同。
- [ ] 对无信号、仅采集、字典未命中和写回失败分别返回清晰状态。

## Stage 4：重复扩展

- [ ] 根据支持矩阵中覆盖缺口选择下一文字技术，不按软件品牌累计特例。
- [ ] 把共享 Hook、区域 Binding、字体布局和 OCR 等阻塞项送回各自 Roadmap，不抢占覆盖主线。
