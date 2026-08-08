# Unity IL2CPP Standard UI Observe-only 原型

Status: Closed

## Goal

直接使用已固定的两个 Windows x64 IL2CPP Standard UI 正例，建立未发布、只读的 TMP/uGUI 观察原型。
原型必须完成后端识别、初始对象枚举、增量 live-object snapshot、Observation 发布与停止清理，但不修改
目标字符串，不进入 Catalog 或 Runtime Bundle，也不声明生产支持。实现复核后已否决依赖私有
`MethodInfo` 布局的 setter detour，动态文字由周期性 snapshot 捕获。

## Delivery

- [x] 把 Mono 与 IL2CPP 共用的 Standard UI 观察状态和生命周期收敛到一个 host-independent 深模块；
  两个后端只在各自 Runtime/ABI Adapter 内保留差异。
- [x] 建立 Windows x64 IL2CPP runtime gate：只接受已加载 `GameAssembly.dll` 与完整必需导出，明确拒绝
  Mono、缺导出、错误架构和无法识别 Standard UI metadata 的目标。
- [x] 在已附加到 IL2CPP 的专用观察线程完成 TMP/uGUI live object 枚举；非空初始 snapshot 到达前
  不返回 Active ACK，且不依赖窗口 hook 或调用 UnityEngine 对象 API。
- [x] 只观察 TMP/uGUI live object 当前 `text` 值及其增量变化；不 detour 属性 setter，不覆盖两次 snapshot
  之间已经消失的瞬时文字、UI Toolkit、NGUI、自研 Mesh、Localization 高层 API 或任何目标字符串写回。
- [x] 停用时先停止并回收观察线程，再停止发布并释放本原型持有的运行时状态；两个真实正例均正常停用
  和退出。
- [x] 先完成纯状态/runtime gate/合成 callback 的定向测试，再分别在两个已授权正例做 observe-only
  attach，确认初始文字和既定动态文字均产生非空 Observation，随后正常停用与退出。

## Current

两个跨代正例已经完成静态门禁与 runtime-only smoke：Unity 2021 LTS 游戏具有每帧更新的
`TextMeshProUGUI.text` 分数；Unity 6 桌面应用具有搜索框与运行时组合 tooltip 的 `TMP_Text.text` 路径。
两包均为 Windows x64 IL2CPP、核心 `il2cpp_*` 导出存在、无 Development Build 标记或已知反作弊制品。

此前把仓库自建负例误设为 observe-only attach 的前置条件，导致主线被本机缺少 Unity Editor 阻塞。
现已纠正：负例只阻止可靠拒绝结论和生产接入，不阻止这两个已授权正例上的只读 Hook 原型。

Mono 原型中真正与后端无关的观察状态和 lifecycle driver 已提取为独立 `unity-standard-ui` 深模块：
统一处理 UTF-16/对象上限、初始 snapshot、增量 snapshot、setter 去重、对象回收与停用；Mono native
模块只保留 Mono ABI、JIT detour 和写回差异。IL2CPP 可以通过同一个 Observer interface 提交周期性
managed observer thread snapshot，不需要复制一套状态机。

未发布的 IL2CPP descriptor 与 native Adapter 已实现。runtime gate 只检查已加载 `GameAssembly.dll` 和
17 个实际使用的公开 `il2cpp_*` 导出，并显式拒绝 Mono、错误平台/架构和缺导出。metadata lookup 跨 image
定位 TMP/uGUI 类及其标准文本字段；对象枚举使用公开 liveness API，文本读取使用 field/string API，
不读取 IL2CPP 私有对象或 `MethodInfo` 布局，也不调用可能被裁剪的 `Resources.FindObjectsOfTypeAll`。
专用观察线程每次 snapshot 前附加到 IL2CPP，停止 GC world 后完成有界枚举和字段复制，再恢复 GC world。

初始和增量 snapshot 已接入共享 Observer。激活必须先取得非空、可解码的初始文字；Target Runtime 在
Adapter ACK 后才进入 Active 的时序由有界 pending 队列吸收，首次被 Host 暂时拒绝的 Observation 会在
下一次 tick 重发。停用先停止并 join 观察线程，再清空 Observer/metadata 状态。

两个跨代正例真实 attach 均已通过。Unity 2021 LTS 正例进入动态场景后捕获 `SCORE` 从 0 更新到
78/156，并捕获 Game Over/Current Score，最终 14 条 Observation；Unity 6 正例捕获完整工作区后，在
搜索框输入 `wall` 得到新的 Observation，最终 72 条。两者均正常停用和退出。

真实样本纠正了三项错误假设：并非所有版本都导出内部 array helper；发布裁剪可能移除
`Resources.FindObjectsOfTypeAll` metadata；停止 GC world 后不能再使用普通进程堆完成 liveness 缓冲
扩容或 Observation 分配。当前实现改用公开 liveness/field/string API，所有输出缓冲在停世界前预分配，
liveness reallocator 使用预先创建的私有无锁 Windows Heap，恢复 GC world 后才构造 `ManagedText`。

## Next

- 本 Slice 无剩余实现；原型继续保持未发布、observe-only，不进入 Catalog/Runtime Bundle。
- 若要生产接入，先取得外部、可重复的 Windows x64 IL2CPP Shipping-like 自绘负例；仓库 source-only
  fixture 不能替代该证据，也不为此安装 Unity 或开始无界搜索。

## Decisions

- observe-only 原型可以先在两个正例上实施；负例缺失必须在结论中保留，但不能再变成“先安装 Unity”。
- 外部 Shipping-like 自绘负例仍是生产 crate、Catalog、Runtime Bundle 与可靠拒绝声明的硬门槛。
- 初始枚举和增量 snapshot 必须汇入同一个 host-independent Observer interface；调用者不学习 Mono/
  IL2CPP 的对象布局、线程附加、方法定位或调度细节。
- 公共 IL2CPP 导出没有提供稳定的 setter machine-code 地址 interface；本原型不以私有 `MethodInfo`
  布局、逐版本 RVA 或机器码扫描换取精确 setter detour。代价是只承诺 snapshot 周期内仍可见的文字变化。
- 只使用稳定 `il2cpp_*` 导出与 metadata 语义；任何逐游戏 RVA、固定偏移、机器码扫描或私有符号依赖
  立即触发 No-Go。
- 本 Slice 不写回目标字符串，因此不复用 Mono writeback 的 GC handle/恢复合同，也不提前设计 IL2CPP
  字符串所有权。

## Verification

- `unity-standard-ui`、`unity-mono-standard-ui`、`unity-mono-standard-ui-native` 定向测试：27 passed；
  三个 package 的 `cargo clippy --all-targets -- -D warnings` 通过。
- 共享模块没有 Mono/IL2CPP 后端词或 FFI；Mono descriptor、writeback 与 Shipping-like runtime 合同继续
  通过。
- `unity-il2cpp-standard-ui` descriptor 与 native runtime gate/metadata/liveness snapshot/观察线程/激活重试
  的定向测试通过；native package 13 项测试与 `cargo clippy --all-targets -- -D warnings` 通过，DLL 定向
  构建通过。metadata 与 stop-the-world snapshot 已拆成独立模块，各源码文件均低于 500 行；原型没有进入
  Catalog、Bundle 或前台。
- Unity 2021 LTS IL2CPP 真实 attach 取得 14 条 Observation，覆盖新场景对象和动态分数；Unity 6 真实
  attach 取得 72 条，覆盖搜索输入动态更新。两者均正常停用和退出。
- 正例包沿用先前授权；本次 attach 的原始路径、截图、日志、Observation 与进程数据只在本机忽略目录，
  本页仅记录可移植结论与计数。

## Boundaries

- 不安装 Unity，不构建仓库自建负例，不开始第五轮公开候选搜索。
- 不进入生产 Catalog/Runtime Bundle，不在桌面 UI 暴露未验证技术项。
- 不把成功注入、metadata 类型存在或单个正例通过冒充完整 Standard UI 覆盖。
