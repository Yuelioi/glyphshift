# 后端基础纵向切片

Status: Finished

## Deliverable

通过已确认的 `AiTranslation` Interface 提供可测试的 pending Dictionary、候选规划、Profile 持久化与
至少两个可替换 Provider Adapter（生产 HTTP 与合成 Adapter），为桌面 IPC 和两个 GUI 入口提供真实纵向能力。

## Confirmed test seams

- Dictionary package / Translation Snapshot 的公开合同：pending 可持久化，completed 才进入运行时映射。
- `AiTranslation` 外部 Interface：规划、启动、查询、取消；测试只观察计划、任务状态与结果。
- Provider 内部 Seam：生产 HTTP Adapter 与确定性合成 Adapter；测试从 `AiTranslation` Interface 发起，
  不断言内部调用次数或私有 JSON 构建步骤。
- Dictionary draft 与 Probe Dictionary 两个写回 Adapter：都必须在写回时再次检查空白状态和版本。

## Steps

1. [x] Red → green：pending Dictionary 可保存、加载，但不会编译进 Translation Snapshot。
2. [x] Red → green：过滤规划只选择空白译文，并给每个跳过项稳定原因。
3. [x] 初版 Red → green：Profile 元数据通过系统 Adapter 引用凭据；该决策后续已由用户明确改为
   Profile `/4` 明文 Key，并由当前发行修复 Work 接管迁移与回归。
4. [x] Red → green：一个合成 Provider 完成小批任务、结构校验、取消与迟到结果丢弃。
5. [x] 接入首个 HTTP Adapter，并为六种协议建立独立 codec 合同。
6. [x] 接入桌面后端命令，运行本 Slice 的定向 Rust 回归。

## Verification

- 每个循环先看到目标测试因缺少行为而失败，再实现最小行为让其通过。
- 只运行受影响 package 与 desktop-backend 定向测试；Slice 完成时再跑相关边界回归。
- 所有网络响应、路径与凭据 fixture 必须是合成值，真实请求只允许进入 `local-test/`。
- AI、Dictionary、Capture、Desktop Backend 与 Desktop Shell 的相关测试全部通过；架构合同通过。
