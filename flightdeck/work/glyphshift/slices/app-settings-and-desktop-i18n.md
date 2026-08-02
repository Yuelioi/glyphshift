# AppSettings 与桌面端多语言

Status: Completed

## Deliverable

建立可直接发布的 `AppSettings/1` 与桌面多语言基础：应用启动前解析真实设置，完整提供
`zh-CN` / `en-US` 核心界面，设置页只展示已经有执行路径的语言和主题选项，Tauri command
跨 seam 返回稳定语义错误码与类型化参数，前端统一翻译。

## Contracts

- `AppSettings/1` 是应用级聚合，只包含 `localePreference = system | zh-CN | en-US` 与
  `themePreference = system | dark | light`；不包含 Dictionary、Font、Adapter、Software、
  Workflow 或目标软件语言。新安装默认深色；只有用户显式选择后才跟随系统主题。
- 设置存储由桌面 composition root 管理；浏览器 Playwright 预览使用相同 interface 的本地
  adapter，不把存储分支散落到页面。
- command failure 使用 `CommandError/1 { schemaVersion, code, args, diagnosticId? }`；Rust 不返回
  vue-i18n key 或最终展示句子，Vue 在一处完成 code → key → message 映射。
- 动态 Dictionary/Adapter 的名称和说明属于 Artifact presentation locale；本 Slice 不把它们
  塞进核心语言包，缺少目标语言时展示 Artifact 默认 presentation。
- 当前产品未发布，不读取旧 schema，也不实现兼容迁移。

## Scope

- 立即交付：设置持久化、系统 locale/theme 解析、`zh-CN` / `en-US`、文档 `lang`、完整核心
  UI 文案迁移、结构化 command error、真实 Settings 页面、双主题与双语言 GUI 回归。
- 暂不展示：开机启动、托盘/关闭行为、更新、遥测、诊断导出、账号、网络或市场设置。它们在
  对应 runtime capability 和命令存在后单独加入，不能保存没有执行意义的值。

## Steps

1. **冻结 interface 与测试 surface**
   - 定义 `AppSettings/1`、locale/theme preference、effective locale/theme 和 `CommandError/1`。
   - 先为设置默认值、持久化往返、非法 schema/枚举以及错误序列化写 Rust 合同测试。
2. **实现深设置 Module**
   - 在 Tauri composition root 新建设置 Module，隐藏路径解析、JSON 读写、原子替换和默认值。
   - 对外只暴露 load/current/update；Tauri 提供读取和更新两个 command。
3. **建立前端启动 Module**
   - 安装 Vue I18n；以 `zh-CN` 为 master schema，提供完整 `en-US`。
   - 应用 mount 前读取设置并解析 effective locale/theme；同步 `html.lang`、Nuxt UI locale 和
     `dark` class，监听 system preference 变化。
4. **迁移产品 UI**
   - 把标题栏、工作流、软件、词典、字体、帮助、通用 Modal/表格、空态和错误提示迁到语义 key。
   - 数据内容、软件名称、词典内容、Adapter technical data 不翻译。
5. **收口桌面错误 seam**
   - 将 Tauri command 的 `Result<_, String>` 改为 `Result<_, CommandError>`，按 backend/runtime
     错误语义生成 code 与 typed args。
   - 前端集中处理已知/未知错误码；未知错误安全回退并保留 diagnostic id。
6. **交付真实 Settings**
   - 使用现有高密度桌面视觉，采用两组薄分隔设置行，不使用卡片或说明性占位栏目。
   - 语言和主题更改立即生效并持久化；保存失败时回滚 UI，显示可恢复错误。
   - 标题栏提供一键明暗切换；它与设置页共用同一 AppSettings 状态和持久化路径。
7. **验证与收尾**
   - 运行 Vue typecheck/build、Playwright 双语言/双主题/窄宽视口、Rust unit/workspace、fmt 与 Clippy。
   - 用仓库 Playwright runner 截取桌面视图并完成一次集中修复、一次确认；再启动真实 Tauri
     桌面验证设置跨重启保留。
   - 更新 PRODUCT、DESIGN、CONTEXT 与 Flightdeck；本机证据只写入 `target/local-test/evidence/`。

## Acceptance

- 首次运行按系统偏好得到受支持的 locale，并默认使用深色主题；显式选择可覆盖默认值且跨重启保存。
- 切换到 English 后核心 UI 不残留硬编码中文，切回中文后无页面刷新或状态丢失。
- Settings 只出现两个真实生效的全局偏好，不出现在线翻译器、市场、启动、托盘或诊断占位。
- Tauri command 不再以本地化字符串作为错误 interface；未知错误码不会让 UI 崩溃。
- 960×640 与 1440×900 下中英文、深浅主题均无溢出，键盘焦点和可访问名称完整。

## Current

`AppSettings/1`、Vue I18n、`CommandError/1`、真实设置页与标题栏主题切换均已交付。生产构建、
Playwright 20 项、Rust workspace test、fmt 与 Clippy 全部通过；中英文、深浅主题和两档视口截图
已完成终审。真实 Tauri 窗口启动正常，通过原生窗口操作写入主题偏好后，重启仍能读取同一设置
合同并应用有效主题。

## Next

本 Slice 已完成。后续由 Glyphshift Work 继续设计在线 Dictionary Catalog、Artifact
presentation locale 与安装记录。
