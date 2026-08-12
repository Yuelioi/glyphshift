# Work Context

## 仓库边界

- 当前仓库是 Glyphshift 的唯一产品实现，不存在需要兼容的旧 workspace。
- 所有 Cargo package、桌面应用、测试支持和架构检查都位于当前仓库。
- `archive/dictionary-sources/` 只保全尚待导入的历史词典数据，不是 Runtime 或产品存储输入。
- 本机路径、真实软件、进程信息、Runtime Bundle、日志与截图只存在于 `local-test/`。

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
- Adapter ID 只标识执行能力，不等于同一 Adapter 内的候选文本流。当前 Work 可按已有 Adapter
  证据筛选 Probe 联合表；未来 Observation Stream Identity / Binding 必须由 Adapter 提供可重匹配
  的不透明证据，属于 Probe/Workflow 运行范围，不能冒充 Region 或进入 Dictionary。
- Font Policy 只属于 Workflow Target，保存有序字体候选和 `dictionary_matches` /
  `all_observations` coverage；不具有独立 CRUD 生命周期。
- 本机字体目录属于应用级机器缓存，不进入 Workflow、Dictionary 或 AppSettings；普通启动读取
  持久缓存，只有用户显式刷新才重新扫描系统字体并替换缓存。
- 在线字典使用纯 payload 与外部 Artifact Descriptor；下载来源、摘要、签名和安装状态不进入
  Dictionary metadata。
- Dictionary Release、Artifact Presentation、Installation Record 与 active working copy 分别拥有
  发布、展示、来源和本地内容事实；本地编辑后安装状态派生为 modified。
- 首个词典发布流程只导出可重新导入的标准 Dictionary `/2` JSON；不建立账号、审批、证书管理、
  远程发布后台或额外包格式。同 ID 导入明确拒绝，不静默覆盖。
- requested state、Runtime actual state 与 applied generation 分别报告。
- 添加软件验证可访问的 Windows PE、正在运行、非重复/自身目标、当前 `x86_64` Runtime 与文字
  观察 Adapter 可用；Adapter/Hook 是否覆盖真实绘制路径仍须在添加后由激活或探针证据确认，
  不按软件名称静态猜测。
- 软件删除在停止 Runtime 前检查 Workflow 与 Probe Run 引用；有引用时保留软件和运行状态，并以
  结构化数量参数在软件表显示解除方法。单项与批量删除复用同一命令，只有真实消失的行才清除选择。
- 可分发桌面候选通过本地 Tauri override 嵌入已验证的 Release Runtime Bundle；跟踪的基础配置不
  保存本机路径。未配置代码签名时只称 unsigned candidate，不冒充公共发行版，也不自动执行安装器。

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
- 工作流实际状态直接区分软件未启动、权限、组件加载/兼容和超时等原因；错误状态可展开逐软件
  完整解释与恢复动作，不使用“需要处理”一类笼统汇总。
- Workflow 创建/编辑、Software 编辑、Dictionary 详情与 Probe Run 详情使用共享头部的独立页；
  简单创建和低频设置继续使用 Nuxt UI Modal。管理页复用页头、表格框架、分页和确认 Module。
- 独立详情页支持 Esc 返回；可见 Modal、菜单或选择器优先消费 Esc。Workflow、Software 与
  Dictionary 的未保存修改在返回、顶部导航和关闭窗口前统一保护。
- 软件页不展示文字/字体功能；词典页不展示 Hook、Adapter 或字体配置。
- 软件页提供浏览路径和 `Ctrl+Shift+F8` 两段式前台快速捕获，两条入口共享接入预检；捕获只预填
  新增 Modal，不直接创建记录。等待捕获可取消，第二次按键后才读取前台进程并返回 Glyphshift。
- 词典详情用一份显式草稿统管 metadata 与词条；主表直接编辑原文和译文并常驻空白新增行，便携
  metadata 在单列设置 Modal 中渐进披露。任何修改立即进入未保存状态，离开前必须保护草稿。
- Dictionary metadata tags 是本地搜索与在线 Catalog 分类筛选的同一套语义；Catalog 支持点击标签
  进行单值精确筛选，不新增重复的 category 字段。本地/在线模式必须保留清晰的主色选中态。
- 探针管理与详情是明确的列表/返回导航；管理页复用页头、顶部搜索/多选工具条、表格与底部分页，
  创建与导出设置进入 Modal。
- 探针详情只有一张 Dictionary + Observation Index 联合表；行内译文直接写绑定 Dictionary，忽略
  只修改证据 sidecar。长表在独立滚动区中滚动，表头和底部分页保持可见。
- Probe 的 Adapter 筛选继续位于同一联合表工具区；不为筛选重新建立“技术目录”或第二张内容表。
- Probe 表格以 5,000 条为基准由 Rust 搜索分页，Vue 只持有当前页；1 秒 revision 轮询只在变化时
  重取当前页，分页模式不叠加虚拟滚动。
- 管理表内容区保持连续表面色；空态铺满表头与分页之间空间，非空表最后一行保留底边界。
- 本地 Workflow、Software、Dictionary 和 Probe Run 行可双击非交互区域进入编辑或详情，但显式
  行级操作继续保留作为可发现和键盘可达的入口；复选框、开关、链接与按钮不触发该捷径。
- Workflow Editor 按“基础配置 / 软件与拦截 / 翻译词典 / 字体策略”四个任务分区组织；每页内部
  保持单列，Dictionary 与 Font 各自只有一个可搜索目录并在行内处理选择和优先级。Adapter、
  Dictionary 栈与 Font Policy 仍按 Target 完全隔离，通过当前 Target 选择器切换配置上下文。
  编辑页占满模块内容区；窄左栏只提供单层分区导航与问题徽标，右侧当前分区是主滚动区，
  软件、词典和字体等大集合目录再使用有界局部滚动。
- Settings 外观、Workflow 基础配置与 Software 编辑共享同一配置表单语法：配置面板以 980px
  最大宽度在可用工作区水平居中，内部复用分组标题、184px 标签轨、compact/fill 控件宽度与基于
  字段容器的单列折叠；compact 控件贴齐右侧控件轨。共享表单不改变页面级导航：Workflow 保留
  真实分区栏，Software 不建立伪侧栏，Settings 仍即时生效且没有保存动作。
- 一级管理页用完整数据表面承载搜索、Table 和分页；二级详情页在详情头下直接使用应用主背景
  画布，不再增加满高外层 Card。简单详情只显示居中配置面板，复杂详情只为真实分区导航保留侧栏
  与分隔线，空白区域始终属于页面画布。
- Settings 由标题栏入口打开，属于无父级列表的工具型详情页：复用全宽详情标题带及其背景、边界、
  高度和标题基线，但不显示无意义的返回按钮。
- Font Policy 的 coverage 使用紧凑“应用范围”选择器；字体目录标题显示缓存数量和显式刷新动作，
  不用两个满宽按钮承载二选一状态。
- 标题栏提供 Help 与 Settings 图标，不展示桌面服务连接状态；真实桌面后端不可用是阻断错误。
- Help 从 Runtime Bundle Catalog 展示公开 Adapter 信息；Settings 维护设备偏好、全局 AI 批次策略与
  独立持久化的 AI Profiles，供应商服务地址只存在于 Profile，不进入 AppSettings。
- GUI 使用仓库 Playwright 验证，实际桌面与实机证据只存本地忽略目录。
- AppSettings 只承载全局且已有执行路径的偏好；当前 `/1` 包含 UI locale、theme、设备行为、软件捕获
  快捷键与 AI 单批条目/输入 Token 策略。模型、供应商端点、凭据和过滤规则属于独立 AI Profile。
- Vue I18n 管理 Glyphshift 核心 UI；Rust/Tauri 跨 seam 返回稳定语义错误码和类型化参数，不返回
  vue-i18n key 或本地化句子。Dictionary 内容语言与动态 Artifact presentation locale 独立。
