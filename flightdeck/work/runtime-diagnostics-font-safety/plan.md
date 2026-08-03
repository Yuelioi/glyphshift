# 运行时诊断与字体安全计划

- [x] 冻结 Decision Trace 的最小状态、publication identity、启停与容量合同。
- [x] 在 Decision/Runtime seam 实现可关闭、非阻塞、有界的 Trace，并覆盖全部决策与失败路径。
- [x] 以完整 Publication identity 扩展 Controller ACK，并拒绝 Generation 正确但内容错误的确认。
- [x] 把有界 Trace Query 和已确认 publication identity 贯通到 Desktop Runtime 本地诊断。
- [x] 收敛 `all_observations` 文案，并以真实 GDI charset 信号保护符号字体。
- [x] 以合成合同覆盖正常替换、保护、溢出、陈旧发布原子性和 fail-open 行为。
- [x] 完成 Playwright、Rust workspace、fmt、Clippy、架构检查和生产构建。
- [x] 完成授权目标软件的符号字体与 GDI+ 风险验收。
