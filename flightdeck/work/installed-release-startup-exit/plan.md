# 诊断计划

1. [x] 建立快速、确定、无人值守的启动存活探针，并在已安装发行版上捕获用户所述症状。
2. [x] 最小化失败条件，区分安装布局、Runtime Bundle、持久数据、权限重启与 WebView/前端启动阶段。
3. [x] 列出并逐项证伪 3–5 个根因假设，保留最小必要的定向诊断证据。
4. [x] 在正确 seam 添加 `/2` 无损迁移与单个不可迁移词典不阻断启动的回归合同，实施原子迁移、错误
   隔离与可恢复诊断。
5. [x] 把 AI Profile artifact 升级为明文 Key 合同，迁移能够关联的旧系统凭据，移除正常读写与删除对
   Windows Credential Manager 的依赖。
6. [x] 设置页增加默认遮蔽的 Key 显示/隐藏交互与明文风险文案；修复 Codex 模型 ID、连接诊断和旧值迁移。
7. [x] 排除归档 UIA 的活动 Rust/架构门禁、生产 NSIS candidate、安装后普通启动、数据迁移、明文 Key
   与 Codex smoke 全部通过。禁止原始 workspace Cargo 命令。
