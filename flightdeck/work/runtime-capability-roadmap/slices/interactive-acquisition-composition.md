# 交互式采集组合合同

Status: Finished

## Outcome

用确定性 Fixture 将一次性 `AcquisitionResult` 分别接入本地 Dictionary 命中、Translation Provider
回退与 External Translation Presentation，固定来源、取消、部分成功和停用清理语义；这条链路始终
是外部呈现，不修改目标软件，也不声明 `TextReplace`。

## Scope

- 为采集结果定义最小的文本选择与归一化编排，不把坐标、anchor 或平台 grant 写入 Dictionary。
- Dictionary 精确命中优先；未命中时才调用受控 Translation Provider，并保留结果来源。
- External Presentation 只持有本次会话的临时内容；取消、目标失效、再次采集和停用均只清理自身状态。
- 用合成 Acquisition、Dictionary、Provider 与 Presenter 固定调用顺序、失败隔离和隐私边界。

## Non-goals

- 不接热键、托盘、窗口、浮层视觉设计或真实在线服务。
- 不选择生产 OCR 引擎、Target Frame Source，也不改变现有 Workflow 的 `TextReplace` 候选规则。
- 不把交互式选区升级为持续 Observation Stream、Region Binding 或 Dictionary 领域对象。

## Verification

- [x] Dictionary 命中不调用 Provider；未命中才进入 Provider，并能呈现稳定来源。
- [x] 取消、拒绝、空结果、部分成功、Provider 失败和 Presenter 失败均有确定性合同。
- [x] 停用与重复请求只清理外部临时内容，不产生目标内写回或持久坐标。
- [x] Workspace test、Clippy、fmt、architecture checks 与代码自审通过。

## Review

- 新增 `product/interactive-translation` 深 Module；Dictionary port 只取得 source，Provider port 只取得
  source 与 locale，Presenter port 只取得译文和短期 anchor，三者均无法取得授权 target/grant。
- 精确 Dictionary 命中会跳过 Provider；逐块 Provider 失败不会丢弃其他成功译文，失败只保留 block
  index 与稳定类别，不复制原文到诊断。
- 新请求在解析前清理旧外部内容；权限拒绝、取消、全量未命中和 Presenter 半失败均不会留下本次活动
  状态，显式 deactivate 幂等且没有任何目标内写回端口。
- 合成组合合同 6/6 通过；60 包 workspace、全 workspace Clippy、fmt、architecture 与 diff 检查全绿。

## Next

Stage 4 仓库内后端合同完成。Desktop/UI 接入前先确定用户触发方式、会话取消和外部呈现产品形态；生产
Visual 路线仍需先取得授权真实目标与 OCR 技术选择证据。
