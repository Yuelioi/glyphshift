# 探针恢复与 AI 状态收敛

Status: Finished

## Goal

修复真实探针点击“连接并继续”时错误报告兼容方式不适用的问题，并把探针内 AI 翻译呈现收敛为紧凑
状态与“查看任务”，完整进度只在全局翻译任务页出现。

## Current

真实探针保存的 9 个 Adapter 与当前兼容列表完全一致；失败不是计划过期。零 Adapter 激活现在只有在
全部 Adapter 安全停用、捕获结束且状态仍是同一 inactive deployment 时才清空失败部署，随后可直接换
Adapter 组合重试；任一清理失败仍保留重启保护。探针 AI 单行状态与恢复文案也已完成。

## Next

None

## Progress

- 用户确认完整 AI 任务信息应归翻译任务页，探针只需要知道是否运行及进入详情的入口。
- 安装版真实差分确认保存集合与兼容集合均为 9/9；首次组合激活返回组件未激活，随后每个单 Adapter
  都明确要求目标重启，排除保存计划、Runtime Bundle 缺项和单个 Adapter 污染。
- 错误恢复文案改为“完整重启目标软件；仍失败则当前界面暂不支持”，不再引导到未变化的探针设置。
- 探针页 AI 状态在 960×640 与 1440×900 均为约一行高度；“查看任务”进入完整任务页，词典和任务页
  原有详细进度保持不变。
- AI/Probe Playwright 34/34、生产构建与架构检查通过；界面机械检查无发现。
- 正式候选 Runtime Bundle 验证 9 个 Adapter 并安装成功；真实安装版恢复文案 Playwright 1/1 通过，
  Glyphshift 已关闭诊断端口并普通启动。
- 本 Work 不运行归档 UIA 测试。
- 用户要求在 tag/push 前修复底层失败部署驻留，不接受把可回滚失败永久升级成重启。
- 目标 Runtime 公开 interface 红灯稳定复现 Qt-only 失败后 GDI-only 返回 `InvalidDeployment`；修复后
  同一进程直接激活 GDI，且兼容 Adapter 与不可用 peer 共存合同继续通过。
- Target Runtime、Session、Desktop Runtime、Desktop Shell 定向回归通过；显式活动包全量门禁和
  Desktop Playwright 104/104 通过，架构、格式、生产构建与隐私扫描通过。
- 最终 NSIS 候选包含验证过的 9 Adapter Runtime Bundle，安装 EXE 与 installer payload 一致；安装版
  已普通启动。UIA 未调度。

## References

- [稳定上下文](context.md)
- [执行计划](plan.md)
- [探针暂停与离线恢复](../probe-pause-offline-target/index.md)
