# 管理表双击编辑与 Dictionary 导出

## Goal

让 Dictionary 导出按钮可靠打开原生保存窗口，并为本地管理表提供一致的双击编辑快捷方式，同时
保留明确的行级操作。

## Current

已完成。Tauri capability 同时允许打开和保存文件；保存窗口调用失败会在 Dictionary 页面显示
可恢复错误。Workflow、Software、Dictionary 和 Probe Run 表支持双击非交互单元格进入编辑或详情，
交互控件不会冒泡触发快捷方式，在线目录保持只读语义。

## Decisions

- 双击是熟练用户快捷方式，不取代行级编辑、打开、导出或删除按钮。
- 统一的事件委托只按当前分页 DOM 行索引解析本地条目，不给整行添加按钮角色。
- 复选框、开关、链接、输入和操作按钮属于独立交互，不触发行双击。
- 原生保存窗口异常必须可见；用户取消保存不视为错误。

## Verification

- 红回归稳定捕获保存窗口异常无反馈，以及四类本地管理表双击无响应；实现后 2/2 转绿。
- Desktop 生产构建、完整 Playwright 46/46 与 Impeccable UI detector 通过。

## Next

- 返回[工作流 Runtime 错误反馈](workflow-runtime-error-feedback.md)。
