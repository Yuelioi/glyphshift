# Dictionary Catalog 与可信安装

Status: Complete

## Deliverable

建立在线 Dictionary 分发的 P0 深 Module：Catalog query 与 presentation locale 回退、release 与
Artifact Statement 合同、限量 fetch 与 digest/trust 校验、Dictionary package 身份复核、内容寻址
制品存储、Installation Record、崩溃恢复和 Desktop installation view。没有真实 endpoint 或 key
policy 时只交付 production-ready seam 与确定性 adapters，不显示伪在线状态。

## Contracts

- [技术设计](../references/dictionary-catalog-and-installation-design.md)是 schema、Module interface、
  签名声明、存储与恢复顺序的权威。
- Dictionary Payload、Catalog Release、Artifact Presentation、Artifact Statement、Installation
  和 active working copy 是独立事实。
- `DictionaryDistribution` 对调用方只暴露 query/install/installations；下载、验证和落盘编排不泄漏。
- Runtime、Decision、Workflow resolve 与 Native Adapter 不依赖 Catalog、HTTP、signature 或安装状态。
- 本地编辑不会伪装成仍被签名覆盖；已安装 Dictionary 改动后派生状态是 `modified`。
- Dictionary 详情只展示语言方向、版本、词条数、修订以及原文/译文；metadata 进入设置 Modal。
  Dictionary `/2` 不再包含 Location/Context/keep。

## Steps

1. **提取 DictionaryPackage Module（完成）**
   - 把 Dictionary `/2` artifact、codec、validation 与 view conversion 从 Desktop Backend 移出。
   - 用现有 Desktop contract 证明文件与产品行为不变，删除 Backend 内重复 validator。
2. **冻结 Distribution interface（完成）**
   - 定义 Catalog、Release、Presentation、Artifact Statement、Signature Envelope 和 Install types。
   - 提供 in-memory Distribution 与 Trust adapters，覆盖 locale fallback 和所有拒绝路径。
3. **实现可信安装存储（完成）**
   - 限量读取、SHA-256、trust verifier、payload identity 比较。
   - 内容寻址 artifact、Installation Record、pending transaction 与 reopen recovery。
4. **接入 Desktop/Tauri（完成）**
   - Dictionary Summary 增加 installation view；本地 CRUD 与 Workflow resolve 保持原义。
   - 增加 query/install commands 与结构化 CommandError 映射。
5. **交付 Catalog UI（完成）**
   - Dictionary 页增加本地/Catalog 模式、离线与空态、安装和 modified replacement 确认。
   - presentation 使用 effective UI locale，payload metadata 和内容语言不跟随切换。
6. **验证与收尾（完成）**
   - Rust contracts/workspace、fmt、Clippy、架构扫描、Vue build、Playwright 双语言/窄宽视口。
   - 真实 endpoint/key policy 不存在时保留 adapters 配置入口并明确记录剩余风险。

## Acceptance

- 一个调用即可完成 release 获取、验证、安装和 installation view 更新；调用方不编排内部步骤。
- size、digest、signature、publisher 或 payload identity 任一不一致时 active Dictionary 不改变。
- 安装中断后的 reopen 只保留完整旧版或完整新版，不产生伪 installed。
- modified/unmanaged 本地资产不会被更新静默覆盖。
- Catalog Presentation 与 UI locale 正确回退，且不进入 Dictionary payload 或 Runtime Publication。
- Catalog/网络失败不影响本地 Library、Workflow resolve 或已运行目标。

## Current

领域语言、authority matrix、深 Module interface、Catalog/Artifact/Installation schema、签名声明、
内容寻址存储与崩溃恢复顺序已冻结。`glyphshift-dictionary-package` 已从 Desktop Backend 提取，
独立拥有 Dictionary `/2` codec、contract validation、view 与 entry mutation；Desktop Backend 保留
产品 DTO facade，并通过 adapter 投影 package view，原文件格式和产品行为不变。

`glyphshift-dictionary-distribution` 已实现 query/install/installations 外部接口，以及 Catalog、Trust、
Install Store、Clock 四个内部 ports。强类型 release/presentation/statement/signature/installation 合同、
HTTPS 与大小上限、固定验证顺序、镜像降级、常量时间 digest 比较、幂等安装和派生 installation state
均已落地；in-memory adapters 不冒充 production HTTP 或 key policy。

文件型 `DictionaryInstallStore` 已按冻结顺序落地：已验证 payload 进入 SHA-256 内容寻址只写制品，
active Dictionary、`glyphshift.dictionary-installation/1` 与 pending transaction 分离保存；同目录临时
文件原子发布 active copy 和 record。重开会同时复核 transaction、active、artifact、digest、大小、
Dictionary ID 与 revision：active 尚未切换时保留完整旧版，已经切换时补全完整新版。四个安装中断
点、modified/missing/unmanaged 派生状态及公开 Distribution—文件存储—重开组合均有 tempdir 合同。

Desktop Backend 已把 installation state、已安装 release、已验证 publisher 与可更新 release 投影到
Dictionary Summary；本地 CRUD 保持 unmanaged/modified 语义。Desktop API v14 提供 Catalog query
和 release install 命令，Distribution 错误映射为稳定 `CommandError/1`，普通 DTO 不泄漏 URL、digest、
signature、key ID 或存储路径。生产组合显式使用离线 Catalog/Trust adapters，在真实 endpoint 与 key
policy 缺失时不制造伪在线能力。

Dictionary 页面在同一一级导航内提供本地词典与在线目录两个紧凑模式。本地表展示 provenance，
目录表使用后端搜索和 cursor 分页；覆盖 modified/unmanaged 词典前必须确认，已安装版本不可重复
安装，presentation 跟随 effective UI locale。离线只影响目录模式，本地 Library、Workflow 与 Runtime
不受影响。全 Rust workspace、Clippy、fmt、架构扫描、Desktop 35 项 Playwright 与生产构建通过；
6 项要求明确本机授权宿主或预构建 Native DLL 的测试保持 ignored。构建只保留既有的单入口 chunk
体积提示，真实在线 endpoint/key policy 仍是后续部署配置。

现有 Dictionary 详情已先完成用户反馈驱动的收口：移除内联 metadata 大表单，设置与词条编辑
改为单列 Modal。后续可观测性复核证明 Location/Context 没有 Runtime 事实来源，纯 Dictionary
Slice 已接管其删除；Catalog 安装工作在该 Slice 完成后继续。

## Next

该 Slice 已完成。回到主工作页推进生产 Extension/Runtime Bundle、词典导入发布流程与授权实机
验收；接入真实 Catalog 前必须先提供 endpoint、publisher key policy 与部署期配置来源。
