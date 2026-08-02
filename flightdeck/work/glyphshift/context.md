# Work Context

## 仓库边界

- 当前仓库是 Glyphshift 的唯一产品实现，不存在需要兼容的旧 workspace。
- 所有 Cargo package、桌面应用、测试支持和架构检查都位于当前仓库。
- `archive/dictionary-sources/` 只保全尚待导入的历史词典数据，不是 Runtime 或产品存储输入。
- 本机路径、真实软件、进程信息、Runtime Bundle、日志与截图只存在于 `target/local-test/`。

## 产品决定

- 工作流是持久 Runtime Intent；软件只管理身份和程序绑定；词典是独立可复用资产。
- 每个 Workflow Target 引用有序 Dictionary 集合，Feature 从编译结果推导。
- 一个软件同一时间最多由一个 enabled Workflow 拥有；冲突需要显式替换。
- 字典同时承载文字规则、默认字体与逐词条字体覆盖，可设置一次适用 Hook。
- requested state、Runtime actual state 与 applied generation 分别报告。

## 架构决定

- Core、Desktop 和 GUI 不包含软件品牌、可执行文件名或 Adapter ID 分支。
- Controller Plugin、Capability Adapter、Extension 与 Route Program 是不同角色。
- Route Program 无代码、无 I/O、执行有界；写回只能通过 Adapter Registry。
- Translation Snapshot 与 Font Policy 正交发布，由 Decision Engine 产生最终 RenderDecision。
- 所有失败路径 fail-open；不终止、不重启、不远程卸载用户正在工作的目标进程。

## UI 决定

- 默认页面是 Workflow 管理表；Software 与 Dictionary 是独立管理页。
- 创建与编辑复用 Nuxt UI Modal；管理页复用页头、表格框架、分页和确认 Module。
- 软件页不展示文字/字体功能；词典规则行不重复设置 Hook。
- GUI 使用仓库 Playwright 验证，实际桌面与实机证据只存本地忽略目录。
