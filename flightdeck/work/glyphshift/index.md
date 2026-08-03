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
合同，以及不冒充生产网络/密钥策略的内存 ports。

Dictionary 详情已按真实使用反馈完成信息降噪，Dictionary `/2` 也已收敛为纯
`source + translation`：Location、Context、keep 与逐词条字体均已从 payload、DTO、编辑器和
Workflow compile 删除。Runtime 内部路由不回流到 Dictionary；当前 Adapter 无法识别主界面、
弹窗或面板，因此没有伪造 Region/Location 配置。

Runtime Bundle 现提供 `ExtTextOutW`、`TextOutW`、`DrawTextW/DrawTextExW` 与 `GdipDrawString`
四个独立 Adapter，中文 presentation 在 Windows PowerShell 5.1 下按 UTF-8 稳定读取。可恢复
Probe Run 已完成：每个任务绑定一个软件、一个 Dictionary 与一组 Adapter；Dictionary 是唯一翻译
内容资产，内部 Observation Index sidecar 只保存技术证据。管理页与详情明确分离，详情以一张联合
表支持搜索、多选、行内编辑、批量处理、分页、导出、暂停/继续和 Live Preview。5,000 条基准下由
Rust 分页，Vue 只渲染当前页；1 秒双 revision 轮询只在变化时重取页面。

旧 AE 与 Premiere 词典仅作为未接入 Runtime 的源数据保存在 `archive/dictionary-sources/`；
架构检查禁止生产代码引用归档路径或旧 schema。原仓库保持不变，继续作为只读回退来源。

最终模型已确认：Dictionary、Platform、Technology 和 Adapter 独立，Workflow Target 是唯一组合
根并直接拥有可选 Font Policy；在线 Dictionary 采用纯 payload 与外部 Artifact Descriptor。
当前未发布，不实现旧 schema 兼容。

## Next

- 继续 Dictionary Catalog、生产 Runtime Bundle 与授权实机验收。

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
- Dictionary 详情完成渐进披露重构；设置与规则 Modal 单列化，矮窗口可滚动，Playwright 22 项及
  1160×527/960×640/1440×900 视觉检查通过。
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
- 桌面生产构建与 GUI 回归覆盖双词典排序、内联字体候选排序与 coverage、多 Target 暂存隔离、
  完整 metadata 保存及 960×640/1440×900 稳定帧。
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
- [纯字典、常用 Adapter 与捕获探针](slices/dictionary-hooks-and-probe.md)
- [可恢复探针工作区与实时草稿](slices/capture-workspaces-and-live-drafts.md)
- [Dictionary-owned Probe Run](slices/dictionary-owned-probe-runs.md)
