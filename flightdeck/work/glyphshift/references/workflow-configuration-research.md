# 工作流配置模型调研

调研日期：2026-08-02

## 问题

Glyphshift 是否应把“同一软件使用不同词典、字体和功能开关”建模为工作流；工作流是否可以
同时包含多个软件；软件、词典、字体配置和 Runtime 状态应如何分离。

## 外部产品与系统事实

### 保存配置与复合配置是两个层次

VS Code 把 launch configuration 定义为可保存、可选择的启动配方，并用 compound
configuration 同时启动多个具名 configuration。JetBrains 也先要求每个进程拥有独立
run/debug configuration，再由 Compound configuration 组合并行启动多个配置。
[VS Code 官方文档](https://code.visualstudio.com/docs/debugtest/debugging-configuration#_compound-launch-configurations)，
[JetBrains 官方文档](https://www.jetbrains.com/help/idea/run-debug-multiple.html)

对 Glyphshift 的启示：一个工作流可以直接包含多个具名 Target Intent；但首版不必再暴露
一层“子配置库”。只有确认相同 Target Intent 会被多个工作流反复复用时，才提升为独立
Profile 实体，避免一开始产生四级导航。

### 期望配置与实际状态必须分开

Kubernetes 对象把用户提供的 `spec` 作为 desired state，把系统更新的 `status` 作为
current state，并由控制循环持续缩小二者差异。这说明“已启用但目标软件未启动”和“实际
已经 Hook 生效”必须是两个不同事实。
[Kubernetes 官方文档](https://kubernetes.io/docs/concepts/overview/working-with-objects/#object-spec-and-status)

对 Glyphshift 的启示：工作流的启用状态是持久期望；运行中、等待软件、部分生效和失败是
Runtime Status。UI 不能因为开关打开就显示运行中，也不能因为软件关闭就删除工作流期望。

### 可复用内容资产不应复制进每套配置

OBS 明确把输出设置保存在 Profile，把场景与来源保存在 Scene Collection；两者分别管理、
分别切换，而不是把所有内容复制到一个对象里。Profile 还提供新建、复制、重命名、删除与
导入导出等标准管理动作。
[OBS Profiles](https://obsproject.com/kb/profiles)，
[OBS Scene Collections](https://obsproject.com/kb/scene-collections)

对 Glyphshift 的启示：Dictionary 应是独立资产，Workflow 只保存引用、启用状态和优先级。
复制工作流不复制词典内容；共享词典修改可确定地影响所有引用它的工作流。

### 翻译工具已经证明“专用覆盖 + 继承”和字体独立配置有价值

LunaTranslator 支持游戏专用 HOOK 设置覆盖默认设置；其游戏专用翻译优化可以同时继承全局
词典，也可以只使用游戏专用词典。内嵌翻译又把“修改游戏字体”作为独立设置，而不是词典
条目的一部分。
[LunaTranslator HOOK 设置](https://docs.lunatranslator.org/en/hooksettings.html)，
[翻译优化](https://docs.lunatranslator.org/zh/transoptimi.html)，
[内嵌翻译](https://docs.lunatranslator.org/zh/embedtranslate.html)

对 Glyphshift 的启示：多个词典需要明确的组合顺序；Font Policy 与 Dictionary 必须正交。
LunaTranslator 仍主要按游戏保存一套专用设置，不能直接满足 Glyphshift 同一软件保存多套
可切换组合的需求，因此工作流应位于 Software 之上。

## 当前 V2 事实

- GDI Native Adapter 声明并执行 `FontSubstitute`，通过临时创建并选入新 `LOGFONTW`
  字体实现。见
  [GDI Adapter](../../../../crates/adapters/implementations/native/gdi-native/src/lib.rs#L25)及
  [Font-only 合同](../../../../crates/adapters/platform/native-host/tests/native_gdi_activation_contract.rs#L87)。
- GDI+ Native Adapter 同样声明 `FontSubstitute`，保留原字号、样式与单位后创建替代字体。
  见 [GDI+ Adapter](../../../../crates/adapters/implementations/native/gdiplus-native/src/lib.rs#L18)及
  [Font-only 合同](../../../../crates/adapters/platform/native-host/tests/native_gdiplus_activation_contract.rs#L89)。
- Desktop Backend 的 `FontArtifact` 只有一个 `family`，读取后应用到软件全部 Location；
  当前没有 GUI 写入接口。见
  [Desktop Backend](../../../../crates/product/desktop-backend/src/lib.rs#L393)。
- 前端和 Backend 都以 Software ID 直接索引翻译集合，仍是隐式一软件一 Catalog。
  见 [桌面模型](../../../../apps/glyphshift-desktop/src/model.ts#L74)和
  [Backend Snapshot](../../../../crates/product/desktop-backend/src/lib.rs#L198)。
- Runtime Publication 已天然包含一份完整 Translation Snapshot 与 Font Policy；新的
  Workflow Composition 可以在 Desktop Runtime 之前生成 Publication，不需要让 Native
  Adapter 理解工作流。见
  [Runtime Contract](../../../../crates/runtime/contract/src/lib.rs#L71)。
- Desktop Runtime Pool 已按 Software 保存 requested Feature，并能在目标重启后 reconcile；
  它缺少的是工作流产生的完整 Target Intent，而不是另一套注入协议。见
  [Desktop Runtime Pool](../../../../crates/runtime/desktop/src/lib.rs#L331)。

## 三种候选方案

| 方案 | 形状 | 优点 | 问题 | 结论 |
|---|---|---|---|---|
| 软件中心 | Software 直接拥有词典、字体和开关 | 最少实体 | 同一软件多方案会互相覆盖；无法自然表达多软件组合 | 拒绝 |
| 工作流直接包含 Target Intent | Workflow 内每个软件保存词典引用、字体策略与 Feature | 完整覆盖当前需求；三页即可；编译结果直接进入 Runtime | Target Intent 暂不能跨工作流复用 | 首版采用 |
| Profile + Compound Workflow | 独立单软件 Profile，再由 Workflow 引用多个 Profile | 最大复用，接近 IDE Compound 配置 | 多一个用户实体、页面和冲突来源；当前没有复用证据 | 延后 |

## 推荐结论

采用“工作流直接包含 Target Intent”。顶栏为“工作流 / 软件 / 词典”。

- Software 只回答“目标是谁”。
- Dictionary 只回答“翻译内容是什么”。
- Workflow Target 回答“这个软件在本方案中启用哪些词典、文字和字体”。
- Workflow Activation 回答“用户现在希望哪些方案持续生效”。
- Runtime Status 回答“目标进程实际上达成了什么”。

一个软件同一时间最多被一个已启用工作流拥有。若用户启用重叠工作流，必须明确替换旧方案；
不允许静默合并。需要同时使用词典 1 与词典 2 时，应在一个 Workflow Target 中按可见顺序
绑定两者。

这是基于上述资料与当前 V2 接缝得出的设计推论，不是外部产品的原样复制。
