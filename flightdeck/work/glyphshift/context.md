# Work Context

## 仓库边界

- 当前仓库是 Glyphshift 的唯一产品实现，不存在需要兼容的旧 workspace。
- 所有 Cargo package、桌面应用、测试支持和架构检查都位于当前仓库。
- `archive/dictionary-sources/` 只保全尚待导入的历史词典数据，不是 Runtime 或产品存储输入。
- 本机路径、真实软件、进程信息、Runtime Bundle、日志与截图只存在于 `target/local-test/`。

## 产品决定

- 工作流是持久 Runtime Intent；软件只管理身份和程序绑定；词典是独立可复用资产。
- 每个 Workflow Target 分别组合 parallel Adapter Plan、有序 Dictionary 集合和至多一个内联
  Font Policy，Feature 从完整编译结果推导。
- 一个软件同一时间最多由一个 enabled Workflow 拥有；冲突需要显式替换。
- Dictionary `/2` 是纯 `source + translation` 映射；不包含 Location、Context、keep、字体、平台、
  技术、Adapter、Hook 或保护策略。同一 Dictionary 内 source 唯一。
- 未来区域能力只有在 Runtime 能提供真实区域信号后才建立独立 Region Binding；当前产品不暴露
  Location、`main-ui`、全部位置或指定位置。
- Probe Run 只监听一个已授权目标进程，必须且只绑定一个 Dictionary 和一组显式 Adapter；一个软件
  可保存多个命名任务，但同一时刻只有一个任务占用目标 Runtime。
- Dictionary 是唯一翻译内容资产。Probe Run 文档只保存任务状态和 Dictionary 引用；内部
  Observation Index sidecar 保存 source、Adapter、计数与时间等证据，绝不持久化译文。
- Probe Run 是持续作业资产：支持 running/paused/ready/interrupted、重启恢复、搜索分页、批量
  处理和导出；支持 TextReplace 的 Adapter 可从绑定 Dictionary 发布 Live Preview。
- Font Policy 只属于 Workflow Target，保存有序字体候选和 `dictionary_matches` /
  `all_observations` coverage；不具有独立 CRUD 生命周期。
- 在线字典使用纯 payload 与外部 Artifact Descriptor；下载来源、摘要、签名和安装状态不进入
  Dictionary metadata。
- Dictionary Release、Artifact Presentation、Installation Record 与 active working copy 分别拥有
  发布、展示、来源和本地内容事实；本地编辑后安装状态派生为 modified。
- requested state、Runtime actual state 与 applied generation 分别报告。

## 架构决定

- Core、Desktop 和 GUI 不包含软件品牌、可执行文件名或 Adapter ID 分支。
- Controller Plugin、Capability Adapter、Extension 与 Route Program 是不同角色。
- Platform 是机器兼容事实，Technology 是 Catalog 分类，Adapter 是具体执行单元。
- Route Program 无代码、无 I/O、执行有界；写回只能通过 Adapter Registry。
- Translation Snapshot 与 Font Policy 正交发布，由 Decision Engine 产生最终 RenderDecision。当前
  纯 Dictionary 条目编译到 Software 已声明的所有内部路由；内部路由不是 Dictionary 字段。
- 当前产品未发布；Dictionary `/2`、Workflow `/3` 与 Target Runtime
  Deployment `/2` 直接替换旧结构，不读取、迁移或双写旧 schema。
- 所有失败路径 fail-open；不终止、不重启、不远程卸载用户正在工作的目标进程。
- Catalog、HTTP、签名和安装状态不得进入 Runtime、Decision、Workflow resolve 或 Native Adapter。

## UI 决定

- 默认页面是 Workflow 管理表；Software、Dictionary 与 Probe 是独立管理页，字体不占顶级导航。
- 创建与编辑复用 Nuxt UI Modal；管理页复用页头、表格框架、分页和确认 Module。
- 软件页不展示文字/字体功能；词典页不展示 Hook、Adapter 或字体配置。
- 词典详情主表和单行 Modal 只编辑原文与译文；便携 metadata 在单列设置 Modal 中渐进披露。
- 探针管理与详情是明确的列表/返回导航；管理页复用页头、顶部搜索/多选工具条、表格与底部分页，
  创建与导出设置进入 Modal。
- 探针详情只有一张 Dictionary + Observation Index 联合表；行内译文直接写绑定 Dictionary，忽略
  只修改证据 sidecar。长表在独立滚动区中滚动，表头和底部分页保持可见。
- Probe 表格以 5,000 条为基准由 Rust 搜索分页，Vue 只持有当前页；1 秒 revision 轮询只在变化时
  重取当前页，分页模式不叠加虚拟滚动。
- 管理表内容区保持连续表面色；空态铺满表头与分页之间空间，非空表最后一行保留底边界。
- Workflow Editor 按 Target 独立配置 Adapter 多选、Dictionary 栈和一个紧凑 Font Policy。
- 标题栏提供 Help 与 Settings 图标，不展示桌面服务连接状态；真实桌面后端不可用是阻断错误。
- Help 从 Runtime Bundle Catalog 展示公开 Adapter 信息；Settings 不保存在线翻译服务地址，只导航
  到本地资产与工作流页面，并说明未来网站/字典市场的下载方向。
- GUI 使用仓库 Playwright 验证，实际桌面与实机证据只存本地忽略目录。
- AppSettings 只承载全局且已有执行路径的偏好；当前 `/1` 包含 UI locale 与 theme，生命周期、
  更新、诊断、账号和市场能力存在后才进入设置 UI。
- Vue I18n 管理 Glyphshift 核心 UI；Rust/Tauri 跨 seam 返回稳定语义错误码和类型化参数，不返回
  vue-i18n key 或本地化句子。Dictionary 内容语言与动态 Artifact presentation locale 独立。
