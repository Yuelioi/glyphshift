# Web 桌面首个真实目标候选

Status: Complete

调查日期：2026-08-05

## 结论

在只有外部软件可选时，首选 **Obsidian（仅限用户显式开启 CLI，并使用一次性空白 Vault）**，备选
**draw.io Desktop**；**Electron Fiddle 不作为首个翻译覆盖目标**，只保留为 Electron transport/兼容性
诊断样本。

后续已确认可使用自有 Wails + WebView2 工具 Yotta，因此首个真实验收目标改为 Yotta。它的宿主接入、
隔离数据目录和生命周期都可由双方明确控制，优先级高于本报告中的外部候选；具体边界见
[Yotta WebView 宿主接入评估](yotta-webview-host-seam-assessment.md)。本报告继续保留为外部目标备选记录。

这个排序不是按开源程度，而是按“能否在不绕过宿主授权的前提下验证真实 DOM 翻译闭环”排序。
Obsidian 官方 CLI 需要用户在设置中显式开启，并直接提供 `dev:dom`、`dev:cdp` 与 `eval`；这是三者中唯一
已经由宿主拥有和公开的脚本/调试 seam。draw.io Desktop 的普通界面文字覆盖更丰富，也开源、离线、免账号，
但官方发布版目前只确认了单实例、共享 `userData` 和通用 Electron 调试端口，没有确认私有 pipe 或独立 profile
入口。Electron Fiddle 同样是单实例，而且主要内容是首版必须排除的 Monaco 代码编辑器。

该结论只授权后续做一次有界验收，不等于生产支持，也不授权下载、启动或连接任何真实软件。

## 对比

| 候选 | 技术栈与 Windows 体量 | 普通 DOM 翻译价值 | profile / 单实例 / CLI 边界 | 决策 |
| --- | --- | --- | --- | --- |
| Obsidian | 官方部署文档明确其安装器承载 Electron；当前 Windows Universal 发布资产约 311.7 MiB | 设置、菜单、侧栏、弹窗等普通 DOM 丰富；官方 `dev:dom` 也直接证明 DOM 可查询。编辑器、笔记正文和用户输入必须排除 | CLI 必须由用户在设置中显式开启；可用 `vault=<name|id>` 定向 Vault，应用未运行时首条命令会启动它。CLI 提供 `dev:dom`、`dev:cdp`、`eval`，但权限很高，不能把真实个人 Vault 当测试环境 | **首选，条件 Go** |
| draw.io Desktop | 官方仓库定义为 Electron 包装的 draw.io 编辑器；当前 x64 Windows installer 约 139.7 MiB，另有 portable/ZIP/MSI | 本地 HTML/JavaScript 编辑器，菜单、面板、对话框和工具栏文字丰富；图布、SVG/Canvas 图元文字不计首版覆盖 | 官方源码调用 `requestSingleInstanceLock`，第二次启动转发给旧实例；README 明确 Windows 状态存放在固定应用数据目录。支持 portable、`--disable-update`、完整文件/导出 CLI 和开发模式，但未发现官方独立 profile 或私有调试 pipe 合同 | **备选，transport seam 未闭合** |
| Electron Fiddle | Electron 官方项目；React + Blueprint UI，并内含 Monaco；当前 x64 Windows setup 约 132.9 MiB | 普通按钮、菜单和面板可测，但主工作区是 Monaco，按 GlyphShift DOM 合同应排除，因此有效翻译面不足 | 官方源码使用 `requestSingleInstanceLock`；协议参数会转发给既有实例。官方开发入口是源码 `start`，发布版未提供独立 profile 或宿主拥有的私有翻译/调试接口 | **拒绝作为首个覆盖目标；仅作诊断样本** |

发布体量取自各官方 GitHub 仓库最新 release asset 元数据，只用于比较测试成本；它不是安装后磁盘占用，
也不作为兼容性合同。

## 首选：Obsidian

### 为什么排第一

- 官方 CLI 文档明确要求在 **Settings → General** 中由用户开启命令行接口，符合“显式授权”原则。
- CLI 明确提供 `dev:dom`、`dev:cdp` 和 `eval`，并说明这些开发命令用于插件、主题和自动化测试；
  GlyphShift 无需打开未认证的通用 remote-debugging 端口，也无需注入 Blink/V8 私有符号。
- 官方文档说明 Vault 是本地文件夹，且 CLI 支持用名称或 ID 定向 Vault。可以创建完全空白、可删除的
  验收 Vault，不需要账号或云服务。
- 普通界面区域足够多，同时可以清晰排除编辑器内容、笔记正文、密码和用户输入，适合验证我们的 DOM
  ownership 合同是否真的成立。

### 风险与限制

- Obsidian 本体不是开源目标，不能把一次通过推导为所有 Electron 软件都可支持。
- CLI 的 `eval`/CDP 能力高于翻译所需权限；生产实现只能发送固定、版本化并做完整性校验的本地脚本，
  不接受用户字典、远端响应或 UI 文本拼接成可执行代码。
- CLI 连接的是正在运行的应用，并可能在应用未运行时启动它。验收前必须关闭个人实例，只使用一次性
  Vault；不得读取、记录或修改个人 Vault。
- `vault=<name|id>` 是内容空间选择，不等于 Electron profile 隔离。若实际验收发现应用级缓存、插件或
  最近文件仍跨 Vault 共享，则立即判定 No-Go，不扩大权限规避。

## 备选：draw.io Desktop

draw.io Desktop 是更理想的“翻译覆盖演示”目标：官方仓库开源、支持免安装版本，编辑器资源本地打包，
默认不发送图表数据，并可用 `--disable-update` 禁止更新检查。它的本地 Web UI 比 Electron Fiddle 拥有
更多普通菜单、面板和对话框文本。

但它不是当前首选，原因是 transport 和状态隔离尚未闭合：官方源码明确实施单实例锁，第二次启动会把
参数交给已有实例；README 还明确状态存放在固定应用数据目录。虽然 Electron 官方提供
`--remote-debugging-port`，我们已经证明随机 loopback 端口不构成认证，而 Electron 官方文档没有把
`--remote-debugging-pipe` 列为这个发布目标的支持合同。因此在目标没有确认私有 pipe、宿主插件或独立
profile 入口前，只能保留为备选。

授权后若先验证 draw.io，必须从“官方发布版是否真实接受父进程继承 pipe 且不接管旧实例”开始；失败就
停止，不能退化为固定端口或复用用户正在工作的实例。

## 拒绝项：Electron Fiddle 作为首个覆盖目标

Electron Fiddle 很适合验证 Electron 版本、启动参数和 renderer attach，因为它是 Electron 官方维护、
开源且体量适中。但它的主要工作区是 Monaco，正好属于首版应排除的代码编辑器；剩余普通 DOM 文本不足以
代表真实工具软件的翻译收益。源码同样使用单实例锁，且未提供发布版独立 profile 或私有宿主接口。

因此它只适合回答“某个 Electron 版本能否连接/创建 isolated world”，不适合回答“GlyphShift 的实时
翻译是否对真实软件有价值”。不要把 transport 测试通过写成翻译适配成功。

## 授权后验收门槛

首选目标只有在用户再次明确授权后才可下载或启动，并必须同时满足以下门槛：

1. **环境隔离**：只使用一次性空白 Vault；关闭个人实例，不安装社区插件，不登录账号，不开启同步；所有
   本机路径、日志和截图只进入 `local-test/`。
2. **显式入口**：由用户主动开启官方 CLI；GlyphShift 只调用官方命令，不扫描端口、不附加未授权进程、
   不改目标安装文件。
3. **只读基线**：先用 `dev:dom` 验证目标窗口、普通 DOM 文本和版本；不得读取笔记正文、Vault 文件列表、
   storage、cookie、网络请求或插件数据。
4. **可见收益**：至少在设置、菜单、侧栏或弹窗中捕获 30 条去重普通界面原文；编辑器、用户输入、密码、
   Canvas/SVG 内容不计入通过数量。
5. **翻译闭环**：同一会话完成首代字典替换、第二代热更新、应用重渲染后的重新决策，以及停用时仅恢复
   仍由 GlyphShift 拥有的节点。
6. **生命周期**：目标关闭、renderer 重建、Vault 切换和 CLI 关闭都必须明确中断；不得显示“仍在翻译”，
   不得自动重启或切回个人 Vault。
7. **权限收敛**：只执行固定本地脚本，脚本有硬编码的节点/批次上限；任何参数都通过数据通道传递，不能
   拼接成 JavaScript。验证完成后移除 observer、binding 和 DOM 修改。
8. **Go/No-Go**：若官方 CLI 不能稳定定位目标 renderer、会触及 Vault 内容、无法恢复 DOM，或需要开放
   未认证端口，则本目标立即回到 Roadmap，不用私有符号、注入或 profile 劫持补洞。

## 一手资料

- [Obsidian CLI：显式启用、Vault 定向与开发命令](https://obsidian.md/help/cli)
- [Obsidian 本地 Vault 数据模型](https://obsidian.md/help/data-storage)
- [Obsidian 团队部署：本地/离线与 Electron installer](https://obsidian.md/help/teams/deploy)
- [Obsidian 官方发布资产](https://api.github.com/repos/obsidianmd/obsidian-releases/releases/latest)
- [draw.io Desktop 官方仓库与 Windows 发布形式](https://github.com/jgraph/drawio-desktop)
- [draw.io Desktop 主进程：本地 Web UI、单实例与 CLI](https://github.com/jgraph/drawio-desktop/blob/dev/src/main/electron.js)
- [draw.io Desktop CLI 参数合同](https://github.com/jgraph/drawio-desktop/blob/dev/src/main/args.js)
- [draw.io Desktop 官方发布资产](https://api.github.com/repos/jgraph/drawio-desktop/releases/latest)
- [Electron Fiddle 官方仓库](https://github.com/electron/fiddle)
- [Electron Fiddle 单实例与协议启动逻辑](https://github.com/electron/fiddle/blob/main/src/main/protocol.ts)
- [Electron Fiddle React/Monaco 依赖](https://github.com/electron/fiddle/blob/main/package.json)
- [Electron Fiddle 官方发布资产](https://api.github.com/repos/electron/fiddle/releases/latest)
- [Electron 官方命令行开关：`--remote-debugging-port`](https://www.electronjs.org/docs/latest/api/command-line-switches)
