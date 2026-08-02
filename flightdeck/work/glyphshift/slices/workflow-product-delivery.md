# 工作流产品交付

Status: Open

## Outcome

把当前按 Software 隐式绑定单一 Catalog、行内文字/字体开关的桌面产品，替换为可持久化、
可启停、可组合多软件与多词典的 Workflow 产品。Dictionary 是独立替换资产，既可替换文字，
也可提供默认字体和逐词条字体覆盖；Workflow 激活后自动维持期望，不再提供“开始翻译”按钮。

决定依据见[工作流、软件与多词典模型](desktop-configuration-model.md)和
[工作流配置模型调研](../references/workflow-configuration-research.md)。

## Current

- Runtime replacement rule 与纯 Workflow Composition 两个纵切均已完成：完整匹配键、
  词典字体行为、确定性优先级、冲突诊断和 `glyphshift.runtime/2` wire 均有合同覆盖。
- 第三个 Desktop persistence and activation 纵切已完成。Backend 通过公开 seam 持久化
  `dictionaries/<id>.json` 的 `glyphshift.dictionary/1`、`workflows/<id>.json` 的
  `glyphshift.workflow/1` 与 `workflow-state.json`，重开可恢复 Definition、revision、
  Activation 和软件选择。
- Dictionary 支持创建、按 ID 读取、整份 revision CAS 更新、条目新增/替换/批量删除、
  词典批量删除和引用保护；规则完整保存 keep/replace、context、Adapter scope、默认字体与
  inherit/unchanged/substitute。
- Workflow 支持创建、按 ID 读取、复制、revision CAS 更新、批量删除、启用、停用和显式
  冲突替换。启用与更新 active Definition 都先完整调用 Composition；失败不改变旧 active set。
- 同一 Software 只能被一个 enabled Workflow 占用；删除被引用 Dictionary/Software 或启用中
  Workflow 会被拒绝。加载 Backend 时会重新验证 active Definition、引用和占用不变量。
- 旧 `catalogs/<software-id>`、`fonts/<software-id>`、`TranslationEdit/Delete`、
  software-owned Workspace 与 Tauri 旧词条命令已删除；不保留迁移或双存储分支。
- `workflow_runtime_spec` 已作为只读交接面把持久化 Definition 编译为目标 Publication；当前
  Backend 进一步提供 `EffectiveWorkflowIntent`，一次交付全部 Target 的 RuntimeSpec 与推导
  Feature，不让 Runtime Pool 或 Tauri 重新组合词典。
- 第四个 Runtime reconcile 纵切已完成。`DesktopRuntimePool` 通过 `reconcile_workflow`、
  `stop_workflow`、`replace_workflow` 与 `refresh_workflow` 接收 Effective Intent；多 Target
  并行、不同 Workflow 隔离、完整替换、共享 Dictionary generation 扇出和重启恢复均已锁定。
- Runtime 的 Windows Controller 与同进程测试实现位于私有内部 seam；公开状态分别保存
  requested Features、实际 active Features 与 applied generation。单 Target 停止失败会保留
  最后有效 Session，其他 Target 不回滚，错误按 Software 报告。
- 第五个 Tauri commands and frontend model 纵切已完成。Backend snapshot 提供确定性的
  Software、Dictionary/Workflow summary 与 Activation；Tauri `desktop_snapshot` 合并产品配置和
  `workflowRuntimeStatus`，并提供 Dictionary/Workflow detail、完整 CRUD、启停和刷新产品命令。
- 已启用 Workflow 或其 Dictionary 更新时会重新 reconcile 并推进实际 generation；桌面重开会
  恢复持久化 Activation，刷新按 Workflow 重新发现和应用目标。
- Tauri Runtime View 不再序列化目标实例 ID、窗口标题、Controller token、路径或 Adapter ID；
  只投影 Software ID、是否发现、requested/active Feature、applied generation 与按 Software 错误。
  旧 discover、per-Software Feature 与 per-Software refresh 命令已经从 handler 删除。
- Vue 与浏览器 localStorage adapter 已统一到 `workflows[]`、`software[]`、`dictionaries[]`、
  Activation 和 Workflow Runtime Status 产品形状；detail 按 ID 加载。旧
  `translations[softwareId]`、软件行 Feature 开关和旧 Runtime invoke 已删除。
- 第六纵切已有可运行骨架：默认 Workflow 表、`工作流 / 软件 / 词典` 顶栏、独立 Dictionary
  编辑和只管理身份/程序绑定的软件表。Workflow 已可从可搜索列表选择多个 Software，并按选择
  顺序为全部 Target 建立 Dictionary 栈；行级复制和确认删除已接产品命令。
- 首次使用链路已闭合：缺少 Software/Dictionary 时“新建工作流”仍可打开，并给出直达创建
  页面的恢复动作；新增 Software 在一个对话框中确认名称和完整程序路径，浏览文件只填充路径；
  Dictionary 可独立创建，且不按 Software 归类或筛选。
- Workflow 与 Dictionary 列表已统一到 Yotta 高密度表格合同：搜索、筛选、显示列、多选、
  批量动作、表格和分页。Workflow 支持批量启用/停用/删除；Dictionary 支持批量删除并继续由
  Backend 执行引用保护。
- Dictionary 规则编辑器不再暴露内部 Location；Runtime bundle 为 Adapter 提供 GDI/GDI+
  等显示标签，Frontend 在整份词典上保存一次 scope ID，不在规则行重复显示 Hook，也不显示
  内部 ID。
- Software、Workflow 与 Dictionary 均新增可选用途描述并贯通浏览器模型、Tauri 命令、Backend
  summary/detail 和持久化 artifact；旧数据缺少字段时按空描述读取。Workflow 复制保留描述，
  Backend 合同证明三类描述在重开后仍可读回。
- Workflow 新建和编辑已复用同一个弹窗；编辑不再跳独立页面，可修改名称、描述、Software
  集合与有序 Dictionary 栈。三个管理页统一图标、标题与描述页头，Software 表增加页级/行级
  选择框和批量删除；主按钮深色前景通过 4.5:1 计算对比度回归。
- Software 页已彻底移除 capability 筛选、可用功能列和文字/字体 Runtime 状态，只管理软件
  名称、用途说明、程序位置与增删改；运行能力和实际状态只在 Workflow 执行语境出现。
  Software 的页级与行级选择框与 Workflow、Dictionary 使用相同的 Nuxt UI 透明 checkbox。
- 2026-08-02 的 `desktop_refresh_runtimes not found` 由旧 WebView 模块与新 Shell handler 错配
  引起，不是 Software 版本问题。开发目录的依赖树处于未完成的包管理器迁移状态，Vite 可返回
  新源码但无法编译 HMR。Shell/Frontend 现先协商 Desktop API v2，错配时阻止产品命令并明确
  提示重启；描述字段把协商版本提升到 Desktop API v3，本机字体列表又把同一协商版本提升到
  Desktop API v5。`scripts/dev-app.ps1` 会校验真实
  依赖入口并在重启后自动 `npm ci` 修复。
- 后续实测发现旧 Vite 进程没有经过更新后的启动脚本，且活动依赖树连 Nuxt UI manifest、
  `Checkbox.vue` 与 vue-tsc 入口都缺失。已由 detached 启动流程只重启 Glyphshift 开发进程并
  执行 `npm ci`；活动 1430 服务恢复，根目录生产构建、完整依赖树与直接复用活动服务的
  Playwright 均通过。启动脚本的哨兵现覆盖实际使用的 Nuxt UI 组件，不再接受半安装目录。
- Dictionary 规则编辑器现已使用与资产列表相同的搜索、行为筛选、显示列、多选、批量删除、
  表格与分页合同；固定底行负责新增规则。“行为”展示列已拆成可独立修改的文字处理、译文、
  字体处理和本机字体列。Windows Shell 同时枚举系统级与当前用户级字体，默认字体与逐词条字体
  均从实际列表选择，不再要求手写字体名。
- 词典创建弹窗、词典列表和编辑页共享字典级“适用 Hook”；规则表不再重复显示 Hook 列或
  打开逐词条范围弹窗。创建弹窗的 Select 通过 body portal、显式浮层 z-index 和向上展开避开
  Modal footer；紧凑视口合同同时验证三项完整可见且菜单不与 footer 重叠。逐 Target 词典编排
  仍待实现。
- 前端组件体系已完成全量收口：业务 Vue 不再直接使用原生 button/input/select/textarea/table/
  dialog，也不直接依赖图标 Vue 包。管理页统一复用 `ManagementPageHeader`、
  `ManagementTableFrame`、`ManagementFormModal` 与 `ConfirmDialog`；页面组件只组合 Nuxt UI
  的 UButton、UInput、USelect/USelectMenu、UTable、UCheckbox、UModal 等 primitive。主题密度、
  主按钮、表格和浮层层级集中在 Nuxt UI AppConfig，源码合同会阻止原生控件和重复手写样式回流。
- Nuxt UI Modal 的 overlay/content 使用显式层级，高于 dropdown/popover 与 sticky UTable；
  Playwright 用实际重叠点命中合同锁定“底层表头不得覆盖弹窗”。
- Software、Workflow 与 Dictionary 的用途描述现为独立表格列，并统一进入“显示列”配置；
  Dictionary 创建、详情编辑、保存和列表检索继续使用同一持久化字段。“显示列”已改为 Yotta
  同形的右侧勾选菜单，并提供分组后的“恢复默认列”；共享分页完整提供首/前/页码/后/末与每页
  数量，字体等长列表使用无原生箭头的统一细滚动条。
- Workflow 启用命令失败不再静默。表格会同时读取页级与逐 Workflow 消息，并用语义化 Nuxt UI
  Alert 展示；Backend 保留 Composition 的具体 `ResolveError`，Tauri 分别返回空词典、语言不匹配、
  未知位置、丢失引用和软件占用的可恢复提示。空词典 Activation 仍被拒绝且不会写入 enabled set。
- 既有全 workspace、Clippy 与格式门槛仍保持；本轮前端生产构建、Playwright 35/35、浮层交互
  重复回归 5/5 和 Impeccable detector 零命中；启用错误修复后生产构建、Playwright 36/36、
  Backend 11/11 与 Desktop Shell 16/16 通过。开发脚本同时通过 1 个目标进程 Runtime 合同与
  3 个桌面 Runtime 合同，并在真实 Tauri WebView 中复核统一页头、满宽表格、透明选择框和
  低亮度主按钮；Software 纯资料管理和 Modal 层级均有专门回归，真实 Tauri 桌面壳已完整重启
  并响应。依赖真实宿主的测试保持 ignored。
- 用户本地修改的 `plugins/adobe.premiere-pro/dictionaries/zh-CN.json` 不属于本 Slice，
  任何测试、格式化、暂存和提交都必须排除。

## Confirmed model

### Dictionary replacement rules

- Dictionary 拥有 `id`、`name`、`description`、`locale`、`revision`、`defaultFont`、可选
  Adapter scope 和 entries。
- `defaultFont` 为 `unchanged` 或 `substitute(family)`。
- Dictionary 的 Adapter scope 统一应用到默认字体和全部 DictionaryEntry；DictionaryEntry
  按 location、可选 context 与 source 匹配。底层仍可读取旧逐词条 scope，产品保存会规范化
  为字典级范围。
- Entry 的文字行为为 `keep` 或 `replace(text)`。
- Entry 的字体行为为 `inherit`、`unchanged` 或 `substitute(family)`。
- 因此一条规则可以表达：只翻译、只换字体、翻译并换字体、明确保持原字体。
- 命中条目时，字体覆盖优先于词典默认；未命中条目时，第一个声明指定默认字体的启用词典
  可提供全局字体。无字体规则时保持宿主字体。

### Workflow composition

- Workflow 拥有 `id`、`name`、`description` 和一个或多个 WorkflowTarget；Target 引用一个 Software 和有序
  DictionaryBinding 集合。
- Target 至少有一份启用词典。请求的 TextReplace / FontSubstitute Feature 从编译结果
  推导，不在软件行保留全局开关。
- 多词典按“高优先级在上”解析，同一个完整匹配键只接受第一条规则，并生成可见冲突诊断。
- 多个 Workflow 可以同时启用，但 Software 集合必须互不重叠。
- 启用重叠 Workflow 默认拒绝并返回冲突；只有显式 replace 操作才原子停用旧 Workflow、
  启用新 Workflow。禁止字段级静默合并。
- Workflow Definition、Activation 与 Runtime Status 分开。目标未运行时仍保存期望；刷新
  发现新实例后按已启用 Workflow 恢复。

## Deep module seam

新增纯 `glyphshift-workflow` Module，隐藏词典优先级、字体规则、冲突诊断和 Feature 推导：

```text
resolve(workflow, software inputs, dictionaries) -> CompiledWorkflow
```

`CompiledWorkflow` 按 Software 输出 TranslationSnapshot、FontPolicy、requested Features
与 diagnostics。Desktop Backend 负责持久化和把 Software 的 Route/Location 输入交给
Module；Desktop Runtime 只接收编译后的 Effective Target Intent。Tauri、Vue、Controller、
Runtime Kernel 和 Native Adapter 不合并词典，也不按 GDI/GDI+ 写产品分支。

不要为了测试暴露内部 resolver、排序器或文件仓储。测试只从上述公开 seam、
`DesktopBackend` 公共 CRUD/activation seam 和桌面用户操作进入。

## Delivery

### 1. Runtime replacement rule contract (Complete)

先在 `glyphshift-decision` 写失败合同，再改实现：

- 词典默认字体可让未命中翻译的原文产生 Font-only Decision。
- 命中条目的 `unchanged` 会阻止词典默认字体。
- 命中条目的 `substitute(family)` 会覆盖词典默认字体。
- `keep text + substitute font` 产生 Font-only；`replace text + unchanged font` 产生
  Text-only；两者同时指定产生 Text+Font。
- context 与字典级 Adapter scope 对文字、默认字体和逐词条字体使用同一个有效范围。
- digest、Runtime Publication JSON wire round-trip 与旧 generation 原子切换合同同时更新。

建议把现有 location-only `FontPolicy` 深化为可解析默认与逐条规则的结构，而不是把字体字段
硬塞进 Native ABI。Decision Engine 应一次解析出 RenderDecision；Adapter 继续只消费最终
TextDecision / FontDecision。

### 2. Pure Workflow Composition Module (Complete)

在根 `Cargo.toml` 注册 `crates/glyphshift-workflow`，测试公开 `resolve`：

- WF-001：一个软件 + 一个词典，编译出 Text-only。
- WF-002：同一词条的指定字体编译出 Text+Font。
- WF-003：词条 `unchanged` 阻止词典默认字体。
- WF-004：两个词典相同完整键时高优先级胜出，并返回冲突来源。
- WF-005：同一词典复用于两个软件，只对各自有效 Location 编译；无效 Location 在激活前
  返回解释性错误。
- WF-006：空 Target、缺失词典、locale 不匹配或无有效规则时拒绝，不产生半成品 Intent。

### 3. Desktop persistence and activation (Complete)

用新产品存储替换隐式一对一 Catalog；V1 未发布，不增加迁移/兼容分支：

```text
dictionaries/<dictionary-id>.json   glyphshift.dictionary/1
workflows/<workflow-id>.json         glyphshift.workflow/1
workflow-state.json                  enabled workflow ids/revisions
```

扩展文件仍描述 Software、Location、Route 与 Capability。Backend 公共合同需要覆盖：

- Dictionary：创建、读取、更新（revision CAS）、删除、批量删除、引用阻止删除。
- Dictionary：创建、更新名称/描述/locale/默认字体/适用 Hook。
- Entry：创建、更新原文/译文/字体行为、删除和批量删除。
- Workflow：创建、读取、复制、更新、删除、批量删除、启用、停用、显式替换冲突。
- 编译先完整验证所有 Target，再原子写 Activation；任何失败不得改变旧 active set。
- 重开 Backend 后 Definition、Activation、Dictionary revision 与选择状态一致。
- 删除 Software 时，若被 Workflow 引用则拒绝并列出引用；不得级联删除用户词典。

删除旧 `catalogs/<software-id>` 产品路径只能与新合同在同一纵切完成，不能保留双模型或自动
迁移分支。测试 fixture 全部使用合成软件、合成词典和临时目录。

### 4. Runtime reconcile (Complete)

把 `DesktopRuntimePool::set_features` 的上游改为 Workflow intents：

- 同一 Workflow 的多个 Software 可并行激活。
- 停用一个 Workflow 只停止它拥有的 Software。
- 显式替换重叠 Workflow 时，旧 Intent 被完整替换，不合并词典或字体字段。
- Dictionary 保存后，所有引用它的 active Target 重新编译并推进 generation。
- 软件关闭后 active Workflow 保持 enabled；刷新发现新实例后重发最后有效 Publication。
- Runtime actual status 只能由 ACK 投影，不反向写 Definition/Activation。

### 5. Tauri commands and frontend model (Complete)

Tauri 只暴露产品命令和序列化 View，不暴露 Controller token、进程标识或内部 Adapter ID。
删除前端 `translations[softwareId]` 与软件行 Feature 开关，模型改为：

- `workflows[]`、`software[]`、`dictionaries[]`
- `workflowRuntimeStatus[workflowId]`
- 进入编辑器后按 ID 加载 Workflow 或 Dictionary detail

浏览器 Playwright fixture 可用同形状的内存/localStorage adapter，但 Tauri 生产命令是产品
真相；两边不可维护两套业务规则。

### 6. UI: Workflow / Software / Dictionary (In progress)

用户已指定现有 Yotta 工作流表作为视觉和交互基准。沿用其高密度深色表格、顶栏导航、
搜索/筛选、多选、分页和行级菜单；使用 Nuxt UI 组件与 Tailwind CSS，CSS 只保留 token、
根规则和无法由 utility 表达的极少部分。

- 顶栏：`工作流 / 软件 / 词典`，设置保留为图标。
- 工作流主页：名称、软件摘要、词典摘要、期望/实际状态、启用开关、编辑/复制/删除；支持
  多选、批量启停/删除、搜索、筛选和分页。
- 软件页：只管理名称、用途说明与完整程序路径；不展示文字/字体能力或运行状态。
- 词典页：独立词典表，显示可配置的“适用 Hook”列；进入后固定底行快速新增。适用 Hook 在
  创建时为整份词典设置一次，编辑页只读展示；每行按原文、译文、文字处理、字体处理、字体的
  顺序编辑，并支持批量删除。默认字体菜单高于 sticky 表格，原文/译文用浅描边标明编辑区。
- Workflow 编辑器：添加多个 Software，为每个 Software 维护有序词典集合；不出现第二套
  字体配置表单。
- 没有“开始翻译”按钮。启用 Workflow 就是持续期望。

编辑 UI 前必须读取 Impeccable `craft-floor.md`。GUI 只通过仓库 Playwright CLI/test runner
验证，不搜索或使用 in-app browser。

## Acceptance

- [x] Dictionary 规则真实产生 Keep、Text-only、Font-only、Text+Font 四种 Decision。
- [ ] 字典默认字体和逐词条字体覆盖经过 wire、Runtime Kernel 与 Adapter 可见合同。
- [x] 同一软件保存两套 Workflow 后可显式原子切换，旧意图不会残留。
- [x] 一个 Workflow 可同时维持多个 Software；不同 Workflow 可并行控制互不重叠的软件。
- [x] 同一 Dictionary 可复用于多个 Workflow/Software；修改后 active 引用分别推进 generation。
- [x] 工作流、软件、词典均具备搜索、多选、分页、行级 CRUD 和必要批量操作。
- [x] 100+ 合成记录下表格、选择、筛选和分页仍可操作；不以普通下拉框承载完整集合。
- [x] 软件重启后 active Workflow 保留期望，刷新后恢复最后有效 Publication。
- [ ] Core/Tauri/Vue 没有软件品牌、程序路径 fixture、PID 或 GDI/GDI+ 产品分支。
- [ ] Premiere 本地字典、真实软件路径、截图、PID 和实机日志未进入暂存或提交。

## Verification

每个小纵切坚持 red → green → refactor；不要积累到最后统一跑：

```powershell
cargo test -p glyphshift-decision
cargo test -p glyphshift-runtime-contract
cargo test -p glyphshift-workflow
cargo test -p glyphshift-desktop-backend
cargo test -p glyphshift-desktop-runtime
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
npm --prefix apps/glyphshift-desktop run build
npm --prefix apps/glyphshift-desktop test
```

提交前必须执行 `git status --short`、检查 staged path，并扫描 staged diff 中的盘符路径、
用户目录、PID、截图和真实宿主名。只暂存本 Slice 的代码/文档，显式排除用户 Premiere 字典。

## Next

继续第六个 UI 纵切：让 Workflow Editor 为每个 Target 分别维护可排序 Dictionary 栈；不要在
UI 重写 Backend 的冲突、引用保护或组合规则。随后同步替换旧 `PRODUCT.md`、`DESIGN.md` 与
Surface Brief，再做完整实际 Tauri 窗口验收。
