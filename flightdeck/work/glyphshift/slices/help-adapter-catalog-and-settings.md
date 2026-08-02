# Help Adapter Catalog 与本地资产设置

Status: Complete

## Outcome

桌面标题栏提供 Help 与 Settings 图标入口。Help 使用 Runtime Bundle 的公开 Catalog 展示当前
Adapter 名称、说明、Platform、Technology、Feature、配置要求与版本；普通界面不暴露内部 ID、
DLL target、hash 或签名。Settings 不再提供在线翻译器/URL 输入，只说明本地 Dictionary、
Workflow 组合和未来网站/字典市场下载方向。

桌面服务连接文案已移除。浏览器预览不伪装连接状态；真实 Tauri 后端不可用时阻断产品数据页，
但 Help 仍可打开用于排查。

## Delivery

- Runtime Adapter presentation 增加语义版本与 Feature 集合，Tauri Desktop API 升为 v7。
- Help 使用 Nuxt UI 高密度表格，并支持 1440×900 与 960×640 两档桌面窗口。
- Settings 变为三个薄边界说明行，直接导航到 Dictionary、Workflow 与 Help。
- 删除前端 `theme`、`translationSource` 与无效的在线翻译源写入路径。

## Verification

- `cargo test -p glyphshift-desktop-runtime -p glyphshift-desktop-shell`：24 passed，4 ignored（需授权实机）。
- `scripts/test.ps1` workspace 全量测试、`cargo clippy --workspace --all-targets -- -D warnings`
  与 `cargo fmt --all -- --check`：通过。
- `npm run build`：通过。
- `npm test`：18 passed；包含 Help/Settings 行为、内部信息不泄露和两档截图。
- 1440×900 与 960×640 两档 Playwright 视觉检查通过；本机证据未进入跟踪文档。
- 本地开发启动流程的 4 项真实 Runtime contract 通过，Tauri 桌面窗口已稳定启动。
