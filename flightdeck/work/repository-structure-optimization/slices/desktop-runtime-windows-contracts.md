# Desktop Runtime Windows 合同结构

Status: Complete

## Goal

保持 12 项 Windows Runtime Bundle、合成目标和授权真实 Host 合同的名称、ignored 条件、本机输入协议与
用户可见断言不变，把 1117 行单文件按被验证的公开 interface 场景组织。

## Baseline

- 根文件混合 4 类场景：Bundle 完整性、集中 Capture、合成目标 Runtime 生命周期、授权真实 Host；另有
  双软件 Runtime Pool 隔离场景。
- `TargetProcess`、backend/workflow builder 和 Adapter id 是跨场景确定性 fixture；真实软件位置和输出只从
  环境变量或 `local-test/evidence/` 获得。
- 12 项测试全部 ignored，普通 CI 只编译并列出合同；本切片不运行真实软件或读取本机配置。

## Plan

- [x] 保留一个小型共享 fixture 根，按 Bundle、Capture、合成 Runtime、授权 Host、Pool 场景拆分子模块。
- [x] 保持 12 个测试名称及 `#[ignore]` 理由精确一致，不修改断言和本机输入协议。
- [x] 运行 test listing/package 编译、Clippy、格式、架构与 diff check。
- [x] Standards/Spec 自审 fixture 所有权、场景独立性、隐私边界与授权条件。

## Decisions

- `TargetProcess` 与 workflow builders 留在测试根：它们是多个公开合同共同使用的测试语言，不属于任一场景。
- 授权 Host 三项合同集中在独立模块，避免确定性合成目标测试意外复用真实进程环境变量。

## Result

- 1117 行单文件变为 184 行共享 fixture 根，以及 Bundle 72、Capture 164、Synthetic Runtime 322、
  Authorized Host 308、Pool 83 行五个场景模块。
- 格式化前按原区间重新拼接的源码 SHA-256 与 checkpoint 完全一致；`rustfmt` 后 12 个测试名称和
  `#[ignore]` 理由集合仍精确一致。
- 普通合同运行结果为 0 passed、12 ignored；没有执行真实软件。package Clippy、workspace 格式、架构检查
  与 diff check 全部通过。
- 审核未删除断言：现有断言分别覆盖 Bundle 完整性、Capture 隐私/单 checkpoint、热更新/重连、授权
  Host 和多软件隔离，没有仅依赖私有字段或重复验证同一内部步骤的断言。

## Review

- Standards：共享根只拥有跨场景测试语言；每个子模块以被测公开 interface 和授权级别命名，真实 Host
  环境变量没有泄漏到合成目标场景。
- Spec：UIA 密码排除、process-family checkpoint、GDI/GDI+ 像素/字体保护、generation 更新、目标重启、
  授权 Host diagnostics/Capture 与 Pool 隔离断言均保持原样。
