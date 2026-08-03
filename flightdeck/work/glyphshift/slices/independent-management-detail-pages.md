# 独立管理详情页

Status: Complete

## Goal

统一 Workflow、Software、Dictionary 与 Probe Run 的编辑/详情交互，避免同类对象在 Modal 与
独立页面之间切换心智模型。

## Current

- Workflow 创建/编辑和 Software 编辑已从 Modal 改为占满模块内容区的独立页。
- 四类详情页复用同一返回、标题、状态、说明与主要动作头部。
- Workflow 详情使用 176px 左侧单层分区导航和右侧任务内容；左栏不承载业务数据或第二套表单。
- Esc 返回上一级；可见 Modal、菜单和选择器优先消费 Esc，不触发页面误退。
- Workflow、Software 与 Dictionary 的未保存修改在页面返回、顶部导航和窗口关闭前统一确认。
- Probe Run 返回前会提交当前待保存译文；失败时保留详情页与待处理状态。

## Decisions

- 管理列表和详情页是同一顶级模块内的两种状态，不新增主导航层级。
- 简单创建和低频设置仍可使用 Modal；复杂、多步骤或持续作业详情使用独立页。
- 行级显式操作继续保留，双击只是效率捷径。
- Workflow 的左右结构是分区导航/内容，不恢复旧的软件列表/配置主从栏；返回位于共享详情头。

## Verification

- `npm.cmd run build`
- `npm.cmd test`：48/48
- 1440×900 与 960×640 的 Workflow、Software 编辑页视觉证据位于本地忽略目录。
- Impeccable layout detector：0 项。

## Next

回到工作流 Runtime 错误反馈 Slice，修复 `AlreadyActive` 误报与多进程目标选择。
