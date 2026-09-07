# GDI 双架构实现与验收边界

## 交付结构

App 保持 x64；现有 Controller、目标 Runtime 和首批 GDI 原生适配器分别编译为 x86/x64。一个 Adapter ID/版本对应多个架构工件，界面合并为同一项，由目标架构选择对应工件。译文匹配仍在目标 Runtime 内完成，共享源码不增加逐次绘制 IPC。

首批 x86 入口为 TextOutW、ExtTextOutW、DrawTextW/DrawTextExW。Qt、MonoGame、DirectWrite 等其他适配器未新增 x86 工件，不能从基础设施双架构推导所有适配器都已支持 32 位。

## 加载与进程边界

- Runtime Bundle schema 4 增加架构分组，继续读取旧 schema 3；构建入口同步生成两种 Controller、Runtime 和适配器，并通过生产 loader 校验。
- Registry 在逻辑 ID/版本下按物理工件架构解析，拒绝同架构冲突。导出的原生 descriptor 保持原值，不能为了选择工件篡改其能力声明。
- PE 头检查机器类型与可选头格式。构建阶段由同位数 Controller 导出原生元数据；App 对同位数工件比较实际 descriptor，对异位数工件校验哈希、PE 和静态元数据，目标 NativeHost 激活时复验实际 descriptor。
- 两种 Controller 各自枚举授权进程族，再筛选自身架构。传输层为内部 token 加架构作用域，保留根进程排序，允许同一会话连接 x64 父进程及 x86 子进程。
- 架构未知时不猜测宿主架构。远程写入前在已打开的同一进程句柄上复核创建时间和架构，拒绝进程身份变化与错误位数。
- 系统函数远程地址按本地函数实际所属模块计算 RVA，兼容系统导出转发。一个 Controller 不可用时保留另一架构的可用连接。

## 验证

- 活动包入口 `scripts/test.ps1`：492 passed、0 failed、21 ignored；未执行归档 UIA/OCR。
- 最后一次路由排序调整后，protocol 定向单元测试 4 项通过。
- 用户发现快捷探针入口仍有 x64 硬编码后，补充桌面壳门控修复及端到端产品命令回归：合成 x86 PE 从预检查到适配器筛选、探针创建通过，缺少 x86 工件仍拒绝；桌面壳 85 项通过。底层 Runtime 合同不能代替产品入口验收。
- 双架构原生合同 3 项通过：同一会话的 x64 父进程/x86 子进程替换、更新、采集、恢复和重新连接；x86 TextOutW/DrawTextW；x86 字体替换及恢复。
- Registry 多架构选择和冲突拒绝、PE 头验证、进程身份/架构拒绝均有定向测试。
- 受影响生产库严格 Clippy 通过。all-targets 检查被既有测试辅助代码的 `chunks_exact_to_as_chunks` 风格告警阻挡，未将其表述为通过。
- review-app 同步构建、两次生产加载器校验和进程启动通过；Playwright CLI 因 WebView 未开放调试连接而未完成界面 smoke，界面仍待人工验收。

这些是合成合同证据。尚未指定并验收真实 x86 软件，不据此宣称某个具体软件或所有 32 位程序兼容。

## 构建兼容性

当前 retour 0.3.1 在 i686 上无条件生成 win64 ABI 实现，与工具链不兼容。仓库保留带许可证的源码副本，仅为该实现添加 x86_64 条件；未修改全局 Cargo 缓存，未迁移到上游开发版 API。原因与上游参考见 [补丁说明](../../../../vendor/retour/GLYPHSHIFT.md)。

后续验收 App 必须运行 `scripts/review-app.ps1`，确保桌面壳与双架构 Runtime 来自同一 checkout。构建产物使用全局 Cargo target 配置；机器证据仅存放在本地测试目录。
