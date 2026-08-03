# 运行时诊断与字体安全收敛

Status: Finished

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

完整 Native 注入像素合同已补齐：同一 `all_observations` Font Policy 下，普通 GDI 文字使用替换
字体，GDI `SYMBOL_CHARSET` 保持原像素，GDI+ Symbol 字体则发生像素变化；停止 Runtime 后三者
全部恢复。这把 GDI 保护和 GDI+ 剩余风险从纯函数提升为真实进程/DLL 合同。

授权 AE 的真实字体写回验收已完成：`all_observations` 下普通 GDI 菜单与 GDI+ 面板均使用替换
字体，诊断同时观测到两条路径；菜单中的 `ETO_GLYPH_INDEX` 在换字体前恢复为 Unicode，避免旧字体
glyph ID 被新字体误解。停止 Runtime 后截图与基线逐像素一致，图标保持完整。原始路径、进程、
截图和日志全部留在本机忽略目录。

## Next

None.

## Progress

- 已把临时 AI 评审按当前实现复核并路由为当前收敛项，排除重复建设与架构回退。
- Decision Trace、完整 Publication identity/ACK、陈旧发布原子性、GDI 符号字体保护与
  `all_observations` 警示已通过仓库验证。
- Target Runtime → Controller → Session → Desktop 的有界诊断查询与工作流诊断 Modal 已通过真实
  Windows 注入、stdio 传输、Tauri 映射、视觉检查和 28 项 Playwright。
- 探针激活拒绝类别、失败会话重试与可操作错误提示已通过 Controller 进程合同、Runtime Pool 回归、
  Desktop 命令合同和第 29 项 Playwright；全 workspace、Clippy、fmt、架构与生产构建继续全绿。
- Native 字体安全像素合同已通过：普通 GDI 字体替换、GDI Symbol 保持、GDI+ Symbol 风险和停止
  恢复均由真实 DLL 注入验证；全 workspace、Clippy、fmt 与架构检查继续通过。
- 授权 AE 的 Debug 可见验收发现并修复 GDI glyph-index 换字体乱码；Release Runtime Bundle 随后
  通过同一真实宿主合同，诊断覆盖 GDI/GDI+，停止后恢复与基线完全一致。完整 workspace、Clippy、
  fmt、架构检查和 36 项 Playwright 均通过。

## References

- [稳定约束](context.md)
- [执行计划](plan.md)
- [AI 评审路由结论](references/ai-review-assessment.md)
- [Decision Trace 与字体安全切片](slices/decision-trace-and-font-safety.md)
- [产品领域语言](../../../CONTEXT.md)
- [当前总体架构](../glyphshift/references/architecture.md)
