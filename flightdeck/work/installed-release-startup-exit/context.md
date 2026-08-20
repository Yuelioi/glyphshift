# 稳定上下文

## 已知事实

- 故障发生在通过 GitHub Actions 发布的 Windows 安装包安装之后。
- 用户可见症状是应用界面短暂出现后进程退出；管理员身份运行不能改变结果。
- 发布记录表明交付物包含桌面壳、Runtime Bundle、候选清单与 SHA-256 校验文件，但既有验收主要覆盖
  未安装候选与生产加载器，不能替代已安装布局的启动验收。
- `v0.2.0` 使用 Dictionary `/3`；此前本机工作区保存的 Dictionary `/2` 仍是合法旧产品数据。
- `/2 → /3` 的结构变化是让 `translation` 可缺省以表达 pending entry；既有 `/2` 的非空字符串译文
  可无损表示为 `/3` completed entry。

## 诊断结论

- Desktop Backend 启动时遍历 `dictionaries/*.json`，当前 Package decoder 遇到 `/2` 会返回
  `InvalidArtifact("dictionary-json")`。
- Desktop setup 没有恢复或呈现该错误，而是把它返回给 Tauri setup；Tauri 随后 panic，用户只看到
  窗口短暂出现后消失。
- 已安装程序在空数据根可稳定显示，证明已安装桌面壳、嵌入前端、WebView 与 Runtime Bundle 足以完成
  基础启动。单个合成 `/2` 文件能独立触发故障，排除了真实词条内容与权限因素。
- 对可理解的旧 schema 应执行原子迁移；对真正损坏或未知的单个词典应隔离并报告，不能让整个 Desktop
  setup 失败。跳过受影响词典时，引用它的 Workflow 必须安全停用并保留可恢复定义。

## 本机配置存储边界

- 默认 `workspace` 是整个桌面数据根，不只是 Dictionary 容器；软件、工作流、词典、AI Profile
  元数据、任务历史和部分 App 数据共享该根。
- 当前 `ai-profiles.json` `/4` 保存 Profile 名称、协议、服务地址、模型、策略、默认选择和明文 API
  Key。旧 `/2`、`/3` 只保存凭据引用，系统凭据库仅作为一次性迁移来源。
- 剩余孤儿系统密钥无法单独重建 Profile 的协议、模型和地址，界面也不会把孤儿凭据列为 Profile；迁移
  不得猜测归属或静默删除这些条目。
- 启动恢复不得再建议用户删除整个数据根；至少先做可逆备份，并优先修复或隔离造成启动失败的具体制品。

## AI Profile 决策

- 用户明确选择 API Key 随 AI Profile 明文保存，不再使用 Windows Credential Manager。产品必须在
  表单旁明确说明本机明文风险，不能继续声称 Key 位于系统凭据库。
- 编辑 Profile 时必须能显示或隐藏当前 Key；默认遮蔽，显示动作具有可访问名称，保存后 Key 仍可再次
  查看。Profile 删除直接删除同一 JSON 中的 Key，不依赖外部凭据清理。
- 现有 schema 与系统凭据应尽可能一次性迁移：先原子写入包含 Key 的新 Profile artifact，成功后才
  清理能够确定归属的旧凭据；无法关联的孤儿凭据不得猜测或静默删除。
- Codex 订阅继续复用本机 Codex CLI 的 ChatGPT 登录，不把登录 Token 复制进 Profile。官方模型 ID 为
  `gpt-5.6-sol`；旧的错误值 `gpt-sol-5.6` 必须迁移或规范化。

## 约束

- 最新桌面应用的重建与审阅启动必须统一使用 `scripts/review-app.ps1`，不能把独立构建的桌面壳与旧
  Runtime 目录混用。
- GUI 自动化验证使用仓库 Playwright CLI/test runner。
- 所有本机安装路径、进程信息、原始日志、截图和临时数据只进入 `local-test/`；Flightdeck 只记录
  可移植的命令、结论、计数与剩余风险。
- 诊断必须先有能捕获“启动后快速退出”的自动反馈信号，再进入根因假设与修复。
- UIA、UIA worker 与依赖它们的 OCR worker 只作仓库归档：禁止构建、测试、运行、打包、验证、修改或
  纳入 workspace Cargo 门禁，除非当前用户请求明确授权归档 UIA 作业。活动包验证必须使用仓库显式
  入口，不得运行原始 `cargo * --workspace`。
- 归档 UIA Rust 测试统一使用 `archive_uia_` 名称前缀与 `#[ignore = "archive-only UIA…"]`；唯一手工
  入口默认拒绝，只有显式 `-ArchiveUia` 才能用 `archive_uia_ -- --ignored` 选择执行。

## 验收语义

- 复现探针能区分快速退出与稳定存活，并在普通启动和管理员启动路径上给出确定结果。
- 有效 Dictionary `/2` 必须在保留 identity、metadata、revision 与全部 completed entries 的前提下
  原子升级为 `/3`；失败时不得覆盖原文件。
- 单个不可迁移词典不得阻断主界面启动；产品应保留文件、给出可定位诊断，并安全停用依赖它的运行期望。
- Dictionary 恢复过程不得丢失或重置 AI Profile 元数据与明文 Key。
- Profile 明文 Key 可保存、重启后显示、替换和随 Profile 删除；序列化、错误与任务日志不得额外复制 Key。
- Codex 连接测试必须使用本机已登录 CLI 与有效模型，错误时保留可恢复的具体原因，不能只显示笼统失败。
- 修复后的 GitHub Actions 等价安装布局可稳定启动，生产 Runtime Bundle 仍通过加载校验。
- 若问题依赖安装后数据或环境，产品必须给出可诊断且可恢复的错误，而不是无提示退出。
