# Probe 创建器可用性纠正

Status: Complete

## Trigger

首次实机审阅暴露四个问题：空 Software/Dictionary Library 被显示成禁用来源；任务名称与目标语言
错误归入“高级选项”；Active Process 只有快捷键捕获，没有可直接浏览的运行中软件列表。

## Result

- 来源 Tab 始终可进入；空资料库显示原因和可用替代路径，不再伪装成权限不足。
- 任务名称常显于表单首项；临时 Dictionary 的目标语言紧随其生命周期说明。
- Active Process 支持已加载的运行中可见软件选择器、显式刷新和快捷键前台捕获。
- Windows Controller 只枚举具有可见标题窗口且可读取程序绑定的目标，按程序路径去重，不跨 Desktop
  seam 暴露 PID；Desktop 继续用同一 preflight 判断架构、Runtime 与已有 Software 命中。

## Verification

- 空资料库、运行中列表、普通字段层级的 Playwright 回归先红后绿。
- Windows Controller 枚举排序/去重单测通过。
- Desktop Probe creation 合同与既有捕获、资料库、取消重开页面路径保持通过。
