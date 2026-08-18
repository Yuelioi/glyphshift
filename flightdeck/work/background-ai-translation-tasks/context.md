# 后台 AI 翻译任务 context

## 已确认产品合同

- 全局同时只有一个活动翻译任务，不建立自动队列；重复发起返回当前任务。
- 切换页面、操作其他功能或最小化 Glyphshift 不停止任务；真正退出会中断未完成请求，重启后不得自动
  重发。
- 任务运行期间只锁目标词典写入；目标词典仍可查看，其他词典、软件、工作流、探针与设置不锁定。
- 开始词典任务前保存有效草稿。完成批次立即写入目标词典，取消保留已经完成的译文。
- 任务页只展示运行事实和脱敏历史；原文、译文、Base URL 与凭据不进入可见历史统计。
- Token 只显示 Provider 或 Codex 实报值；缺失时显示未提供，不再展示运行后预估值。
- Codex 订阅 Profile 复用本机 Codex CLI 的 ChatGPT 登录，隐藏 Base URL 与 API Key，不读取、复制或
  保存 `auth.json`。
- Codex 批次使用 `codex exec --ephemeral --json --output-schema`，在空临时目录与只读沙箱中运行；
  Profile 默认并发 1、低推理强度。
- Codex 订阅用量不换算货币费用；API Profile 保留 Provider 实报的计费 Token。
- 任务工作台遵循频率与价值排序：当前任务、整体进度和停止始终突出；模型统计与任务历史分 Tab；批次
  和单次历史详情按需展开。
- 标题栏任务入口必须使用任务/进度语义图标，不使用容易被理解为界面语言切换的语言图标。
- 翻译不是通用推理任务；新建与旧版 Profile 的推理策略默认关闭。用户可以显式改为自动、低、中、高
  或最高，但只有协议和模型支持的值才发送。
- DeepSeek V4 默认 thinking enabled/high；关闭必须显式发送 thinking disabled（Chat 格式）或等价的
  Responses 推理关闭值。DeepSeek 的 low/medium 会映射到 high，不能把它们宣传成实际低思考。
- Codex Profile 的关闭映射为官方支持的 minimal；自动不覆盖本机模型默认，其余强度映射到 Codex
  `model_reasoning_effort`。

## 当前实现事实

- Rust 后台线程拥有唯一活动任务、批次写回、目标词典锁、终态历史与中断恢复。
- Vue 根组件持续同步任务状态；页面切换和 WebView 计时器不拥有执行或写回。
- 当前任务页已经能显示五项实报 Token，但三个使用频率不同的表面仍堆在一页，需要本轮分 Tab。

## 验证边界

- Rust 定向合同覆盖唯一活动任务、锁定/解锁、逐批写回、取消和重启中断恢复。
- Provider 合同覆盖 Codex JSONL usage、结构化结果、缺失 CLI/未登录/模型失败与取消。
- Playwright 覆盖切页继续、重复发起跳转、任务页状态、目标词典只读、中英文和紧凑窗口。
- Playwright 还需覆盖三个 Tab、默认当前任务、显式停止、批次折叠与任务详情展开。
- 推理控制合同必须检查最终 wire body，而不是只验证 Profile 保存；真实 ds 小批需对比 reasoning Token。
- 最终桌面审阅必须使用 `scripts/review-app.ps1`，并以同步 Shell 与 Runtime Bundle 启动。
