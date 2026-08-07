# Adapter 实现二级目录归档

Status: Complete

## Goal

降低具体 Adapter crate 增长后的导航成本，同时保持现有 Adapter interface、Registry 分类、Native ABI、
Worker/process seam、Cargo package 身份和运行时行为不变。

## Current

`crates/adapters/implementations/` 平铺 23 个 crate。首次家族迁移用 `platform/implementations` 分开共享
平台与具体实现是正确的，但没有继续区分实现的获取机制，导致系统原生接口、UI 框架、可访问性与 OCR
fallback 混排。随着 OCR 两个 crate 加入，原来“先保持平铺”的成本判断已经失效。

## Decisions

- `native/`：Console、Direct2D、DirectWrite、Win32 DrawText/GDI 与 GDI+。
- `framework/`：GTK3/Pango、Qt Painter 与 raylib。
- `accessibility/`：UI Automation descriptor 与 Worker。
- `fallback/`：OCR descriptor 与 Worker。
- 同一技术的 descriptor、Native DLL、共享 native support 或 Worker companion 留在同一分类下。
- 二级目录是物理导航，不是新的运行时 seam；不改 package/lib/target 名、依赖集合、Registry identity、
  ABI 或 bundle artifact。
- 不创建没有真实 crate 的 `rendering/`，避免把未来 DX/OpenGL/Vulkan 方案提前固化成空结构。

## Steps

- [x] 验证 23 个源目录与唯一目标分类，记录所有 workspace/path/script/document 消费者。
- [x] 机械迁移目录并修正因深度变化产生的相对路径。
- [x] 更新 Adapter README、架构文档与历史导航链接，不把外部软件名称或本机路径写入仓库。
- [x] 运行路径残留、Cargo metadata、架构/格式/diff 与 Adapter 家族定向门禁。
- [x] 自审 package/target/feature/依赖集合未发生语义变化，完成 Work 收口。

## Next

None.

## Evidence

- 23 个实现 crate 唯一归类为 native 13、framework 6、accessibility 2、fallback 2；叶目录未平铺残留。
- Cargo metadata：62 个 workspace package、27 个 Adapter package；全部 manifest path dependency 指向
  存在目标，package identity 保持。
- `architecture-tests/check.ps1`、`cargo fmt --all -- --check`、`git diff --check` 通过。
- GDI 6/6、Qt Painter 6/6、UIA 12/12、OCR 5/5，共 29/29 定向合同通过；未运行全仓测试。
- 架构精确合同补齐此前漏记的 OCR Worker、进程授权 package、UIA Worker → 进程授权及 Desktop Shell
  → 交互翻译依赖；两者均由当前实现真实使用，不是路径迁移制造的新依赖。

## Review

- Standards：二级目录提升同类 Adapter 的 locality，但没有形成浅转发 module 或新的公开 interface；
  合成 Windows 路径仅用于确定性路径解析测试，机器特定路径仍由隐私扫描拒绝。
- Spec：采纳外部建议中有实际 crate 支撑的四类导航；没有创建空 `rendering`，没有改 package/lib/target
  名、Registry identity、ABI、feature、制品或实现源码。
