# Glyphshift 通用产品交付

Status: Open

## Goal

交付不依赖具体软件品牌的 Windows 运行时界面翻译产品：工作流组合软件与独立词典，动态
Runtime Bundle 提供可验证的文字和字体写回能力，桌面界面保持高密度、可扩展和技术上诚实。

## Current

通用实现已从旧组织仓库的第二代 workspace 提升为独立仓库根目录并完成验证。31 个 Cargo
package、Tauri/Vue 桌面、测试支持与架构检查均在当前仓库内闭合，不再存在根级旧 workspace、
旧 GUI、旧脚本或兼容依赖。Dictionary、Font Profile、Platform/Technology、Adapter Catalog
与 Workflow Target 已完成正交拆分，并贯通 Backend、Runtime reconcile、Target Runtime 合同
和桌面 UI。标题栏提供 Help/Settings 入口，Help 从 Runtime Bundle 展示当前 Adapter Catalog，
Settings 不再保存在线翻译服务地址；桌面后端不可用按产品错误处理，不显示伪连接状态。

`AppSettings/1`、Vue I18n 与 `CommandError/1` 已交付：应用 mount 前解析真实语言和主题设置，
完整核心 UI 提供 `zh-CN`/`en-US`，Settings 只展示可立即生效并持久化的界面语言与主题，Tauri
command 跨 seam 返回稳定 code + typed args。UI locale、字典内容语言和动态 Artifact
presentation locale 保持正交。生产构建、Playwright 20 项、Rust workspace test、fmt、Clippy
及真实 Tauri 设置跨重启验证均通过。

旧 AE 与 Premiere 词典仅作为未接入 Runtime 的源数据保存在 `archive/dictionary-sources/`；
架构检查禁止生产代码引用归档路径或旧 schema。原仓库保持不变，继续作为只读回退来源。

最终模型与[技术方案](references/composable-assets-technical-design.md)已确认：Dictionary、Font
Profile、Platform、Technology 和 Adapter 独立，Workflow Target 是唯一组合根；在线 Dictionary
采用纯 payload 与外部 Artifact Descriptor。当前未发布，不实现旧 schema 兼容。

## Next

- 以现有 [Artifact Descriptor seam](references/composable-assets-technical-design.md) 设计在线
  Dictionary Catalog、presentation locale、安装记录与签名边界。
- 随后建立生产 Extension/Runtime Bundle 与词典导入发布流程。

## Progress

- [独立仓库迁移](slices/repository-migration.md)完成：根结构、脚本、标识和文档均已去除旧代际边界。
- 根级测试编排显式预构建原生 DLL/EXE；workspace test、Clippy、fmt 和架构检查通过。
- 可组合资产与 Adapter Catalog Slice 完成；桌面 API v7、Dictionary `/2`、Font Profile `/1`、
  Workflow `/2` 与 Target Runtime Deployment `/2` 已落地。
- Help/Settings Slice 完成；Adapter 版本与 Feature presentation 贯通 Runtime、Tauri 与 Vue，
  标题栏去除连接状态，设置页去除在线翻译器配置。
- AppSettings 与桌面多语言 Slice 完成；语言/主题持久化、完整中英文 UI、结构化桌面错误及真实
  Tauri 跨重启验证闭合，Playwright 20 项通过。
- 桌面生产构建与 Playwright 18 项 GUI 回归通过；覆盖双词典排序、双字体绑定冲突恢复、
  多 Target 暂存隔离、完整 metadata 保存及 960×640/1440×900 稳定帧。
- 完成同类产品、现有 Adapter Registry 与 macOS 拦截边界的一手资料调研。
- 完成字典、字体、在线 metadata 和插件 Catalog 的主流实践终审；技术方案与测试 seam 已固定。
- 完成 PowerToys、VS Code、Docker Desktop、OBS、Zotero 设置作用域与 Vue/Tauri i18n 一手资料
  调研，否决跨 IPC 直接传 vue-i18n key，推荐语义错误码与前端映射。

## References

- [稳定约束](context.md)
- [实际架构](references/architecture.md)
- [Hook 与 Adapter 调研](references/hook-translation-market-and-adapters.md)
- [Capability Adapter 与跨平台拦截调研](references/adapter-registry-and-cross-platform-interception-research.md)
- [可组合模型主流实践复核](references/composable-dictionary-font-and-adapter-model-review.md)
- [可组合资产技术方案](references/composable-assets-technical-design.md)
- [工作流配置调研](references/workflow-configuration-research.md)
- [全局设置与桌面端多语言调研](references/settings-and-desktop-i18n-research.md)
