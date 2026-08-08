# 可组合资产与 Adapter Catalog

Status: Complete

## Outcome

把 Dictionary、Font Profile、Adapter Plan、Platform/Technology Catalog 与 Workflow Target
拆为可独立管理和复用的产品事实；直接替换未发布的旧持久化 schema，并让 Desktop/Runtime
只通过编译后的 Target Intent 协作。

技术依据见[技术方案](../references/composable-assets-technical-design.md)和
[主流实践复核](../references/composable-dictionary-font-and-adapter-model-review.md)。

## Confirmed seams

- `glyphshift-workflow::resolve` 是组合规则唯一测试和调用 seam。
- `DesktopBackend` 公共 CRUD/activation 是持久产品 seam。
- Tauri Commands/View 是 Desktop Shell seam。
- Playwright 用户操作是 Vue 产品 seam。
- 未来在线 Dictionary 使用新的远端 Catalog Port；当前不创建网络实现或假服务端。

## Delivery

### 1. Domain、Adapter Descriptor 与 Catalog

- [x] Target Facts 暴露 Platform；Adapter Descriptor/Native ABI 声明 Platform 与 arm64。
- [x] Registry 在 Architecture 前检查 Platform，并保留明确拒绝原因。
- [x] Runtime Bundle presentation 提供 name、summary、technology、technical target 和
  `configuration: none`；Catalog View 不再叫 Hook Type。
- [x] GDI/GDI+ 的 UI 名称分别为 `ExtTextOutW` 与 `GdipDrawString`，Platform/Technology 独立展示。

### 2. Pure composition

- [x] Dictionary 只保存文字规则，不保存 Adapter 或字体行为。
- [x] 新增有序 Font Profile candidates 与按 Target Location 的 Font Binding。
- [x] Workflow Target 保存 parallel Adapter Plan、Dictionary 栈和 Font Bindings。
- [x] `resolve` 从 Composition Environment 验证 Adapter/Font 可用性并推导 Requirements/Features。
- [x] 纯字体 Target、纯翻译 Target、多 Adapter、fallback、重叠 Location 与无能力拒绝有合同覆盖。

### 3. Desktop persistence

- [x] `glyphshift.dictionary/2` 使用 metadata 区域，区分 schema、release version 与 local revision。
- [x] 新增 `glyphshift.font-profile/1` 的 CRUD、CAS、批量删除和引用保护。
- [x] `glyphshift.workflow/2` 保存逐 Target 的三个独立绑定。
- [x] 旧 schema 明确拒绝；没有兼容读取、迁移或双写分支。
- [x] active Dictionary/Font Profile/Workflow 更新先完整编译，再原子保存并推进 Runtime generation。

### 4. Tauri and Vue

- [x] Desktop API 版本提升；产品 snapshot 提供 Adapter Catalog 和 Font Profiles。
- [x] Dictionary 页面只管理 metadata 和文字规则。
- [x] 新增 Font Library/Edit Surface，管理有序 family candidates 与引用状态。
- [x] Workflow Editor 为每个 Target 独立维护 Adapter 多选、Dictionary 排序和多条 Font Bindings。
- [x] 普通界面不显示内部 ID、路径、hash、签名或旧 Hook Type。

### 5. Verification

- [x] Rust 相关 package 合同逐纵切 red → green。
- [x] 根测试编排、Clippy、fmt 和架构检查通过。
- [x] 前端生产构建与仓库 Playwright 全部通过。
- [x] Git diff 不含 `local-test/`、本机盘符、用户名、PID、真实软件路径或证据链接。

## Current

正交模型已贯通 Workflow、Desktop persistence、Runtime Descriptor/Deployment、Tauri 与 Vue。
Dictionary、Font Profile、Adapter Catalog 和逐 Target 组合均有独立产品表面与合同覆盖。

## Next

1. 以现有 Artifact Descriptor seam 设计在线 Dictionary Catalog/安装记录，不污染 payload。
2. 建立生产 Extension/Runtime Bundle 与词典导入发布流程。
3. 在明确授权的真实软件上执行可见验收与首个可分发桌面构建。
