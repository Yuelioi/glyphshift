# Work Context

## 保持不变的核心

- Dictionary 保持纯 `source + translation`；探针证据、字体、平台、技术和运行路由不进入 Entry。
- Workflow Target 继续组合 Software、Adapter Plan、Dictionary 和内联 Font Policy。
- Capability Adapter 是观察或写回能力的执行单元；Platform 是兼容事实，Technology 只用于分类。
- 最小 `TextObservation`、Adapter Descriptor 和完整 Runtime Publication 是稳定 Interface；平台富信息
  只能进入可选且有界的证据 envelope。
- 所有失败路径 fail-open；Core、Desktop 和 GUI 不按软件品牌、可执行文件名或 Adapter ID 分支。

## 候选升级方向

### Process Family

- Software Extension 以授权根可执行文件和显式 `descendant_executables` 描述进程关系；Controller
  只接纳当前进程树中命中 allowlist 的后代，不从窗口标题、安装目录邻近或品牌名称猜测成员。
- Controller 内部以 PID 与进程创建时间识别一次进程实例，对外分配跨 inventory 稳定且永不复用的
  opaque token；PID 和创建时间都不跨 Controller seam。
- 已接纳成员在根先退出后可继续存活，并可授权自己新产生且命中 allowlist 的后代；成员退出会删除
  自己的 target/runtime 状态，不影响仍存活的兄弟实例。
- 无法取得稳定实例身份的单个进程不进入 inventory，但不能让其他可识别实例一起失败。
- PID、窗口标题和本机路径只属于短期 Runtime actual state，不进入持久字典或通用 UI。

### Target Execution

```text
Target Execution
├─ Publication Owner      0..1
└─ Observation Subscriber 0..n
```

- 一个目标实例只维护一套注入、Adapter Host 和 Hook，避免 Workflow 与 Probe 竞争或重复注入。
- Publication Owner 独占文字/字体写回；Observation Subscriber 只能消费有界观察流。
- 只有真实的并发探针、诊断或协作需求证明价值后，才替换当前简单所有权规则。
- 当前 Target Runtime 的 capture 是 activation-time 单一 FileCaptureSink，diagnostics query 是取走式
  批次；在独立 cursor 的 Observation Stream 交付前，Owner/Subscriber 注册表不能提供真实共享，
  Workflow/Probe 软件级互斥继续保留。

### Observation Stream

- Adapter 与同一 Adapter 内的候选文本流是两层身份；候选流属于 Runtime Observation，不是
  Dictionary location，也不能用 PID、绝对地址或窗口标题充当持久语义。
- Adapter 可提供不透明 Stream Fingerprint、样本历史和重匹配证据；Probe 保存 Stream Binding，
  目标重启后解析当前实例并分别报告 matched、degraded 或 unmatched。
- 只有 Adapter 证明指纹跨启动稳定且可诊断后，Workflow 才能引用 Stream Binding；会话内调用点
  不自动升级为 Region Binding。

### Adapter 扩展

- 优先验证 DirectWrite；只有能稳定观察且具备安全写回 Seam 时才声明 TextReplace。
- Console client 与 Console/Terminal host 是独立技术边界；当前四个目标进程内 GDI/GDI+ Adapter
  在隔离 CMD 持续输出中为零信号。Console 输出、Process Family 与 ConPTY-owned session 单独评估，
  不把成功注入或 PE 子系统当作文字覆盖证明。
- UI Automation 优先作为 observe-only Adapter，不把可访问性树等同于实际绘制文字。
- OCR 是无法取得结构化文字时的显式兜底，必须标明延迟、置信度和隐私影响，不进入目标进程热路径。
- Direct2D、Direct3D、OpenGL 和 Vulkan 仅在目标软件证据显示真实缺口后分别立项，不创建万能图形
  Renderer。

### 原生稳定性与布局

- 在现有 reentry/panic fail-open 基础上，单独研究 Windows SEH/VEH、Adapter Watchdog 和熔断；必须
  先确认故障能在宿主退出前被可靠观察。
- `LayoutAdjust` 已是 Feature 候选，只有字体度量、截断或控件布局的真实案例与合成 Host 测试齐备后
  才实现。
- 分裂 text run 由具体 Adapter 做观察重组并携带证据，Decision Engine 不做全局 substring fallback。

### 匹配与辅助创作

- Runtime 默认继续精确匹配。大小写、空白或 Unicode normalization 只有在字典级显式策略和编译期
  冲突诊断同时成立时才考虑。
- 模糊匹配和 AI 语义能力最多用于 Probe 的离线翻译建议，不进入目标进程实时决策。

### Transform Profile

- 捕获正规化、保护/排除和结果修正是独立、有序、版本化且有执行预算的规则资产，由 Probe 或
  Workflow 显式组合；它们不进入 Dictionary Entry。
- 首版只允许声明式、可预览和可测试的 Operator，不允许任意 Python、WASM 或脚本进入 Runtime。

## 明确拒绝

- 富 `TextEvent` 成为包含进程、窗口、控件、坐标、上下文和缓存字段的核心 ABI。
- Dictionary Context、逐条字体、Probe Metadata、Overlay/Draft 双写和通用 Renderer。
- 硬编码 UI Region、窗口句柄、DC 或调用栈 hash 作为稳定 Route。
- 事件溯源重建产品状态，以及没有深度/删除测试依据的 Module 合并。

## 立项门槛

- 每个升级项必须有目标软件证据或合成 Fixture、明确 Interface owner、失败语义和可见验收。
- 新能力应通过新增 Extension、Descriptor 或 Adapter 落地；若必须修改 Core 品牌分支，说明 Seam 不合格。
- 真实软件输入输出全部位于 `target/local-test/`，仓库只保留确定性 harness 和合成 Fixture。
