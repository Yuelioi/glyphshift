# Unity Mono Standard UI TextReplace 原型

Status: Complete

## Goal

在已完成的 Windows x64 Unity Mono TMP/uGUI Observe-only seam 上回答下一个窄问题：能否不修改业务程序集、
不依赖逐游戏符号或偏移，把 Dictionary 决策安全应用到附加前已有对象和附加后的标准属性 `set_text`，
支持同进程新代次，并在停用时把所有仍存活对象恢复为最新原文。只有可见写回与恢复同时成立才算 Go。

## Delivery

- [x] 将 Mono domain、metadata、JIT entry、main-thread dispatch、managed string 与 GC handle 收敛为共享深
  runtime seam；Observe-only 和 TextReplace lifecycle 不复制 25 项 FFI 与 hook 所有权。
- [x] 在确定性可调用 Mono 宿主中证明：初始对象翻译、动态 setter 翻译、重复原文去重、空值/非法 UTF-16
  放行、同进程第二代更新，以及停用后恢复最新原文。
- [x] managed string 只能在已验证 Mono 主线程上下文创建；Dictionary 未命中、容量超限、对象回收、
  host 错误、Rust panic、重入或三秒内无恢复 dispatch 都 fail-open，并保留可延迟完成的恢复状态。
- [x] 在两个已授权 Standard UI 正例完成首代可见译文、第二代更新和停用恢复；NGUI-only 负例继续拒绝。
- [x] 完成所有权、线程、GC、重入、hook pass-through 与进程退出自审；通过前不进入 Catalog/Bundle。

## Current

原型已完成。未发布的 Observe-only 两个 package 已在实现前收敛并改名为
`glyphshift-adapter-unity-mono-standard-ui` 与 `glyphshift-adapter-unity-mono-standard-ui-native`；单一 native
会话协商 `TextObserve` / `TextReplace`，共享 26 项实际使用的 Mono 导出、对象表、三组 detour 与停用状态，
没有并存两套 FFI 或 hook owner。

host-independent 写回状态机保存“最新业务原文 / 当前译文 / Dictionary generation”，主线程 dispatch 负责
初始写回、每 30 帧的轻量 generation probe、整批热更新和停用恢复。managed string 通过
`mono_string_new_utf16` 创建并以强 `*_v2` GC handle 覆盖 setter 调用；对象仍使用弱 handle 跟踪。停用线程
最多等待三秒；超时会返回失败并保持 pending restore，setter 已转为透传，下一次已验证主线程 dispatch 会
完成回滚和句柄清理，不能把超时伪装成成功。

## Next

- 新建独立生产化 Slice，把已经通过 gate 的 Adapter 接入 Catalog/Runtime Bundle、中文技术说明与文档 URL；
  接入前复核正式包布局、默认能力选择和停用失败文案。
- 保持 TMP `SetText(...)` 多重 ABI、IL2CPP、UI Toolkit 与 NGUI 在边界外，不因本原型通过扩大支持文案。

## Decisions

- 写回与观察仍是两个可协商产品能力，但不能由两个 DLL 分别占有相同 JIT hook。Observe-only 原型尚未发布，
  因此直接收敛为一个 `RetainedObject` Standard UI Adapter，不保留旧 package 名或兼容分支。
- 目标对象内当前显示的译文不能当作下一次 source；每个对象必须保留“最新业务原文”和“最后应用代次”。
- 初始写回、代次更新和停用恢复都只能从已验证的 Unity 主线程 dispatch 执行；foreign controller thread
  只提交事务并有界等待。
- 停用恢复失败触发 No-Go；“停止后仅 pass-through、旧译文仍留在界面”不满足实时写回合同。
- Runtime package 不卸载的现状不能代替资源清理；GC handle、pending string 和对象状态必须显式释放。
- 写回事务继续调用已经验证的 original setter trampoline；实验性地在主线程 managed dispatch 内重入
  `mono_runtime_invoke` 会阻塞，已撤回。标准 TMP/uGUI 属性 setter 之外的自定义/抛异常 setter 不在承诺内。

## Verification

- `cargo test -p glyphshift-adapter-native-abi -p glyphshift-adapter-unity-mono-standard-ui
  -p glyphshift-adapter-unity-mono-standard-ui-native`：32 passed（3 ABI + 12 state + 17 native）。
- 三个相关 package 的 `cargo clippy --all-targets -- -D warnings`：通过。
- 确定性可调用 Mono 宿主：初始写回、动态 setter、去重、非法 UTF-16 放行、generation 2 热更新、正常停用
  恢复、停用后透传均通过；另一次有界超时证明三秒后返回失败，下一次主线程 dispatch 延迟恢复并清理。
- 正式 Controller → Target Runtime → native Adapter 链在两个跨代 Standard UI 正例均完成首代可见替换、
  同进程 generation 2 可见更新、停用后可见原文恢复与正常退出；分别捕获 10 / 15 条唯一原文。
- NGUI-only 同 Mono 后端负例继续在 live-object ACK 阶段拒绝并正常退出。
- 未运行全仓测试；所有真实输入、日志、截图和原文仍仅位于 `local-test/`。

## Boundaries

- 外部程序、日志、截图、进程信息、真实 Dictionary 与可见证据只保存在 `local-test/`。
- 不覆盖 IL2CPP、UI Toolkit、NGUI、TextMesh、Sprite/texture、自研 Mesh、TMP `SetText(...)` 或字符数组入口。
- 原型通过前不新增正式 Catalog、Runtime Bundle、软件支持声明或兼容旧格式分支。
