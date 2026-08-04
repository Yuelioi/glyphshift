# Windows Console `WriteConsoleW` 观察适配器

## Goal

以最小、可验证的 Target Process Adapter 观察 Console 客户端实际经过 `WriteConsoleW` 的 UTF-16
输出，供探针生成候选词条；不把一次 API 命中夸大为整个 CMD、子进程或 Terminal session 覆盖，也不
声明翻译写回。

## Current

`windows.console.write-console` 已作为第五个 Adapter 进入正式 Runtime Bundle。Descriptor 只声明
`TextObserve + ObserveOnly + TargetProcess`；桌面探针可以选择它，工作流会过滤所有不具备
`TextReplace` 或 `FontSubstitute` 的 Adapter，因此不会误导用户把它用于实时翻译。

原生实现拦截 `KernelBase!WriteConsoleW`。授权 CMD 的隔离实测显示，直接按 `Kernel32!WriteConsoleW`
安装会因 API-set 转发边界得到零命中；改为共享实现入口后，同一持续输出场景得到 18 次命中，并得到
干净的 `Open`、`File`、`Edit` 三类观察文本。捕获层会按 CR/LF 切分非空片段，换行符不进入探针候选。

后续真实 Clink 场景证明 `WriteConsoleW` 还会承载 ANSI/VT 终端协议，包括独立的光标移动、清行、
样式和 OSC 超链接调用。Adapter 现在会在观察边界做有界解析：纯控制序列不再形成候选，样式或超链接
包裹的可见正文会去除协议后继续上报；原始 UTF-16 buffer 和 `WriteConsoleW` 返回语义保持不变。

一次显式 `WriteFile` 探索在该 CMD 场景中得到零信号，且其通用字节流、代码页、句柄分类与异步 buffer
生命周期尚无完整合同，因此实验实现已删除，没有进入生产 Adapter。

真实 Windows 进程合成合同现已覆盖一个父进程和由它启动的长寿命子进程。只向父进程部署 Runtime
时，父进程的 `ParentConsoleText` 可观察，子进程的 `ChildConsoleText` 不会泄漏到父诊断；对子进程的
opaque target 单独部署同一 Runtime 后，才会在子诊断中观察到 `ChildConsoleText`，父诊断仍为空。
这证明 Process Family 发现不会让进程内 Hook 自动继承，但 Controller 已具备按成员独立部署能力。

## Decisions

- Adapter 只观察被注入进程中实际经过 `WriteConsoleW` 的调用；不承诺子进程、重定向、pipeline、
  `WriteFile`、`WriteConsoleOutput*` 或 Windows Terminal 的完整覆盖。
- 不注入系统 `conhost.exe`；系统宿主内部 seam 不是稳定公开 ABI，也会扩大用户授权范围。
- 不声明 `TextReplace`。返回字符数、部分写入、VT 状态、字符列宽和光标行为尚不能安全映射回 source。
- ANSI/VT 属于 Console Adapter 的协议正规化边界，不进入 Dictionary 或通用 Capture 匹配层；CSI、
  OSC 与控制字符串被剥离，只有可见且非空的行片段成为 Observation。
- Detour 安装后以原子 feature gate 激活或停用；非法 UTF-16、过长文本、重入、捕获失败和内部异常全部
  fail-open，原始输出只调用一次。
- Console、进程和 session 证据留在 Adapter / Runtime；Dictionary 继续保持纯
  `source + translation`。
- Workflow 仍只激活 inventory 中一个 target；Probe 未显式缩小时会激活全部已授权家族成员。每个
  Target Runtime 只产生 batch，Desktop 的唯一 `FileCaptureSink` owner 将各 target 的 observation
  串行写入同一 checkpoint，不会让多个进程竞争文件 revision。
- 暂不立项 Controller-owned ConPTY。它只能覆盖由 Glyphshift 新建并拥有的终端 session，不解决任意
  已有软件的子进程观察；除非产品明确增加“受管终端”体验，否则不扩大当前范围。

## Verification

- 纯 Adapter 合同覆盖观察、非法或过长 UTF-16、重入、失败开放、CR/LF 切分和 original 恰好一次。
- 新增 Clink 风格 CSI、样式与 OSC 超链接回归合同；原生 `WriteConsoleW` 也已证明样式正文可观察、
  纯控制调用不产生候选，停用后继续透传。
- 正式 Runtime Bundle 校验通过，包含一个 Console Observer、总计五个 Adapter。
- 授权隔离 CMD 实测通过：18 次命中，观察集合为 `Open`、`File`、`Edit`；真实日志与输入只保存在
  `target/local-test/`。
- 真实进程合成 parent/child 合同通过：父进程部署不观察子进程；子进程单独部署后只在自己的诊断流
  中观察，Controller 的两个 target 互不串流。
- 真实 Process Family capture 合同通过：父子 target 同时部署并写入同一 checkpoint；暂停/恢复、
  子进程退出隔离与父进程继续采集均通过。
- 全 workspace tests、Clippy `-D warnings`、fmt、架构检查和桌面 production build 通过；桌面
  Playwright 51 条全绿，覆盖观察型 Adapter 只出现在探针、混合选择保持预览、仅观察选择禁用预览。

## Next

- Console Slice 到此收敛。Process Family Probe 的单写入聚合已完成；突然退出前一个 pull 周期内尚未
  运输的进程内尾批次仍可能丢失，记录为 transport 残余风险，不在 `WriteConsoleW` Adapter 内解决。
- Stage 3 转向 UI Automation observe-only Adapter，先交付 Isolated Worker/IPC 与合成 Provider。
