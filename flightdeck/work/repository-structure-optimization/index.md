# 全仓代码结构梳理与优化

Status: Finished

## Goal

在不改变产品行为、Adapter ABI、Runtime Bundle 合同和安全失败语义的前提下，让代码库更容易定位、
修改与验证：拆解职责堆叠的超长文件，把测试放到合适的 seam，通过明确的 crate 家族与依赖方向降低
workspace crate 的导航成本，并避免产生新的浅模块或万能 `core`。

## Current

Adapter 实现二级目录纠正已完成。当前 23 个 implementation crate 位于
`implementations/{native,framework,accessibility,fallback}/<technology-crate>`；同一技术的 descriptor
与 Native/Worker companion 保持相邻。当前没有正式 DX/OpenGL/Vulkan Adapter，因此没有提前创建空
`rendering/`。二级目录只表达物理导航，Registry Technology/Capability、package/lib/target 名、ABI、
制品与依赖集合保持不变。

实时 Cargo metadata 为 62 个 workspace package、27 个 Adapter package，其中 4 个 platform、23 个
implementation。架构检查已同步覆盖此前漏记的 OCR Worker、进程授权模块及真实依赖，并把明确的
合成路径夹具与机器特定路径区分开；完整 package partition、依赖层级与隐私扫描通过。

全仓代码结构优化已完成。29 个非 Adapter package 的官方资料、实时依赖图和删除测试没有发现应合并的
package；Cargo package/lib/target 身份保留 `glyphshift-` 上下文，47 个 `crates/` package 全部迁入短家族
路径。当前入口为 `adapters/`、`core/`、`dictionary/`、`runtime/` 和 `product/`，例如
`glyphshift-protocol` 位于 `crates/runtime/protocol`；Target 家族使用可被 Git 跟踪的复数
`runtime/targets/`。54 个 package 统一 `publish = false`。

完整 workspace、全 targets Clippy、格式、架构、Desktop build、Playwright 60/60、metadata/lockfile、
239 个 Cargo path、92 个文档链接、行数和隐私门禁全部通过。工作树按要求保持未提交。

用户复核后明确要求的 Adapter 真实物理入口已经完成：4 个平台/契约 crate 位于
`crates/adapters/platform/`，21 个具体技术实现位于 `crates/adapters/implementations/`。实时 Cargo
metadata 仍为 54 个 workspace package、25 个 Adapter；package 名、ABI、代码与依赖合同保持。路径审计、
架构/格式/diff、完整 workspace tests/Clippy、桌面构建和 Playwright 60/60 全部通过；最终结构、链接与
隐私 Review 通过，工作树按要求保持未提交。

全仓结构优化目标已完成。236 个代码文件中没有文件达到 1200 行；主要 Rust/Product/Test/Frontend owner
均按真实变化原因形成 module，54 个 crate 已逐项删除测试并由完整家族/层级门禁约束。外部 API、ABI、
schema、Tauri 命令与用户行为保持，最终全仓验证和隐私审核通过，工作树按要求保持未提交。

前序 Adapter 覆盖、桌面加固和技术文档入口已在 checkpoint `bbebea9` 固化。超长文件职责图已经完成，
并确认首批真正超过 1200 行实现体的是桌面 Tauri 壳、桌面后端、桌面运行时、翻译与隔离 Worker Host；
Capture、Controller Windows 与 Capture Workspace 主要先处理内嵌测试。

Rust workspace 当前有 54 个 crate，其中 25 个属于 Adapter 家族。`glyphshift-domain`、Adapter SDK、
Adapter Registry、Native ABI、Translation 与 Capture 是高复用依赖；Target Runtime、Native Host、
Desktop Runtime 和 Desktop Shell 则编排大量内部依赖。现有架构测试对每个 crate 的直接依赖做精确
约束，因此不能先移动目录或合并 crate，再事后解释依赖。

桌面壳已经完成“Probe/Dictionary/Workflow/Software 纵向 module + 小型平台能力”重构。
`font_catalog` 隐藏缓存与系统扫描，五类根测试按 interface 拆分；根 `lib.rs` 从 4633 行降至 564 行，
本 crate 没有超过 1200 行的 Rust 文件。Desktop Shell 44/44、package Clippy、格式、架构、diff check
和组件/命令合同 Playwright 6/6 通过；45 个 Tauri 命令名与唯一公共入口 `run()` 保持不变。crate 家族
已同时按物理能力和依赖层级完成分类；`core` 只作为候选目录，Runtime、产品和测试支持保留各自 seam。
Adapter Platform 与具体 Adapter 已在 Stage 7 落入统一物理入口，同时保留各自部署 seam。

Translation 已按消费模型拆成不可变 Snapshot/Font Policy 与可编辑 Workspace 两个私有 module；根
`lib.rs` 从 1593 行降至 8 行。两段实现与 checkpoint 对应源码逐行一致，10 项 Workspace 合同和全部
13 个直接下游 package 回归通过，digest、generation、冲突和序列化语义未变。

Isolated Worker Host 已拆成 Worker 传输、Supervisor、Isolated Host 和 Hybrid Placement 四个生命周期
module；根从 1479 行降至 17 行。生产逻辑逐段审计一致，11 项本 crate 测试和两个直接下游 package
回归通过，重启预算、超时回收、generation 与部分 Placement 激活语义未变。

Capture 已按 Observation wire、非阻塞 Batch Producer、单写者 Catalog Sink 拆分，397 行内嵌测试也按
三个公开 seam 迁移；根从 1339 行降至 15 行。三段生产实现逐段一致，15 项本 crate 测试与 11 个直接
下游 package 回归通过，两个 schema、drop/gap 和双槽 checkpoint/恢复语义未变。

Capture Workspace 的 904 行深 Store 实现保持逐行不变，362 行内嵌测试按 join/lifecycle、query/export、
settings/clear 场景迁移；文件降至 907 行，5 项 Workspace 场景与 Capture 全部 15 项测试通过。

Controller Windows 已拆成 Executable/Elevation、Controller 状态机与 Platform facts，既有 `remote.rs`
继续隔离 unsafe 注入；根从 1284 行降至 17 行。三段实现审计一致，18 项确定性本 crate 测试和 Desktop
Shell 44 项下游回归通过，稳定 token、PID reuse、路径/hash、PE 与提权语义未变。

Session 已拆成 705 行部署 Contract 与 422 行有状态 Manager；根从 1122 行降至 7 行。两段实现和公开
surface 审计一致，18 项 Session 合同与 4 个直接下游 package 回归通过。1274 行外部合同测试留到测试
结构阶段按场景迁移。

Protocol 已拆成 618 行 Model 与其 492 行私有 Connection 子模块；根从 1107 行降至 5 行。实现逐行、
公开 surface 与字段私有性均未变，9 项 Protocol 合同和 5 个直接下游 package 回归通过。

Target Runtime 将 111 行 File/Batch Capture owner 提取后，803 行根继续集中拥有 RuntimeState、Native
ABI/Kernel 与原子激活路径；1 项内部和 1 项确定性公开合同通过，3 项授权 Native DLL 合同保持 ignored。
至此 Stage 3 的 Translation、Worker Host、Capture、Controller、Session、Protocol、Target Runtime
生产结构复核全部完成。

1757 行 Desktop Playwright 单文件已拆为一个共享 product model fixture 和五个功能 spec，最大 Probe
spec 506 行。原 44 个测试名称集合一致，44/44 功能测试及前端生产构建通过；全 testDir 仍发现 60 项。

1274 行 Session 合同测试已拆为 472 行共享 fake/fixture 根和六类状态机场景，最大 233 行；18 个测试名
集合一致，18/18 合同与测试结构门禁通过，生产代码未改。

1239 行 Windows test-support Host 已拆为 977 行 Rendering acceptance 与 233 行 UIA synthetic server；
crate 根和 Windows facade 只保留条件编译与稳定 re-export。公开 item 与 unsafe seam 审核一致，2 项本 crate
合同、三个直接依赖 package、Clippy、格式、架构和 diff 门禁通过。

1117 行 Desktop Runtime Windows 合同已拆为 184 行共享 fixture 与五类场景模块，最大 322 行。格式化前
按场景重新拼接的源码 hash 与 checkpoint 一致，12 个测试名及 ignored 理由精确保持；普通运行 12/12
ignored，未启动真实软件，Clippy、格式、架构和 diff 门禁通过。

前端状态复核已完成：`useWorkspace.ts` 由 800 行变为 51 行 facade 和四个 capability/state module，45 个
返回字段与 42 个函数实现审计一致；Capture 只提取独立滚动 DOM 生命周期，业务 owner 保持。中英文各
661 个 key 完全一致并保留整 catalog。生产构建、类型检查和 Playwright 60/60 通过。

crate 拓扑与删除测试已覆盖 Cargo metadata 的全部 54 个 package；每个小 crate 均以 ABI、进程/部署
制品、SDK 或多调用者 policy 通过删除测试，没有发现应合并的空壳 crate。前序“全部保留扁平”的物理
结论已由用户复核推翻：Stage 7 已补齐 `crates/adapters/` 真实入口，没有改变任何 crate seam。

架构检查已从 48 个补齐到 54 个精确 dependency contract，并增加两个 54-package 完整 partition 与
跨层 allow-list。真实 workspace 依赖和允许方向 self-test 通过；L0→Host、Host→具体 Adapter、Product→
Test 三类故障注入均被拒绝。Stage 5 的物理拓扑、删除测试和架构规则全部定稿。

## Next

None.

## Residual risks

- 显式授权的真实 Host/本机 DLL 合同仍保持 ignored；未读取本机软件证据。
- Vite 仍报告既有的大 JavaScript chunk warning；不影响本次结构/行为验证。
- 工作树按用户要求未提交。

## Progress

- 已完成 [Adapter 实现二级目录归档](slices/adapter-implementation-second-level-groups.md)：13 个 native、
  6 个 framework、2 个 accessibility、2 个 fallback crate 已归档；旧平铺路径为 0。
- Cargo metadata、全部相对 path 存在性、架构、格式与 diff 门禁通过；GDI、Qt、UIA、OCR 四个分组代表
  Adapter 合同共 29/29 通过，没有运行全仓测试。

- 用户要求继续审视非 Adapter crate 的合并、物理家族与 `glyphshift-` package 前缀；Work 已重开并新增
  Stage 8，先做官方资料与删除测试审计。
- 已完成[非 Adapter crate 边界与命名复核](slices/non-adapter-crate-boundaries-and-naming.md)：29 个 package
  均有实际 seam；保留 Cargo 身份前缀，不批量引入 alias，不合并 package。
- 已完成[crate 家族短目录迁移](slices/crate-family-short-paths.md)：47 个 crate 归入五个家族并使用短叶目录；
  metadata 身份/targets/dependencies 不变，54/54 package 禁止 registry 发布。
- 已完成[家族短目录迁移后全仓验证](slices/post-family-full-validation.md)：修正会命中 `**/target` 的单数
  目录后，完整 Rust、Clippy、Desktop build、Playwright 60/60、路径/链接/隐私门禁全部通过。

- 用户复核推翻了“Adapter 保持扁平”的物理拓扑决策；Work 已重新打开，新增 Stage 7，并以实时 Cargo
  metadata 确认 25 个正式 Adapter crate 与 3 个非 crate 空目录。
- 已完成[Adapter 家族物理归档](slices/physical-adapter-family.md)：4 个平台 crate 与 21 个实现 crate 归入
  `crates/adapters/`；Cargo/path、脚本、架构扫描和文档链接已更新，内容/依赖自审与全家族测试通过。
- 已完成[迁移后全仓验证](slices/post-adapter-full-validation.md)：完整 workspace/Clippy、Desktop build、
  Playwright 60/60、格式/架构/diff、236 文件行数门槛、66 个本地链接和隐私扫描全部通过。

- 已创建当前成果 checkpoint `bbebea9`，工作树从干净状态进入本 Work。
- 已用物理行数重新统计代码文件，避免 PowerShell 行统计忽略空行导致优先级偏差。
- 已盘点 workspace crate 数量、Adapter 家族数量、内部依赖出入度和现有架构依赖合同。
- 已确认行数只用于分流审查；拆分由接口深度、职责局部性、变化原因和测试 seam 决定。
- 已完成首批超长文件职责图与删除测试，选择桌面壳作为第一实施对象。
- 已提取 `font_catalog` 深 module 并迁移其两项测试；根壳只保留加载与显式刷新调用。
- 已完成 Probe、Dictionary、Workflow 与 Software 纵向 module，并把快速捕获平台逻辑归入 Software。
- 已把根内嵌测试拆为共享 fixture 与五个 feature 场景文件，没有制造新的超长测试文件。
- 已完成桌面壳 Standards/Spec 自审和全部切片门禁；本阶段未提交。
- 已定义覆盖 54 个 workspace package 的[物理家族与依赖方向](references/crate-families-and-dependency-directions.md)，
  明确禁止反向依赖和万能 `core/common` crate。
- 已完成[Desktop Backend 内部 module](slices/desktop-backend-internal-modules.md)：根从 2851 行降至
  209 行，最大 feature module 938 行；公共 API 和持久 schema 审计一致，全部切片门禁通过。
- 已完成[Desktop Runtime 内部 module](slices/desktop-runtime-internal-modules.md)：根从 2781 行降至
  82 行，Bundle/Pool/Target 与确定性测试分别归位；公开 API、Bundle v2 和失败语义保持。
- 已完成[Translation 内部 module](slices/translation-internal-modules.md)：不可变运行输入与可编辑
  Workspace 分离，根降至 8 行；实现逐行搬迁一致，直接下游回归和结构门禁通过。
- 已完成[Isolated Worker Host 内部 module](slices/isolated-worker-host-internal-modules.md)：Worker、
  Supervisor、Isolated Host 与 Hybrid Placement 分离；协议、重启和失败隔离语义保持。
- 已完成[Capture 内部 module 与测试 seam](slices/capture-internal-modules.md)：wire、热路径 producer、
  checkpoint sink 与 397 行测试分别归位；公开 API/schema 和恢复语义保持。
- 已完成[Capture Workspace 测试 seam](slices/capture-workspace-tests.md)：生产 Store 逐行不变，5 项
  内嵌场景按 join、query 和 settings seam 迁移。
- 已完成[Controller Windows 内部 module](slices/controller-windows-internal-modules.md)：Executable、
  Controller、Platform 与测试分别归位，远程注入 seam 和公开 Controller 行为保持。
- 已完成[Session 内部 module](slices/session-internal-modules.md)：部署 Contract 与有状态 Manager 分离，
  18 项合同和直接下游回归通过。
- 已完成[Protocol 内部 module](slices/protocol-internal-modules.md)：wire/model 单向拥有 Connection，
  协议字段私有性、公开 API 与失败状态保持。
- 已完成[Target Runtime Capture module](slices/target-runtime-capture-module.md)：Capture owner 独立，
  Native ABI/RuntimeState 原子路径保持集中，核心生产结构阶段完成。
- 已完成[Desktop Playwright spec 拆分](slices/desktop-playwright-specs.md)：44 项用户可见合同按五个功能面
  归位，共享 fixture 单一，完整功能套件与生产构建通过。
- 已完成[Session 合同测试结构](slices/session-contract-tests.md)：共享 fake/fixture 与六类状态机场景
  分离，18 项公开合同不变。
- 已完成[Windows test-support Host module](slices/windows-test-host-modules.md)：Rendering 与 UIA server
  生命周期分离，公开测试宿主 API、unsafe 资源边界和隐私协议保持。
- 已完成[Desktop Runtime Windows 合同结构](slices/desktop-runtime-windows-contracts.md)：12 项授权/合成
  Runtime 合同按公开 interface 归位，ignored 条件与本机输入协议保持。
- 已完成[前端状态与视图 seam](slices/frontend-state-and-view-seams.md)：Workspace capability、Capture DOM
  生命周期与 locale catalog 的变化原因已定稿，完整前端回归通过。
- 已完成[crate 拓扑与删除测试](slices/crate-topology-and-deletion-audit.md)：54 个 package 逐项审计和删除
  测试仍成立；其中“Adapter 保持扁平”的判断已由 Stage 7 的用户复核推翻。
- 已完成[crate 家族与依赖方向门禁](slices/crate-architecture-rules.md)：54/54 精确合同、两个完整 partition
  和跨层禁止方向已由同一架构脚本执行。
- 已完成[全仓验证、隐私与最终 Review](slices/full-repository-validation.md)：完整 workspace tests/Clippy、
  Desktop build、Playwright 60/60、架构/格式/diff 与隐私扫描通过；完成态 0 个文件达到 1200 行。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [仓库结构基线](references/repository-structure-baseline.md)
- [Crate 家族与依赖方向](references/crate-families-and-dependency-directions.md)
- [全仓验证与最终 Review](slices/full-repository-validation.md)
- [Adapter 家族物理归档](slices/physical-adapter-family.md)
- [Adapter 迁移后全仓验证](slices/post-adapter-full-validation.md)
- [Adapter 实现二级目录归档](slices/adapter-implementation-second-level-groups.md)
- [Rust/Cargo 命名与 workspace 最佳实践](references/rust-crate-naming-and-workspace-best-practices.md)
- [非 Adapter crate 边界与命名审计](references/non-adapter-crate-boundary-audit.md)
- [crate 家族短目录迁移](slices/crate-family-short-paths.md)
- [家族短目录迁移后全仓验证](slices/post-family-full-validation.md)
- [Adapter 按文字技术划分的既有结论](../../knowledge/research/market-landscape.md)
- [产品契约](../../../PRODUCT.md)
- [领域语言](../../../CONTEXT.md)
- [桌面设计契约](../../../DESIGN.md)
