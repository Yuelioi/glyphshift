# 全仓代码结构优化上下文

## 稳定事实

- Glyphshift 是由桌面产品、Dictionary、Capture、Session、Controller、Target Runtime、Adapter 平台和
  多种文字技术 Adapter 组成的 Rust workspace；前端位于桌面 App 内。
- Adapter 是文字读取或绘制技术在既有 seam 上的具体实现。UIA、GDI、DirectWrite、Qt、GTK、raylib
  等按技术划分；软件身份和配置不应反向变成每软件一个 Adapter。
- Native、Isolated Worker 与桌面进程属于真实部署 seam；跨进程协议、Native ABI、制品信任、授权、
  Placement 和 fail-open 语义不能为了减少 crate 数量而被折叠。
- 现有 `architecture-tests/check.ps1` 精确约束 workspace package 的直接依赖，并检查关键源码禁用项。
- 机器专属路径、进程值、日志和截图只属于忽略的 `local-test/`，不得进入结构文档或提交。

## 设计语言

- **Module**：具有一个 interface 和内部 implementation 的任意尺度代码结构。
- **Interface**：调用者必须知道的完整使用合同，包括类型、顺序、不变量、错误与性能特征。
- **Seam**：不修改调用位置即可替换行为的位置；只有真实存在至少两种 Adapter 时才提升为外部 seam。
- **Adapter**：在 seam 上满足 interface 的具体实现。
- **Depth**：调用者通过较小 interface 获得较多行为；拆文件不能以增加浅转发层为代价。
- **Locality**：同一变化原因、错误和验证集中在一个 module 内，而不是跨多个调用者同步修改。

## 决策与约束

- 1200 行是“必须评审”的分流信号，不是自动拆分规则；800 行以上进入观察清单。
- 先拆同一 crate 内的 implementation 与内嵌测试，再判断是否需要新的 crate。纯目录移动不算完成架构
  优化。
- 保持外部 interface 稳定，优先建立私有内部 module；不要为了测试把内部 seam 暴露为公共 interface。
- 新测试在被重构 module 的 interface 上验证可观察行为；新 interface 测试覆盖后，删除只验证旧浅层
  实现细节的重复测试。
- `core` 只能是目录和依赖方向概念，不能成为容纳所有共享代码的巨型 crate。
- 物理家族与依赖层级是两套分类：目录按能力导航，架构门禁按 L0 基础模型/SDK、L1 策略、L2 部署
  合同、Adapter 侧车、L3 Host/Kernel、L4 产品编排、L5 Shell 约束方向。
- 47 个 `crates/` package 使用 `adapters/`、`core/`、`dictionary/`、`runtime/` 与 `product/` 家族入口；
  叶目录使用短能力名，例如 `crates/runtime/protocol`。Target 子家族必须用复数 `runtime/targets/`，避免
  命中 Cargo/仓库的 `**/target` 构建输出忽略规则。
- Cargo package/lib/target 身份保留 `glyphshift-` 上下文；目录名与 package 名解耦。当前全部 54 个
  workspace package 继承 `publish = false`，未来公开 SDK 必须显式建立 registry/versioning 合同。
- 物理归档不改变 package 名、ABI、部署 seam 或依赖层；是否合并 crate 仍须逐项删除测试，不由目录家族
  机械外推。
- Adapter 逻辑、Native/Worker Placement 和宿主编排可以物理分组，但保留其真实进程与 ABI seam。
- Desktop Tauri 壳采用纵向功能 module 加小型平台 module；根保留 `run()` 与 Builder 组合，不采用把全部
  DTO、应用操作和命令各自堆成横向大文件的结构。
- Desktop Tauri 壳的共享应用状态和 Runtime seam 留在根组合层；功能 module 通过同一个
  `DesktopApplication` 实现用例，跨功能协作仅使用 `pub(super)`，不拆成互相转发的状态 facade。
- Desktop Backend 根是公开 re-export 与加载组合层；Dictionary、Workflow、Software/Extension 各自
  拥有 DTO、artifact、用例和验证，Snapshot 单向读取 feature view，Storage 不拥有业务 schema。
- Desktop Runtime 的 Bundle、Pool 与 Target Runtime 是三个独立变化原因：Bundle 认证 bytes，Target
  管理一个已验证连接的 Session 生命周期，Pool 管理多软件所有权；Native/Worker Host 继续是外部 seam。
- 每个重构切片必须无产品行为变化，并通过受影响 crate 测试、架构检查、全 workspace Clippy；涉及桌面
  界面或命令合同的切片还要通过仓库 Playwright。

## 成功标准

- 首批超长实现体按稳定职责形成内部 module，顶层 `lib.rs` 或 App 壳只保留清晰 interface 与组合入口。
- 测试文件按被测 interface 分组，单个端到端规格不再承担全部桌面功能。
- crate 家族、命名和依赖方向能从 package 名、`crates/README.md` 与架构测试共同读出，新贡献者无需逐个
  打开 54 个 package 猜测角色。
- 结构调整后外部 ABI、序列化字段、持久文件、Tauri 命令名、错误码和 Runtime Bundle 结果保持兼容。
