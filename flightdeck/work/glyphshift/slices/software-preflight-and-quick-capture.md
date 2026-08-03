# 软件接入预检与快速捕获

## Goal

在创建软件记录前拦住当前 Runtime 明确无法接入的目标，并允许用户通过全局快捷键捕获正在前台
运行的软件，减少手工寻找和输入程序路径的成本。

## Current

已完成。新增软件现在必须先通过本地预检；`Ctrl+Shift+F8` 第一次进入等待状态，切换到目标软件后
再次按下会读取前台程序、返回 Glyphshift 并打开已填充的新增弹窗。捕获或浏览都不会直接保存，
路径变化会立即使旧预检失效。

## Decisions

- 前置屏障只阻断能够确定的失败：无效 Windows PE、当前不支持的位数、程序未运行、重复绑定、
  Runtime Bundle 或文本观察 Adapter 不可用。
- GDI/GDI+ 等真实绘制路径无法从文件名或前台窗口可靠推断；预检通过只表示具备基础接入条件，
  实际文字捕获仍由添加后的 Probe 证据确认。
- 当前交付 Runtime 只接纳 `x86_64` 目标；其他位数明确提示，不让用户在工作流阶段才遇到模糊失败。
- 快捷捕获只读取前台进程身份，不自动持久化；用户仍可确认名称、描述和路径后添加。
- 使用 `Ctrl+Shift+F8` 而不是裸 `F8`，避免覆盖目标软件常见的功能键操作。

## Verification

- `cargo test -p glyphshift-controller-windows`：3 个单元测试、2 个目标 Hook 合同和 4 个 Windows
  inventory 合同通过，1 个需显式实机授权的合同保持 ignored。
- `cargo test -p glyphshift-desktop-shell`：原 31 项完整通过；新增 preflight 状态决策合同单独通过。
- `cargo clippy -p glyphshift-controller-windows -p glyphshift-desktop-shell --all-targets -- -D warnings`
  与 `cargo fmt --all -- --check` 通过。
- Desktop production build 通过；完整 Playwright 41/41 通过，新增合同覆盖未检查不可保存、检查后
  可保存、两段式捕获预填和取消后不重开。
- Impeccable UI detector 返回空结果。

## Next

- 返回工作流 Runtime 错误反馈 Slice，处理 `AlreadyActive` 幂等接管与多进程目标选择。
