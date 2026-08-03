# 运行时诊断与字体安全收敛

Status: Open

## Goal

在扩大 Hook/Adapter 范围以前，让现有文字与字体写回具备可解释、可诊断且不误伤宿主界面的产品
闭环：失败继续 fail-open，默认字体覆盖仍只作用于词典命中，全量观察模式必须清楚表达风险并具有
可验证的保护行为。

## Current

Decision Trace 基础切片已交付：Decision Interface 用无额外字符串分配的 Trace 解释 NoMatch、
Matched、ContextRecorded 及四类 fail-open，并分别报告文字替换、字体替换和显式字体保护。Runtime
Kernel 默认关闭诊断，开启后以非阻塞方式保留最近 256 条并报告 dropped 数；完整 Runtime
Publication 以覆盖 Route、Translation 与 Font Policy 的 SHA-256 标识，Controller Runtime ACK 同时
校验 Generation 与内容身份。陈旧 Publication 不再可能留下新 Route 与旧 Snapshot 的部分状态。

字体安全已按真实信号收敛：ExtTextOutW、TextOutW、DrawTextW/ExW 共享 GDI `SYMBOL_CHARSET`
保护；GDI+ 暂无可靠原字体类别，不伪造保护能力。Workflow 切到 `all_observations` 时显示紧凑高影响
警示，明确未翻译文字、图标和符号的风险。

Trace Query 已贯通完整生产链：Target Runtime 通过固定上限 C ABI 输出最近批次，Windows Controller
使用远程只读缓冲取回，stdio Controller、Protocol、Session 与 Desktop Runtime 逐层映射为本地诊断
Interface。工作流实际运行时可打开诊断 Modal；它显式开启采集、每秒读取、前端最多保留 256 条，
关闭即停，并只展示公开软件/Adapter 名称、原文、结果、Generation 与短 publication identity。

升级后探针重连截图暴露的激活诊断缺口已收敛：Controller `Response::Error` 不再被误报为 malformed
message，而是把目标进程访问、远程内存、Runtime 模块/导出、远程线程、超时及 Target Runtime 拒绝
映射为稳定失败类别。失败的 Capture Runtime 会立即从 Pool 丢弃，因此“连接并继续”会真正重新发现，
不会复用半失效会话；Desktop 提示对应的重启、权限或安装修复动作。

当前唯一剩余交付缺口是授权目标软件的符号字体和 GDI+ 风险实机验收；真实路径、进程与证据必须
继续留在本机忽略目录。当前本机私有授权变量尚未配置；即使发现候选进程，也不得反推出路径或
进程标识替代显式授权输入。

## Next

- 使用当前构建完整重启 Glyphshift 与授权目标软件后重连探针，确认新提示报告的稳定失败类别或成功
  激活；随后完成 GDI 符号字体与 GDI+ 剩余风险验收。从
  [当前切片](slices/decision-trace-and-font-safety.md)和
  [Windows Runtime 合同](../../../crates/glyphshift-desktop-runtime/tests/windows_runtime_contract.rs)
  开始。先在被忽略的本机配置中提供授权可执行文件，再把原始证据只写入本机测试目录。

## Progress

- 已把临时 AI 评审按当前实现复核并路由为当前收敛项，排除重复建设与架构回退。
- Decision Trace、完整 Publication identity/ACK、陈旧发布原子性、GDI 符号字体保护与
  `all_observations` 警示已通过仓库验证。
- Target Runtime → Controller → Session → Desktop 的有界诊断查询与工作流诊断 Modal 已通过真实
  Windows 注入、stdio 传输、Tauri 映射、视觉检查和 28 项 Playwright。
- 探针激活拒绝类别、失败会话重试与可操作错误提示已通过 Controller 进程合同、Runtime Pool 回归、
  Desktop 命令合同和第 29 项 Playwright；全 workspace、Clippy、fmt、架构与生产构建继续全绿。

## References

- [稳定约束](context.md)
- [执行计划](plan.md)
- [AI 评审路由结论](references/ai-review-assessment.md)
- [Decision Trace 与字体安全切片](slices/decision-trace-and-font-safety.md)
- [产品领域语言](../../../CONTEXT.md)
- [当前总体架构](../glyphshift/references/architecture.md)
