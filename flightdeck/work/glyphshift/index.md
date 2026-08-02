# Glyphshift 通用产品交付

Status: Open

## Goal

交付不依赖具体软件品牌的 Windows 运行时界面翻译产品：工作流组合软件与独立词典，动态
Runtime Bundle 提供可验证的文字和字体写回能力，桌面界面保持高密度、可扩展和技术上诚实。

## Current

通用实现已从旧组织仓库的第二代 workspace 提升为独立仓库根目录并完成验证。31 个 Cargo
package、Tauri/Vue 桌面、测试支持与架构检查均在当前仓库内闭合，不再存在根级旧 workspace、
旧 GUI、旧脚本或兼容依赖。工作流、软件和词典产品模型已经贯通 Backend、Runtime reconcile
与桌面 UI；词典支持默认/逐条字体、固定 Hook 范围和批量表格编辑。

旧 AE 与 Premiere 词典仅作为未接入 Runtime 的源数据保存在 `archive/dictionary-sources/`；
架构检查禁止生产代码引用归档路径或旧 schema。原仓库保持不变，继续作为只读回退来源。

## Next

- 继续[工作流产品交付](slices/workflow-product-delivery.md)：为每个 Target 维护独立的有序
  Dictionary 栈。
- 为归档词典设计一次性、显式的产品导入路径；导入逻辑不得进入 Core 或 Runtime 热路径。

## Progress

- [独立仓库迁移](slices/repository-migration.md)完成：根结构、脚本、标识和文档均已去除旧代际边界。
- 根级测试编排显式预构建原生 DLL/EXE；workspace test、Clippy、fmt 和架构检查通过。
- 桌面生产构建与 Playwright 37 项 GUI 回归通过，Nuxt UI 控制台无属性透传警告。

## References

- [稳定约束](context.md)
- [实际架构](references/architecture.md)
- [Hook 与 Adapter 调研](references/hook-translation-market-and-adapters.md)
- [工作流配置调研](references/workflow-configuration-research.md)
