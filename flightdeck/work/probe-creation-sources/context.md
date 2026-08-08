# Work Context

## Stable facts

- Probe Run 的持久合同仍是一个 Software、一个 Dictionary 和一组 Adapter；来源选择不进入已启动
  Probe 的 Runtime wire。
- Software Library 和 Dictionary Library 是可复用资料来源，不是创建 Probe 的前置条件。
- 当前前台捕获已经能返回路径、建议名称、架构、运行状态和现有 Software 命中；普通界面不保存 PID
  或窗口标题。
- 普通创建命令支持已有 Dictionary 或创建用户命名的新 Dictionary，但只接受已有 `softwareId`。
- Quick Probe 命令按可执行路径复用/创建 Software，始终创建系统命名的临时 Dictionary，并自动选择
  兼容 Adapter；ownership ledger 已覆盖失败补偿、保留、引用保护和崩溃恢复。
- Vue 不能通过多个已有 command 自行模拟原子创建；临时资产的所有权必须由 Desktop 单一 Module 落盘。

## Constraints

- 不按显示名称、ID 前缀、路径字符串格式或 UI 模式推断资产所有权。
- 当前程序来源必须由用户明确捕获并通过同一软件 preflight；不扫描或持久化整机进程列表。
- 真实程序路径、进程值和实机证据只进入 `local-test/`，不写入此 Work。
- 未发布请求结构直接演进，不保留旧 command 或旧 payload 的兼容分支。
- 开发循环只运行 Desktop Shell 与 Probe 页面定向门禁，不运行全 workspace 测试。

## Source combinations

| Target source | Dictionary source | Result |
| --- | --- | --- |
| Software Library | Dictionary Library | 普通 Probe，复用两项资产 |
| Software Library | Temporary Dictionary | 临时 Probe，只拥有新 Dictionary |
| Active Process | Dictionary Library | 复用或临时创建 Software，复用 Dictionary |
| Active Process | Temporary Dictionary | 复用或临时创建 Software，并拥有新 Dictionary |

只要有一项资产由本次创建会话拥有，Probe 就保留 ownership session；两项都复用时，创建成功后不保留
临时 session。
