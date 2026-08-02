# Work Context

## 仓库边界

- 当前仓库是 Glyphshift 的唯一产品实现，不存在需要兼容的旧 workspace。
- 所有 Cargo package、桌面应用、测试支持和架构检查都位于当前仓库。
- `archive/dictionary-sources/` 只保全尚待导入的历史词典数据，不是 Runtime 或产品存储输入。
- 本机路径、真实软件、进程信息、Runtime Bundle、日志与截图只存在于 `target/local-test/`。

## 产品决定

- 工作流是持久 Runtime Intent；软件只管理身份和程序绑定；词典是独立可复用资产。
- 每个 Workflow Target 分别组合 parallel Adapter Plan、有序 Dictionary 集合和 Font Profile
  Bindings，Feature 从完整编译结果推导。
- 一个软件同一时间最多由一个 enabled Workflow 拥有；冲突需要显式替换。
- 字典只承载便携元数据与文字规则，不包含字体、平台、技术、Adapter 或 Hook 属性。
- 字体方案是独立可复用资产，保存有序字体候选；Target Binding 决定其 Location 范围。
- 在线字典使用纯 payload 与外部 Artifact Descriptor；下载来源、摘要、签名和安装状态不进入
  Dictionary metadata。
- requested state、Runtime actual state 与 applied generation 分别报告。

## 架构决定

- Core、Desktop 和 GUI 不包含软件品牌、可执行文件名或 Adapter ID 分支。
- Controller Plugin、Capability Adapter、Extension 与 Route Program 是不同角色。
- Platform 是机器兼容事实，Technology 是 Catalog 分类，Adapter 是具体执行单元。
- Route Program 无代码、无 I/O、执行有界；写回只能通过 Adapter Registry。
- Translation Snapshot 与 Font Policy 正交发布，由 Decision Engine 产生最终 RenderDecision。
- 当前产品未发布；Dictionary `/2`、Font Profile `/1`、Workflow `/2` 与 Target Runtime
  Deployment `/2` 直接替换旧结构，不读取、迁移或双写旧 schema。
- 所有失败路径 fail-open；不终止、不重启、不远程卸载用户正在工作的目标进程。

## UI 决定

- 默认页面是 Workflow 管理表；Software、Dictionary 与 Font Profile 是独立管理页。
- 创建与编辑复用 Nuxt UI Modal；管理页复用页头、表格框架、分页和确认 Module。
- 软件页不展示文字/字体功能；词典页不展示 Hook、Adapter 或字体配置。
- Workflow Editor 按 Target 独立配置 Adapter 多选、Dictionary 栈和 Font Profile Bindings。
- 标题栏提供 Help 与 Settings 图标，不展示桌面服务连接状态；真实桌面后端不可用是阻断错误。
- Help 从 Runtime Bundle Catalog 展示公开 Adapter 信息；Settings 不保存在线翻译服务地址，只导航
  到本地资产与工作流页面，并说明未来网站/字典市场的下载方向。
- GUI 使用仓库 Playwright 验证，实际桌面与实机证据只存本地忽略目录。
