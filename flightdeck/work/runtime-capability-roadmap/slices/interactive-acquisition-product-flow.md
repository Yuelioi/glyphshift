# UIA 取词翻译产品入口

Status: Complete

## Outcome

用户在 Glyphshift 中启动一次取词，把鼠标指向目标软件中的文字并按快捷键后获得 UIA 结构化原文、
翻译结果和目标外呈现；
目标界面不会被修改，结果不会冒充 `TextReplace`。

## Scope

- 先完成既定 [Desktop Acquisition Command](interactive-desktop-acquisition-command.md)，让 Runtime Pool
  管理 Point 请求与现有 Workflow/Probe 的共存。
- 设计一个取点状态和一张有界结果表面：原文、译文、来源、复制、重试和关闭；不新建完整 Probe。
- Dictionary 精确命中优先；产品尚未选择生产 Translation Provider 时保留原文并明确标记未命中，
  不虚构网络翻译。未来 Provider 必须返回来源与失败语义。
- 快捷键只触发一次明确的用户请求；密码、受保护目标、权限不匹配、超时和取消均失败关闭。
- UIA 无结构化文本时返回稳定、可恢复的错误；“尝试 OCR”动作由下一独立 Slice 显式接入，
  这一层绝不静默截屏。

## Non-goals

- 不持续监听鼠标悬停，不把坐标或结果写入 Dictionary。
- 不用透明全屏 Overlay 承担通用目标绘制或原位替换。
- 不在这一 Slice 实现生产 OCR Provider。

## Verification

- [x] 受验证 Debug Bundle 对真实 Windows 标准控件进程完成一次 UIA Point 取词；产品 Shell 与结果
  表面分别通过组合合同和 Playwright 证明，不把观察冒充目标内写回。
- [x] Dictionary 命中、Provider 回退、部分成功、取消与 presenter 清理均通过产品合同。
- [x] UI 使用仓库 Playwright CLI 覆盖键盘、浅色、英文、960×640 与错误状态。
- [x] 本地截图、Runtime Bundle、原文和目标信息只进入本机忽略目录；跟踪合同只使用合成 fixture。

## Next

进入[生产 OCR 回退](interactive-ocr-production-fallback.md)：只在 UIA 明确无结构化结果且用户点击动作后，
获取授权 frame 并复用同一翻译与结果表面。

## Progress

- 前置 Desktop Acquisition Command 已完成：产品层已有 platform-neutral Point request/result、稳定错误
  code 与取消合同。当前先审计既有 `interactive-translation` 组合接口和桌面 target/point 获取能力，
  再固定最小用户流程；GUI 变更只用仓库 Playwright CLI 验证。
- 后端产品入口已接通：用户先选择已登记软件与本地 Dictionary，再将鼠标指向目标文字并按
  `Ctrl+Shift+F9`；Shell 只在这一次显式请求中读取前台进程与光标位置，通过短命 UIA
  acquisition runtime 取词，不持续监听鼠标，也不把 target identity 暴露给前台。
- `interactive-translation` 组合已用于 Dictionary 精确命中；当前产品尚未配置网络 Translation
  Provider，因此未命中会保留结构化原文并标记 `missing`，不会虚构在线回退。后端定向测试 5/5
  通过，覆盖命中、全未命中、预取消、重复请求拒绝与状态销毁取消；序列化合同未包含 PID、路径、
  worker、grant 或 token。
- 标题栏新增紧凑“取词翻译”入口；同一 Modal 覆盖选择、等待 `Ctrl+Shift+F9`、读取中、命中、未命中、
  错误、复制、重试和关闭。新 Playwright spec 3/3 通过，并完成两轮 960×640 英文浅色截图审核；
  Vue 类型检查通过，桌面 API 版本提升到 22，不保留未发布旧命令兼容层。
- Slice 自审补跑产品组合合同 6/6、Shell 产品合同 5/5、最小 UIA worker 1/1、受验证 Debug Bundle
  Windows Point 合同 1/1，以及相关包 Clippy `-D warnings`。Windows 合同首轮因固定坐标窗口被遮挡而
  fail-closed 为 `PermissionDenied`；最小复现证明授权本身正常，合成目标增加显式置顶命令后原始合同
  转绿，跨进程 Point 拒绝策略未放宽。无临时 debug instrumentation 留在仓库。
