# Observation Stream Identity

Status: Deferred

## Outcome

Adapter 可以为同一目标实例中的候选文字流提供不透明、可诊断且有界的 Stream Identity；多个只读
消费者使用独立游标读取同一观察日志，目标重启后通过样本证据重匹配为 matched、degraded 或
unmatched，而不是依赖 PID、绝对地址、窗口标题或 Dictionary location。

## Decisions

- Stream Identity 属于 Runtime Observation，不进入 Dictionary Entry，也不等同于 Region Binding。
- `surface_token`、调用地址和窗口句柄只能成为会话内证据，不能单独成为跨重启 fingerprint。
- 先定义 Adapter 可选 evidence envelope、采样预算和匹配结果；没有稳定信号的 Adapter 继续只有
  单一默认流，不伪造区分度。
- 多消费者必须有独立 cursor；慢消费者只看到自己的 dropped/gap，不阻塞宿主绘制线程或其他读者。

## Delivery

- [ ] 盘点现有 TextObservation、NativeDecision context、Capture Catalog 与 diagnostics query 的信息量。
- [ ] 冻结 Stream Fingerprint、样本摘要、cursor/gap 和 Binding 的最小 Interface。
- [ ] 用合成 Adapter 验证两个流、两个消费者、容量溢出和目标重启重匹配。
- [ ] 决定哪些现有 Windows Adapter 有足够真实信号声明多流，哪些继续使用默认流。

## Current

现有 NativeDecision context 只有 Adapter ID，Target Runtime 为所有原生调用写入固定
`native.surface`；Capture Catalog 仅按 `source + adapter_id` 聚合，FileCaptureSink 在 activation
时绑定一个输出路径；diagnostics query 返回并取走最近批次。它们都不能表达候选流身份或安全的
多消费者进度。

该能力不直接增加任何软件的实时翻译覆盖，当前从主线撤下。只有真实软件出现“同一 Adapter 必须区分
多个文字来源、且该区分需要跨重启保持”的可复现需求时才恢复；此前 Dictionary、Probe 和 Workflow
继续不暴露 location 或 Stream Binding。

## Next

等待真实翻译覆盖工作提供具体需求和可验证的 Adapter 证据；当前不实现。
