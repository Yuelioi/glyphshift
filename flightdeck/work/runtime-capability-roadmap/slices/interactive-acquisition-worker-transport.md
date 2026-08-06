# 一次性 Acquisition Worker transport

Status: Finished

## Outcome

Structured/UIA 与 Visual/Frame+OCR 可以通过同一受监督进程 transport 执行一次性 Acquisition，而不复用
持续 Observation batch。Host 绑定当前 Controller 授权 grant，发送有界 selection request，接收有界
source blocks；超时、取消、协议错误和 Worker 退出都只结束本次请求。

## Scope

- 设计独立、版本化的 acquisition worker wire；不扩充 `glyphshift.isolated-worker/1` observation
  lifecycle。
- Host 输入仍是 opaque target + Controller worker grant；wire 内不出现 Desktop token、独立 PID/窗口句柄
  字段、字典、翻译或截图路径。平台进程身份只允许封装在 Controller 签发且 Host 不解析的 grant payload。
- Point / Text Range / Region、source policy、anchors、granularity、provenance、confidence 与稳定错误均
  有硬上限和 deny-unknown 解码。
- 合成 Worker 覆盖成功、权限拒绝、目标不匹配、无文字、超时、取消、畸形响应和进程退出。

## Non-goals

- 不在本切片实现真实 Windows UIA、Windows Graphics Capture 或 OCR 引擎。
- 不复用观察 Worker 的 capture producer、generation、pause、health 或 deactivate lifecycle。
- 不接 Desktop command、全局热键、Translation Provider 或 Presenter。

## Verification

- [x] Wire request/result round-trip、1 MiB 大小、块/anchor/文本/confidence/坐标上限与
  unknown-field rejection 通过。
- [x] Process Host 只向目标 grant 对应 Worker 发一次请求；本地 target 不匹配时不启动进程，超时/取消
  均回收子进程。
- [x] 合成 Worker 成功、六类稳定错误、崩溃和畸形响应通过；无截图、路径或机器数据落盘。
- [x] 59 包 Workspace test、全目标 Clippy、fmt、architecture checks 与代码自审通过。

## Review

- `glyphshift.acquisition-worker/1` 与 observation worker lifecycle 完全独立，没有 producer、generation、
  pause、health、deactivate 或 capture batch 字段。
- SDK 与 Host 分包：Adapter/Worker author 只依赖 L0 SDK；进程、线程、超时、取消和 artifact 路径只位于
  L3 Host，不形成 Adapter → Host 反向依赖。
- Desktop target token 只在 Host 本地比较，不上 wire；Controller grant 作为 opaque payload 转交，Host
  不解析平台进程身份。grant 类型不实现 `Debug`，避免无意记录 payload。
- 响应必须只含一种 provenance；混合 Structured/Visual、空成功、过量/越界字段、unknown field 和错误
  schema/request identity 均按畸形消息拒绝。

## Next

继续 [Windows UIA 交互式 Provider](windows-uia-interactive-provider.md)：复用既有 Controller process
grant 和 acquisition worker wire，以确定性标准控件进程验证 Point / Text Range、密码、目标身份和
超时；之后再评估生产 Target Frame Source 与 OCR 引擎。
