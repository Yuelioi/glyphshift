# 探针创建来源统一

Status: Complete

## Goal

把探针创建收敛为一个入口：目标既可来自 Software Library，也可来自当前运行程序；词典既可引用已有
Dictionary，也可由系统自动命名并创建为临时资产。用户无需为了临时测试先去软件、词典、探针三个
页面准备资产，同时不建立第二套 Software、Dictionary 或 Probe 模型。

## Current

当前 Probe 页存在两个重叠入口：普通“新建探针任务”只接受资料库 Software，并在新建 Dictionary 时
要求填写名称和语言；“快速测试”可以捕获前台程序并自动创建临时 Software/Dictionary，但不能选择
已有 Dictionary。两条路径分别编排同一组领域对象，导致用户必须先理解资产生命周期，入口本身也
无法表达 Software Library 只是目标来源之一。

后端已经具备可复用基础：`QuickProbeSession` 会按程序路径复用或创建 Software、创建 Dictionary、
选择兼容 Adapter Plan，并通过持久 ownership ledger 处理失败补偿、保留、独占清理和崩溃恢复。本
Work 将深化该边界为统一的 Probe creation command，而不是让 Vue 串联多个资产命令。

## Decisions

- 持久 Probe Run 继续绑定一个已解析的 Software 和一个已解析的 Dictionary；“资料库 / 当前程序”与
  “已有 / 临时”只属于创建输入，不改变运行时模型。
- 当前程序来源在用户明确捕获后产生短期 preflight；若路径已在 Software Library 中则复用，否则创建
  有显式 ownership 的临时 Software。
- 临时 Dictionary 不要求用户命名。系统根据目标显示名生成名称，源语言默认 `auto`，目标语言使用
  当前创建设置；用户保留后它就是普通 Dictionary。
- 任一临时资产存在时，Probe 显示“临时”状态并提供“保留 / 结束并清理”；全部引用资料库资产时就是
  普通 Probe，不制造无意义的临时会话。
- OCR 与交互式取词降为最后阶段 fallback，不占用本 Work。

## Architecture note review

外部架构建议中“Adapter 发现文字、Context 理解文字、Translator 转换文字”的职责分离与现有边界一致，
继续保留。OCR 作为 fallback 的优先级也采纳。暂不按 native/framework/accessibility/rendering 搬迁目录：
这些是 Technology 分类，不是所有权与生命周期边界，当前 Registry 已可表达。也不把窗口、程序、坐标、
上下文和置信度全部塞入全局 `TextEvent`；持续 `TextObservation` 保持最小，富证据继续走有界 envelope
或交互式 acquisition result。

## Next

- 本 Work 已完成。后续回到运行时能力 Roadmap；OCR 与交互式取词保持最后阶段 fallback，不从本
  Work 延伸新的 Provider、截图或划词范围。

## Progress

- 已确认现有 Quick Probe 不是第二套领域模型，而是 Software、Dictionary 与 Probe 的原子编排和所有权
  补偿层；本 Work 将复用并深化它。
- 已明确资料库资产与本次创建来源不是同一概念，并同步根领域语言。
- Desktop API `/24` 已以 `desktop_create_probe_from_sources` 固定四种来源组合；旧 quick-test 创建
  payload 没有兼容读取或双写。
- Probe 页已合并为单一入口，临时词典由系统命名，用户无需先去软件库或词典库创建测试资产。
- 外部架构建议已按本项目边界评审：采纳 Adapter/Context/Translator 职责分离与 OCR fallback；拒绝
  按技术类别重组所有权目录以及膨胀全局 TextObservation。
- 最终定向门禁通过：Desktop Shell 来源/ownership 合同 9/9、Probe Playwright 12/12、前端生产构建
  与 Rust fmt check 均通过；未运行全仓测试。
