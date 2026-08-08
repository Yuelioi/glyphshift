# Unity Mono Standard UI observe-only 原型

Status: Complete

## Goal

回答一个窄问题：在不依赖逐游戏符号、偏移或预装插件的 Windows x64 Unity Mono Shipping-like 目标中，
能否仅通过公开 Mono runtime metadata 识别 TMP/uGUI 标准组件，枚举附加前已有文字并观察附加后的 setter
更新，同时可靠拒绝 NGUI/自绘 Mesh 路径。此 Slice 只建立 `ObserveOnly` 能力，不修改目标对象或字符串。

## Delivery

- [x] 建立 host-independent Observer 状态模块，以单一 `apply(event)` interface 处理初始快照、setter、对象
  回收与停用；UTF-16、按对象去重、空值、容量和失效输入全部在实现内 fail-open。
- [x] 建立确定性 Shipping-like Mono 宿主，公开与真实 Player 同形的 domain、assembly、image、class、
  method、object 与 managed string 合同；覆盖 TMP、uGUI 和无标准 UI 三种目标。
- [x] 用窄 Runtime seam 证明 x64、Mono 模块、核心导出与目标程序集/类型门禁；缺模块、缺导出、IL2CPP、
  NGUI-only 或未知版本必须在 attach 前安全拒绝。
- [x] 证明附加时初始对象枚举、附加后标准属性 `set_text` 观察、同一对象重复值去重、空值/失效对象
  放行，以及停用后不再发布 Observation。
- [x] 只在合成合同通过后对两个已授权 Mono 正例做无写回 observe-only attach；负例必须明确拒绝，任何
  崩溃、目标状态修改或无法正常退出都触发 No-Go。
- [x] 自审所有权、线程附加、GC handle、异常边界与停用后 pass-through 顺序；本 Slice 不进入生产
  Catalog、Runtime Bundle 或 `TextReplace` 支持声明。

## Current

真实样本 gate、纯状态合同、可调用 Mono 合成宿主和正式 Controller/Target Runtime 实机链均已通过。
workspace 现有 host-independent Observer 状态包和一个未进 Catalog/Bundle 的 native 原型包：前者以单一
`apply(event)` 隐藏快照折叠、setter 去重、对象回收、停用、UTF-16 与容量边界；后者已实现 Native ABI
激活、late-attach metadata/JIT 门禁、`CanvasUpdateRegistry.PerformUpdate` 与 TMP/uGUI `set_text` detour、
一次性主线程快照、weak GC handle 和停用后 pass-through，不把 Mono 导出 ABI 扩散到 Core/Runtime。

激活分成 metadata 和 live-object 两级：标准类型存在只允许建立临时主线程 dispatch，必须在三秒内枚举到
至少一个真实 TMP/uGUI 对象才返回 ACK；空快照会失败并清理，避免 Unity 自带标准程序集让 NGUI-only
目标误报成功。初始 Observation 在 ACK 前缓存，等 Target Runtime 标记 active 后的下一次 dispatch 才
发布。Runtime gate 只读检查 Windows x64、Mono/IL2CPP 模块和 25 个实际使用的公共导出；GC handle 使用
指针宽度 `mono_gchandle_*_v2`，四份真实 Mono runtime 对照均无缺项。

合成宿主验证附加前 `Score: 0` / `Download Resources`、附加后 `Score: 248` / `Download Res` 和停用后
无发布。native package 的 11 个单元测试与 6 个 Shipping-like 集成测试、Clippy 和合成端到端宿主均
通过。正式部署链最终在两个样本 gate 正例分别采集 10 条和 15 条唯一非空原文，另一个跨代正例采集
389 条；三次均激活、停用并正常退出。NGUI-only 同后端负例被 live-object gate 拒绝且正常退出，没有
目标崩溃、强制清理或目标文字写入。

## Next

- 本 Slice 已关闭；不要在 Observe-only Adapter 内逐步混入字符串替换、managed string 分配或停用恢复。
- 下一步进入独立的
  [Unity Mono Standard UI TextReplace 原型](unity-mono-standard-ui-writeback-prototype.md)，先设计共享 runtime
  seam，再分别证明初始对象写回、动态属性 setter、同进程代次更新和停用恢复。
- 当前动态观察仍只承诺 C# `text` 属性产生的 `set_text`；TMP `SetText(...)`、`SetCharArray` 和直接 backing
  array 更新不在本 Slice 结论内。

## Decisions

- Mono 与 IL2CPP 是独立后端；本 Slice 不复用或顺带实现 IL2CPP。
- TMP/uGUI 是唯一允许的标准 UI 范围；UI Toolkit、NGUI、TextMesh、Sprite/texture 和自研 Mesh 均拒绝。
- 初始对象枚举与后续 setter 观察缺一不可；只捕获启动后的变更不能提升为可用原型。
- `mono_gc_walk_heap` 只允许从 GC pre-stop-world profiler 回调进入；不得从 attach 线程直接调用，也不为
  加速验收主动强制目标执行完整 GC。Mono 官方同时限制 profiler 创建、call-context introspection 和
  method instrumentation filter 必须在 managed code 启动前安装，因此该路线不能用于运行中 attach。
- Runtime gate 只允许读取当前进程已经加载的模块与导出，不主动 `LoadLibrary`；Mono 与 IL2CPP 同时存在时
  只在完整 Mono 合同通过后进入本 Slice，缺任一导出均拒绝。
- `MonoObservationRuntime` 负责线程、managed-method detour、GC handle、事件串行化与停止；
  `ObserverDriver` 只接受标准
  UI profile 和语义事件，setter 早于初始快照或重复 attach snapshot 都会 fail-closed 并清理本地状态。
- 首版只允许从跨版本 uGUI `CanvasUpdateRegistry.PerformUpdate` detour 进入 Unity 主线程并执行一次性
  `FindObjectsOfType`；不建立窗口线程调度、WndProc 替换或逐版本 native PlayerLoop Hook。缺该 managed
  dispatch 时可靠拒绝，而不是从 injection thread 调用 Unity API。
- 标准类型/方法存在不是支持证据；激活 ACK 必须等待主线程枚举出至少一个 live TMP/uGUI 对象。空快照、
  三秒内无 dispatch 或枚举失败都保守拒绝。临时 dispatch hook 在失败后保持进程内 pass-through；当前
  Target Runtime 保留已加载 package，不会卸载仍被 detour 引用的代码。
- 初始快照只用于 live-object ACK，Observation 必须延迟到 Target Runtime active 后的下一次 dispatch，
  不能在 `activate()` 尚未返回时发布。
- Observe-only 只发布原文与技术身份，不命中 Dictionary、不替换字符串，也不承诺可见翻译。
- TMP `SetText(...)` 有多种字符串、数值、StringBuilder 与数组 ABI，本 Slice 不以单个 `set_text` Hook
  冒充覆盖这些重载；后续若扩展，必须另建具备签名门禁和独立合成合同的 Slice。
- 复用现有 `NativeRuntimeHostV1::decide_utf16` 作为 Observation seam；不扩展 Core、Runtime protocol 或
  Native ABI。Mono metadata 解析与 Hook 保持为 native Adapter implementation 的内部 seam。

## Verification

- 完成时 host-independent Observe 合同 6 passed，native 合同 17 passed（11 unit + 6 integration），两者
  Clippy 均通过；后续 TextReplace Slice 在发布前将 package 收敛改名，当前命令与总数以该 Slice 为准。
- 本地可调用 Unity Mono 合成宿主完成 Native ABI 激活、初始 TMP/uGUI 快照、setter 更新、停用与停用后
  无发布；输出 4 条预期 Observation，进程正常结束。
- 四份本地真实 Mono runtime 的 25 项 late-attach 导出对照均无缺项；四套 uGUI/TMP 程序集都包含
  `PerformUpdate`、标准字段与 setter，CoreModule 跨代包含 `FindObjectsOfType`。原始文件与结果仅在
  `target/local-test/`。
- 正式 Controller/Target Runtime 本地部署链：两个最终正例分别得到 10 条和 15 条唯一非空 Observation；
  一个额外跨代正例得到 389 条；三次均停用并正常退出。NGUI-only 同后端负例激活被拒绝并正常退出。
- 未运行全仓测试；两个 package 都未进入 Catalog 或 Runtime Bundle。

## Boundaries

- 外部成品、源码副本、路径、日志、截图、进程与模块信息只保存在 `target/local-test/`。
- 生产选择不得按软件品牌、可执行文件名、逐游戏地址或机器码签名分支。
- 当前结论仅为有界 Observe-only 原型 Go；没有 Dictionary 写回、可见翻译或生产支持声明。
