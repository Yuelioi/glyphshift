# 快速探针测试流程

Status: Complete

## Outcome

用户从 Probe 页只需选择一个正在运行的程序并点击“开始测试”，Desktop 即可建立测试会话并打开结果页；
不再要求先后前往软件、词典和探针三个位置创建资产。

## Primary flow

1. 点击“快速测试”。
2. 浏览程序，或使用现有前台程序捕获结果。
3. 确认系统识别出的软件名称、架构与可用能力，点击“开始测试”。
4. App 自动创建 Probe 专用软件绑定、空白草稿词典和兼容 Adapter Plan，并直接打开实时结果。
5. 有价值时选择“保留”，将测试资产提升为普通软件与词典；放弃时选择“结束并清理”，删除仅由该
   测试会话拥有的资产和证据。

已有软件、已有词典、语言、Adapter 和实时预览进入“高级选项”，不占用默认路径。

## Module seam

由一个深 `QuickProbeSession` Module 暴露窄 Interface：接收授权程序选择、目标语言和可选高级覆盖，
返回已启动的 Probe 或稳定失败。软件复用/创建、草稿词典、Adapter Plan、启动、补偿清理、保留与退出
均属于其 Implementation；Vue 不跨三个 command 编排半完成状态。

## Lifecycle

- 临时资产必须有显式所有权，不靠 ID 前缀或显示名称推断。
- 创建失败要回滚本次新建且未被引用的资产；不能删除创建前已经存在的软件或词典。
- “结束并清理”只删除仍由该测试独占的资产；已经被 Workflow、其他 Probe 或用户“保留”的资产继续
  存在并说明未删除原因。
- App 崩溃或退出后可以恢复未结束的测试会话，不遗留不可见孤儿资产。

## Non-goals

- 不把 Quick Probe 变成第二套 Software、Dictionary 或 Probe 数据模型。
- 不默认展开全部技术选项，不要求用户理解 Adapter ID、进程类型或 DLL。
- 不在本 Slice 实现翻译 Provider、取点浮层或 OCR。

## Verification

- [x] 默认路径只有程序选择和“开始测试”一个主要动作。
- [x] 新程序与空仓库可在一次提交中得到运行中的 Probe；已有程序可复用而不重复登记。
- [x] 失败补偿、保留、独占清理、引用保护和崩溃恢复具有 Shell 合同测试。
- [x] Playwright 覆盖中英文、深浅主题、空仓库、已有资产、高级选项和失败恢复。
- [x] 960×640 与 1440×900 无溢出，键盘与可访问名称完整。

## Next

完成后交付[UIA 取词翻译产品入口](interactive-acquisition-product-flow.md)。

## Progress

- 已确认不能由 Vue 串联现有 Software / Dictionary / Probe 三个命令；Shell 将新增单一
  `QuickProbeSession` 边界与持久 ownership ledger。
- ledger 只记录会话和资产所有权；Software、Dictionary、Probe 继续使用现有模型。启动前先登记
  Preparing 状态，覆盖进程在任意创建步骤退出后的恢复清理窗口。
- Shell 已落地单一 `start / retain / cleanup` Interface：自动复用或创建 Software、创建临时词典、
  选择兼容 Adapter Plan 并启动 Probe；失败时按显式 ownership 补偿，不根据名称或 ID 猜测归属。
- Probe View 通过 `quickProbe` 标记恢复后的临时会话；ledger 采用 pending/primary/backup 替换协议，
  可恢复主文件切换中的中断。最终定向合同 8/8 通过，覆盖 Workflow 引用后的 Software/Dictionary
  保留；相关 Shell Clippy 通过。
- 本轮自审修正了三个边界问题：已有但停止的软件不能绕过运行检查；retain 必须先确认 Probe 仍在，
  再释放 ownership；ledger 提交成功后的陈旧 backup 清理不再反向制造内存/磁盘状态分歧。
- 后续验证按 Flightdeck 比例化策略执行：先写 Shell 生命周期合同，再跑相关 Shell 测试与 Probe
  Playwright；不默认运行全仓测试。
- Probe 页现以“快速测试”为主动作、手工新建为次要路径；临时详情只呈现保留、ownership 清理和
  运行控制。快速路径 Playwright 3/3、Probe 页面边界回归 11/11、既有 Shell Probe 合同 8/8 通过；
  中英文、深浅主题、失败重试、前台捕获和两档目标尺寸均已验证，机械界面检测零问题。
