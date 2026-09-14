# Unity IL2CPP Standard UI TextReplace 原型

Status: In Progress

## Goal

在既有 Windows x64 Unity IL2CPP TMP/uGUI observe-only seam 上建立 `TextReplace` 实现。可见写回必须
调用标准属性 `set_text`，让 Unity 自己完成 dirty/layout/repaint；不得直接改 `m_text` / `m_Text` 字段，
不得依赖私有 `MethodInfo` 布局、逐游戏 RVA、符号或机器码扫描。原型必须支持附加前已有对象、运行时业务
文字变化、同进程 Dictionary 新代次和停用恢复最新业务原文。

## Delivery

- [x] 将 Mono 已验证的“最新业务原文 / 当前译文 / Dictionary generation / collected / restore”状态机提升为
  `unity-standard-ui` 后端无关深模块；Mono 保留兼容名称，不改变现有生产行为。
- [x] 扩展 IL2CPP runtime gate，只加入实际写回必需的公开导出：setter method lookup/runtime invoke、UTF-16
  string、GC handle 与异常格式化；缺任一入口均在安装写回前 fail-closed。
- [x] 用公开 metadata 同时定位 TMP/uGUI 文本字段与 `set_text(string)`；snapshot 继续只读字段，所有可见修改
  只经 setter invoke。
- [x] 建立 Windows GUI 线程 dispatch：从当前进程顶层窗口取得窗口线程，以短生命周期 `WH_GETMESSAGE`
  hook + 标记消息把写回事务送到窗口所属线程；执行前以 `il2cpp_thread_current()` 非空验证该 GUI 线程已附加
  到 IL2CPP runtime。无稳定窗口、
  dispatch 超时、异常或重入均 fail-open。
- [x] 对 live object 使用弱 GC handle 跟踪，对一次 setter 调用的 replacement string 使用强 handle；对象已回收
  时删除状态，不把已显示译文重新当业务 source。
- [x] 用确定性 fake/runtime contract 覆盖首代写回、业务 source 更新、generation 2、词典移除恢复、对象回收、
  非法 UTF-16、invoke exception、主线程超时和停用恢复；随后运行相关 package tests/clippy。
- [x] 在两个已授权跨代 IL2CPP 正例完成首代、generation 2 与停用恢复的可见验收，并确认 Adapter 写入的
  中文译文不会重新进入 Observation Stream；Unity 2021 旧 TMP 与 Unity 6 新 TMP 都真实显示兜底字体字形。
- [ ] 再补一个真实目标中的动态业务 source 可见写回验收；确定性合同已经覆盖该状态机，但当前跨代实机复跑
  选择的是稳定静态条目，不能把观察到其他动态文本冒充成动态写回证据。

## Current

代码原型已经完成。Mono 的 retained-text 状态机已提升到共享 `unity-standard-ui`，Mono 通过兼容别名继续
使用同一实现；IL2CPP facade 为 `RetainedObject + TextObserve/TextReplace`。用户已明确授权在外部
Shipping-like 自绘负例仍缺时先进入生产 Catalog/Runtime Bundle，因此该 Adapter 现已作为 Windows x64
候选能力加入正式分发。两个跨代真实 TextReplace 正例的首代、第二代、停用恢复、防译文回流和兜底字体
可见复跑已经补齐；证据等级现在只剩真实动态业务 source 的可见写回和外部同后端自绘负例未完成。

IL2CPP runtime gate 现在按 negotiated feature 分层：原有 `TextObserve` 继续只要求历史 17 个公开导出和
text field，不因新增写回能力收紧；只有请求 `TextReplace` 才额外要求 8 个公开写回导出和标准
`set_text(string)` metadata。写回通过 `il2cpp_string_new_utf16`、强 GC handle 和 `il2cpp_runtime_invoke`
完成。Unity 公开的 IL2CPP codegen 示例对引用类型参数也是直接把 managed object pointer 放进
`void* params[]`，与当前 setter 参数布局一致。

对象身份使用 Adapter 自己的单调 `ManagedObjectId`，弱 GC handle 只作为内部对象寿命引用，避免 handle 数字
释放后复用造成 ID 碰撞。liveness 得到对象指针后会在 GC world 仍停止时立刻转换成 weak handle，避免恢复
GC 后对象移动导致裸指针失效；恢复 GC 后再从 handle 解析当前地址并构造 `ManagedText`。旧 Observe-only 路径
仍使用原有 raw object identity，不创建 GC handle。已回收对象显式通知 observer/writeback。

GC handle FFI 使用 Windows x64 的 pointer-sized `usize` 承载同一个公开 `il2cpp_gchandle_*` API，兼容旧版
32-bit/integer handle 与新版 pointer-sized `Il2CppGCHandle`：旧 ABI 的低 32 位保持不变，新 ABI 则完整保留
64 位，不需要按 Unity 2021/Unity 6 写版本分支。

主线程 dispatch 已实现并通过真实 Win32 合成测试：枚举当前进程可见顶层窗口，只接受唯一 GUI thread，安装
短生命周期 `WH_GETMESSAGE` hook 后发送带 generation 的 `WM_NULL`，callback 实际运行在线程所属窗口的 GUI
thread；激活 TextReplace 前再要求该 callback 上 `il2cpp_thread_current` 非空。多个不同可见 GUI thread、
无窗口、hook/dispatch 超时都会拒绝 TextReplace，不做逐游戏猜测。

native lifecycle 已接入首代写回、周期 snapshot 动态业务 source、Dictionary generation probe/refresh、显式
`request_refresh` 和停用恢复。写回事务只有全部 setter invoke 成功才提交状态；失败时先尝试恢复业务原文，
恢复也失败则保留完整 latest-source 状态，让后续 stop/activation 可以再次恢复，避免清掉唯一恢复依据。

Session 生命周期现在以短事务独占 `ActiveSession`，只在拿取/归还事务时持有全局 `SESSION` mutex；Host
`decide_utf16`、IL2CPP snapshot、`runtime_invoke(set_text)`、恢复和 GC handle 释放均在全局锁之外执行。
`request_refresh` 只原子置位，不在 Runtime 控制线程同步重绘；observer 消费请求后再把写回事务送到已验证的
GUI thread，因此 Host callback 或 managed setter 内的重入 refresh 也只会延后执行。Host decision 与 managed
setter 都受 Session transaction ownership 保护；若重入 `deactivate`，该次停用返回失败并保持 Adapter
仍为 active，只取消尚未开始的写回。正常停用先取得独占 deactivation gate，恢复成功后才发布 inactive、
停止 observer 并清理 handle；恢复失败会撤销 gate 并继续保持可重试的 active 状态，因此 Host 与 Adapter
不会出现一边仍记为 Active、另一边已自行清掉 Session 的生命周期分裂。

## Next

- 选现有已授权 IL2CPP 正例中的一个真实动态业务 source，完成“业务原文变化 -> 新译文 -> 停用恢复最新业务
  原文”的可见验收；继续只走公开 metadata/runtime invoke 与已验证 GUI main-thread dispatch。
- 外部 repeatable Shipping-like IL2CPP custom-drawn text negative 仍需补齐，用于证明同后端可靠拒绝；它不再
  阻止当前 Catalog/Runtime Bundle 分发，但在补齐前仍保留证据等级限制。

## Decisions

- 可见写回只允许标准 `set_text` 语义。字段读取可以用于 observation/source snapshot，字段直接赋值不能作为
  `TextReplace`，因为它不能保证 TMP/uGUI 的 dirty、layout 与 repaint 语义。
- `il2cpp_runtime_invoke` 只解决“怎么调用 managed method”，不解决 Unity 对象线程归属；foreign observer/
  controller thread 永不直接修改 TMP/uGUI 对象。
- Windows 消息调度属于通用 OS seam，不按游戏、进程名或版本分支；若某目标没有可证明的 Unity GUI 线程，
  Adapter 必须拒绝 `TextReplace`，不能回退到逐游戏技巧。
- snapshot 观察到的当前字段如果等于 Adapter 上次写入译文，不能覆盖保存的业务 source；只有确认来自业务更新的
  新值才推进 source。必要时先比较 last-applied replacement，再决定是否接受 snapshot。
- 外部 Shipping-like IL2CPP 自绘负例仍是已验证支持的证据门槛。当前生产接入来自用户明确的发布决定，
  不等价于该证据已经完成。

## Verification

- `glyphshift-adapter-unity-il2cpp-standard-ui-native`: 30 tests passed，包含 Observe-only 17-export 兼容合同、
  TextReplace 额外 8-export 逐项缺失、setter metadata、legacy raw object identity、snapshot/liveness、invoke
  exception、主线程超时、deactivation gate 重入/重叠合同、自身译文不回流，以及真实 Win32
  `WH_GETMESSAGE + WM_NULL` 合成窗口线程派发。
- `glyphshift-adapter-unity-standard-ui`: 16 tests passed；覆盖 snapshot 不把自身译文当 source、动态业务 source、
  generation refresh、词典移除恢复、collection 和停用恢复。
- Desktop backend 定向合同确认：无显式 workflow 字体策略时，目标 locale 的语言兜底字体会编译成
  `DictionaryMatches`；核心 workflow 合同确认未翻译 Observation 不会被兜底字体覆盖。
- `glyphshift-adapter-unity-il2cpp-standard-ui`: 1 test passed，descriptor 保持 unpublished retained-object 原型语义。
- Mono 回归：facade 1 test、native 11 tests、shipping-like runtime 6 tests 全部通过；既有 production adapter
  调用面通过 `UnityMonoWriteback` 兼容别名保持不变。最终 IL2CPP native Clippy 通过；Mono 仅出现
  `vendor/retour` 既有 calling-convention warning，本次代码没有新增项目 warning。
- Release Runtime Bundle 构建与 loader 验证通过，共 14 个 Adapter；`windows.unity.il2cpp.standard-ui` 只存在于
  x64 主清单，x86 附加架构保持不包含该 Adapter。Desktop Catalog、中文/英文展示与官方 Unity IL2CPP 文档
  入口已接通；前端生产构建通过。
- Unity 2021 LTS 真实正例使用旧 TMP 路线，从实际字体文件创建 `UnityEngine.Font` 后再生成 TMP font asset；
  首代与第二代中文均真实可见，停用恢复原文，6 条 Observation 始终只保留业务原文，正常退出。
- Unity 6 真实正例使用新版 TMP 的 system-family/style 动态字体路线；首代与第二代中文均真实可见，停用恢复
  原文，59 条 Observation 始终只保留业务原文，正常退出。两条跨代路径都通过同一个通用 IL2CPP Standard UI
  Adapter，没有软件、进程名或逐版本专属分支。
- 本 Slice 继续保持 `In Progress`，剩余实证项是动态业务 source 的可见写回与外部 Shipping-like 自绘负例；
  这两个缺口不否定已经完成的跨代静态条目实时翻译与字体兜底验证。

## Boundaries

- Windows x64、Unity IL2CPP、TMP/uGUI 标准 `text` 属性。
- 不覆盖 UI Toolkit、NGUI、TextMesh、Sprite/texture/自研 Mesh、TMP `SetText(...)` 多重格式化入口。
- 不安装 Unity、不启动 archive UIA/OCR、不新增软件专属 Adapter；生产接入只复用这一通用 IL2CPP Standard UI
  Adapter，并保持 Windows x64 与标准 TMP/uGUI `text` 属性边界。
- 真实路径、PID、窗口标题、截图、日志、原始 Observation 与目标副本只允许留在 `local-test/`。
