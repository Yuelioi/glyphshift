# Glyphshift 通用产品交付

Status: Open

## Goal

交付不依赖具体软件品牌的 Windows 运行时界面翻译产品：工作流组合软件与独立词典，动态
Runtime Bundle 提供可验证的文字和字体写回能力，桌面界面保持高密度、可扩展和技术上诚实。

## Current

通用实现已从旧组织仓库的第二代 workspace 提升为独立仓库根目录并完成验证。33 个 Cargo
package、Tauri/Vue 桌面、测试支持与架构检查均在当前仓库内闭合，不再存在根级旧 workspace、
旧 GUI、旧脚本或兼容依赖。Dictionary、Platform/Technology、Adapter Catalog、内联 Font Policy
与 Workflow Target 已完成正交拆分，并贯通 Backend、Runtime reconcile、Target Runtime 合同
和桌面 UI。标题栏提供 Help/Settings 入口，Help 从 Runtime Bundle 展示当前 Adapter Catalog，
Settings 不再保存在线翻译服务地址；桌面后端不可用按产品错误处理，不显示伪连接状态。

`AppSettings/1`、Vue I18n 与 `CommandError/1` 已交付：应用 mount 前解析真实语言和主题设置，
完整核心 UI 提供 `zh-CN`/`en-US`，Settings 只展示可立即生效并持久化的界面语言与主题，Tauri
command 跨 seam 返回稳定 code + typed args。UI locale、字典内容语言和动态 Artifact
presentation locale 保持正交。生产构建、Playwright 20 项、Rust workspace test、fmt、Clippy
及真实 Tauri 设置跨重启验证均通过。

在线 Dictionary 分发[技术设计](references/dictionary-catalog-and-installation-design.md)已冻结：
Catalog Release、Artifact Presentation、签名 Statement、Installation Record 与 active working
copy 分权；深 `DictionaryDistribution` Module 隐藏查询、下载、验证、安装和崩溃恢复，Runtime
继续完全离线。首个实现边界已落地：`glyphshift-dictionary-package` 独立拥有 Dictionary `/2`
codec、校验与 mutation，Desktop Backend 只保留产品 DTO facade；
`glyphshift-dictionary-distribution` 已提供 query/install/installations 深接口、强类型 Catalog 与签名
合同，以及不冒充生产网络/密钥策略的 ports。文件型 Install Store 已闭合内容寻址制品、Installation
Record、pending transaction、原子 active copy 与重开恢复；四个中断点只会收敛到完整旧版或完整
新版。Desktop API v16 和 Backend Summary 已接入安装状态，Dictionary 页以本地/在线目录双模式
展示 provenance、后端搜索、metadata tag 精确筛选、cursor 分页、离线空态与安全覆盖确认；当前
模式使用明确主色选中态。生产仍显式离线，等待真实 endpoint、publisher key policy 与部署配置。

标准 Dictionary `/2` JSON 文件交换已完成：本地词典页可导入当前 schema，并逐行导出可重新导入
的发布文件。重复 ID、非法 JSON 和旧 schema 明确拒绝；流程不包含账号、上传、审批、证书管理或
额外包格式。授权继续由用户显式选择程序并启动工作流/探针表达。

Dictionary 详情已按真实使用反馈完成信息降噪，Dictionary `/2` 也已收敛为纯
`source + translation`：Location、Context、keep 与逐词条字体均已从 payload、DTO、编辑器和
Workflow compile 删除。Runtime 内部路由不回流到 Dictionary；当前 Adapter 无法识别主界面、
弹窗或面板，因此没有伪造 Region/Location 配置。当前编辑器进一步收敛为一份显式草稿：metadata
设置、已有词条行内修改、末行新增和删除共享未保存状态，离开编辑器或关闭窗口前统一保护。
生产构建与完整 Playwright 39 项通过，960×640 紧凑视口已完成视觉复核。

Workflow Editor 已从固定左右主从栏和连续长表单，改为“基础配置 / 软件与拦截 / 翻译词典 /
字体策略”四个任务 Tab。每页内部保持单列；软件与 Adapter 是一个连续阶段，Dictionary 与 Font
各自使用单一可搜索列表并行内调整优先级。编辑 Modal 使用稳定工作区高度，紧凑分段导航固定在
顶部，只有当前 Tab 内容滚动；大集合目录继续使用有界局部滚动。合成 10 个软件、100 份词典的
回归证明搜索、切换、优先级与 Target 隔离仍稳定。

Font Policy 已收紧最后一处高占宽交互：coverage 已从两个满宽按钮改为“应用范围”选择器；本机
字体目录改为跨启动持久缓存，普通启动不再重复扫描，目录标题显示缓存数量并提供显式刷新动作。

Runtime Bundle 现提供 `ExtTextOutW`、`TextOutW`、`DrawTextW/DrawTextExW` 与 `GdipDrawString`
四个独立 Adapter，中文 presentation 在 Windows PowerShell 5.1 下按 UTF-8 稳定读取。可恢复
Probe Run 已完成：每个任务绑定一个软件、一个 Dictionary 与一组 Adapter；Dictionary 是唯一翻译
内容资产，内部 Observation Index sidecar 只保存技术证据。管理页与详情明确分离，详情以一张联合
表支持搜索、多选、行内编辑、批量处理、分页、导出、暂停/继续和 Live Preview。5,000 条基准下由
Rust 分页，Vue 只渲染当前页；1 秒双 revision 轮询只在变化时重取页面。

LunaTranslator 的官方文档和固定源码版本调研已完成。它验证了持续 Hook 作业需要候选文本流的
采样、选择、持久化和重匹配，也证明该身份属于运行流而不是 Dictionary `location`。当前产品交付
已完成无需扩展 Runtime 协议的 Probe Adapter 筛选与暂停/重启/恢复验收；筛选状态按 Run 恢复且
只影响后端分页视图，Dictionary、Observation Index 和完整导出保持不变。Observation Stream
Identity / Binding、Transform Profile、OCR 和 Overlay 分别保留在运行时能力升级 Roadmap。

生产 Runtime Bundle `/2` 与桌面交付候选已闭合：共用构建器生成固定第一方 authority、逐文件
SHA-256 且不含测试宿主的 Release 文件集；本地 Tauri override 已将其嵌入 unsigned NSIS
candidate。未安装 Release 桌面 smoke 通过，授权 AE 可见验收证明 `File` 可翻译为 `文件` 并在
停止后恢复。安装器尚未执行，代码签名与公共发布也不在当前实现中。

工作流 Runtime 错误反馈已完成首轮 UI 修复：后端原有结构化错误不再被表格压缩成“需要处理”。
`runtime.target_not_found` 明确显示为“软件未启动”并使用警告色；权限、组件加载/兼容、超时等错误
分别显示具体状态。点击状态可展开逐软件完整原因与恢复动作，工作流的持久启用期望保持不变。
添加软件仍只校验可访问、绝对路径的 `.exe`；Hook 覆盖属于软件运行后的 Adapter 证据，不做静态猜测。

后续实机验收发现两项底层缺陷：桌面重启后，目标内仍 active 的 Runtime 被再次激活并以
`AlreadyActive(10)` 拒绝，但后端误报“组件不兼容”；多进程目标存在多个同路径实例时，Desktop
固定选 inventory 第一项，可能选中无窗口 utility 并导致 Runtime 加载失败。当前架构已保留原始
Runtime status，但 Tauri 错误映射将其丢弃；位数不匹配与 Bundle 整体损坏已经排除。

旧 AE 与 Premiere 词典仅作为未接入 Runtime 的源数据保存在 `archive/dictionary-sources/`；
架构检查禁止生产代码引用归档路径或旧 schema。原仓库保持不变，继续作为只读回退来源。

最终模型已确认：Dictionary、Platform、Technology 和 Adapter 独立，Workflow Target 是唯一组合
根并直接拥有可选 Font Policy；在线 Dictionary 采用纯 payload 与外部 Artifact Descriptor。
当前未发布，不实现旧 schema 兼容。

## Next

- 等待用户一并验收[Dictionary 行内草稿编辑](slices/dictionary-inline-draft-editing.md)和
  [字体目录缓存与紧凑策略控件](slices/font-catalog-cache-and-compact-policy.md)，随后回到
  [工作流 Runtime 错误反馈](slices/workflow-runtime-error-feedback.md)的 `AlreadyActive` 幂等接管。

## Progress

- [独立仓库迁移](slices/repository-migration.md)完成：根结构、脚本、标识和文档均已去除旧代际边界。
- 根级测试编排显式预构建原生 DLL/EXE；workspace test、Clippy、fmt 和架构检查通过。
- 可组合资产与 Adapter Catalog Slice 完成；Dictionary `/2` 与 Target Runtime Deployment `/2`
  已落地。
- 内联 Workflow Font Policy 完成；桌面 API v12、Workflow `/3`、词典命中/Hook 全量 coverage、
  顶栏去字体页与用户 Location 清理已闭合，27 项 Playwright 和全 Rust workspace 通过。
- Help/Settings Slice 完成；Adapter 版本与 Feature presentation 贯通 Runtime、Tauri 与 Vue，
  标题栏去除连接状态，设置页去除在线翻译器配置。
- AppSettings 与桌面多语言 Slice 完成；语言/主题持久化、完整中英文 UI、结构化桌面错误及真实
  Tauri 跨重启验证闭合，Playwright 20 项通过。
- 在线 Dictionary Catalog 与可信安装设计完成；固定 presentation fallback、签名声明、内容寻址
  provenance、modified 状态与崩溃恢复边界。
- `glyphshift-dictionary-package` 已从 Desktop Backend 提取；Dictionary `/2` codec、校验、view 与
  mutation 收归独立 Module，仓库测试、fmt 与 Clippy 通过。
- `glyphshift-dictionary-distribution` 已交付深接口与内存 adapters；locale fallback、镜像降级、
  size/digest/signature/publisher/payload identity 拒绝、幂等安装和四类 installation state 有合同覆盖。
- 文件型 Dictionary Install Store 已交付；SHA-256 内容寻址制品、Installation Record、pending
  transaction、原子 active copy 与四个中断点重开恢复有 tempdir 合同覆盖，完整仓库验证通过。
- Dictionary Catalog 与可信安装 Slice 完成；当前 Desktop API v15、installation summary、同页
  Library/Catalog 模式、metadata tag 搜索与精确筛选、provenance、cursor 分页、离线空态和
  modified/unmanaged 覆盖确认已闭合；模式选中态完成视觉复核。全 Rust workspace、36 项
  Playwright、生产构建、Clippy、fmt 与架构检查通过。
- 简化词典文件交换完成；Desktop API v15 支持标准 Dictionary `/2` JSON 导入与逐行导出，非法
  格式和重复 ID 明确拒绝。没有引入发布中心、账号、审批、额外包格式或授权存储；36 项
  Playwright 与完整仓库验证通过。
- Dictionary 详情完成渐进披露重构；设置与规则 Modal 单列化，矮窗口可滚动，Playwright 22 项及
  1160×527/960×640/1440×900 视觉检查通过。
- Dictionary 编辑器完成单一草稿收敛；设置应用与表格修改立即标记未保存，现有词条直接编辑，
  末行常驻新增入口，返回/导航/关闭统一保护。生产构建与 39 项 Playwright 通过。
- Font Policy coverage 改为紧凑“应用范围”选择器；Desktop API v16 增加显式字体刷新命令，系统
  字体目录跨启动持久缓存。Shell/Backend 38 项 Rust 测试、Clippy、fmt、生产构建和 40 项
  Playwright 通过，1180×760 与 1440×900 视觉复核完成。
- 完成 Location/Context 可观测性复核：当前 GDI/GDI+ 回调没有区域信号，同一 Hook 无法区分主
  界面与弹窗；决定从 Dictionary 删除伪区域字段，未来只通过独立 Region Binding 使用真实信号。
- 纯 Dictionary、常用 Adapter 与捕获探针 Slice 完成；本机 2285 条词条升级，四个 Adapter Bundle、
  有界 Capture Catalog、Dictionary Draft 与探针 UI 已闭合，Playwright 24 项通过。
- 可恢复 Probe Workspace 完成；多工作区、双槽 checkpoint、暂停/继续、Preview、
  后端分页/批量/导出和管理表 UI 已闭合，Playwright 26 项及全 Rust workspace 验证通过。
- Probe 长表完成视口约束复核；表格独立滚动、可见可拖动滚动条、固定底部分页与
  Catalog 行内 Draft 译文编辑已闭合，1,180×760 的 5,000 条回归及全部 26 项 Playwright 通过。
- Dictionary-owned Probe Run 完成；旧 Draft/Catalog 双视图和一次性捕获命令已移除，绑定既有或
  新建空 Dictionary、联合查询、直接编辑、暂停恢复、导出及双 revision 刷新已闭合。管理表空态
  连续铺满且末行保留边界；Desktop build、27 项 Playwright、全 Rust workspace、Clippy、fmt 与
  架构检查通过。
- Workflow Editor 四 Tab 化完成：基础信息、软件与拦截、翻译词典、字体策略按任务隔离；各页删除
  左右主从栏和重复资产列表，以 Target 选择器保持多软件配置上下文；Modal 高度和顶部导航保持
  稳定，切换 Tab 不再引起窗口跳动或让导航共用内容滚动条。10 软件/100 词典合成边界、生产构建、
  设计扫描与 30 项 Playwright 全部通过。
- 桌面生产构建与 GUI 回归覆盖双词典排序、内联字体候选排序与 coverage、多 Target 暂存隔离、
  完整 metadata 保存及 960×640/1440×900 稳定帧。
- 完成同类产品、现有 Adapter Registry 与 macOS 拦截边界的一手资料调研。
- 完成字典、字体、在线 metadata 和插件 Catalog 的主流实践终审；技术方案与测试 seam 已固定。
- 完成 PowerToys、VS Code、Docker Desktop、OBS、Zotero 设置作用域与 Vue/Tauri i18n 一手资料
  调研，否决跨 IPC 直接传 vue-i18n key，推荐语义错误码与前端映射。
- 完成 LunaTranslator 产品、Hook 文本流、处理管线、字体/覆盖、输入源、恢复与多进程的一手资料
  调研，并把无需协议扩展的 Probe 闭环与后续 Runtime 升级能力分开路由。
- Probe Adapter 筛选与持续作业验收完成：多选条件贯通 Rust 分页与 Tauri/Vue，查询/筛选/分页状态
  按 Run 恢复；暂停编辑、完整导出、运行/暂停重启恢复和 5,000 条边界已有合同与 Playwright 覆盖。
- 生产 Runtime Bundle 完成：`/2` 清单使用固定第一方 authority 和逐文件 SHA-256，Release 精确包含
  Controller、Target Runtime 与四个 Adapter，不包含测试宿主；共用构建器同时服务开发桌面。
- 可分发桌面与可见验收完成：生成并检查 unsigned NSIS candidate，未安装 Release 桌面 smoke
  通过；授权 AE 中 `File` → `文件` → `File` 的启用/恢复可见链路闭合，36 项 Playwright 与完整
  Rust 门禁通过。安装器未执行。
- 运行时诊断与字体安全完成：授权 AE 同时覆盖 GDI/GDI+ 字体写回，修复 `ETO_GLYPH_INDEX` 在换
  字体时复用旧 glyph ID 导致的乱码；Release Bundle 真实宿主合同和停止恢复通过，安装器仍未执行。
- 工作流 Runtime 错误反馈首轮完成：删除“需要处理”汇总并提供逐软件详情；实机随后确认
  `AlreadyActive` 被误报为组件不兼容，且多进程目标固定取第一项会选错实例，底层修复待完成。

## References

- [稳定约束](context.md)
- [实际架构](references/architecture.md)
- [Hook 与 Adapter 调研](references/hook-translation-market-and-adapters.md)
- [Capability Adapter 与跨平台拦截调研](references/adapter-registry-and-cross-platform-interception-research.md)
- [可组合模型主流实践复核](references/composable-dictionary-font-and-adapter-model-review.md)
- [可组合资产技术方案](references/composable-assets-technical-design.md)
- [工作流配置调研](references/workflow-configuration-research.md)
- [全局设置与桌面端多语言调研](references/settings-and-desktop-i18n-research.md)
- [纯字典、常用 Adapter 与捕获探针](slices/dictionary-hooks-and-probe.md)
- [可恢复探针工作区与实时草稿](slices/capture-workspaces-and-live-drafts.md)
- [Dictionary-owned Probe Run](slices/dictionary-owned-probe-runs.md)
- [Workflow Editor 四 Tab 任务流](slices/workflow-editor-vertical-flow.md)
- [LunaTranslator 产品与运行时调研](references/lunatranslator-product-runtime-research.md)
- [Probe Adapter 筛选与持续作业验收](slices/probe-adapter-filter-and-recovery-acceptance.md)
- [工作流 Runtime 错误反馈](slices/workflow-runtime-error-feedback.md)
