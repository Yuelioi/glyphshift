# MonoGame 游戏文字适配方案

日期：2026-09-07。当前结果是源码研究与目标程序集的只读 IL 检查；没有附加 Profiler、执行游戏方法、捕获动态文本或验证可见译文。

后续进展：现已完成 [CoreCLR 合成接入实验](../slices/coreclr-late-attach.md)。通过新增内存桥接类型/PInvoke 直接进入 Native ABI，无额外托管助手程序集；本页其余内容仍是整体方案，尚未验证真实游戏和 MonoGame 画面。

## 判断

### 无损与驻留标识边界

用户强调卸载后恢复，并确认不需要单独扩展插件。因此下面 CoreCLR/ReJIT 路线作为 Runtime Bundle 内置 Adapter 候选；通过 `ProcessResidentAfterDeactivate` 标识提示组件可能驻留。

- 停用：停止捕获与替换，原文行为恢复；失败需明确报告，不能只返回成功。
- 进程内卸载：不再有活跃 Hook、回调或组件引用；驻留组件存在时不得报告完整卸载。此维度与行为恢复分开验证，不推定已有 Adapter 全部满足。
- 扩展移除：禁止未来加载，卸除自身安装内容；若当前游戏仍持有组件，显示需退出并重新启动游戏才能完成清理。必须验证新进程没有重新加载扩展，不能只根据配置已删除判断成功。
- 不可可靠恢复游戏行为、破坏文件或存档的实现，不因“扩展”身份获得例外。

建议保持同一 Adapter 技术契约，用可选分发和明确生命周期能力区分。现有 ExtensionPackage 支持 Software 包及代码 artifact，但这不证明安装、移除、驻留组件和重启提示已有完整实现；需要另行设计接线与验证。此次没有修改公共 ABI、产品目录或扩展代码。

建议沿现有 Adapter 模式，增加 **CoreCLR 接入底座 + MonoGame 标准文字 Adapter**。以星露谷检验标准入口覆盖，再单独验证自绘文字入口的通用发现机制。无需 SMAPI 或游戏 Mod，但并非已有组件接上线即可完成。

不能承诺一个 DrawString Hook 完整翻译星露谷：本次 IL 检查已证明对话绘制路径会进入游戏自绘字符渲染器，直接提交纹理精灵。若不做游戏专属绑定，完整对话覆盖取决于自绘入口发现与布局处理是否能通过验证；这是本方案最大的未决问题。

## 目标程序集的静态证据

检查使用 PEReader 解码托管方法体、解析调用 token，没有加载执行游戏程序集。机器信息、参数记录与扫描脚本只存在 local-test/。

| 检查范围 | 结果 | 可推导的边界 |
| --- | --- | --- |
| 运行技术 | CoreCLR、MonoGame；既有 Unity Mono Adapter 依赖 mono_get_root_domain 等 Mono 导出 | 需要新的 CoreCLR 接入，不复用 Mono embedding API |
| Utility.drawTextWithShadow | 两个重载调用 SpriteBatch.DrawString | 标准文字 Adapter 有真实静态调用来源；尚未确认当前画面命中 |
| IClickableMenu.drawHoverText | 调用 DrawString、MeasureString 和其他文字包装方法 | 提示文字值得优先验证，不能从方法名推出全部内容覆盖 |
| Game1.parseText | 使用 MeasureString，并处理字符串与换行 | 绘制入口可能晚于上层排版，拦截测量也未必恢复完整段落 |
| DialogueBox.draw | 调用 SpriteText 的绘制、宽度与高度方法 | 对话不能只依赖标准 DrawString |
| SpriteText.drawString | 调用 String.get_Chars/get_Length、字符度量、SpriteBatch.Draw；无直接 DrawString 调用 | 存在绕过标准文字 API 的自绘字形路径 |
| 全程序集结构扫描 | 不按游戏类名/方法名筛选，搜索同时调用 Draw、String.get_Chars/get_Length 的方法，得到 6 个候选，包含自绘文字方法和其他绘制方法 | 静态规则能缩小范围但会误报；不能直接授权写回，也未证明规则无漏报 |

上述证据是本次检查的便携结论，不是上游文档对游戏覆盖率的声明。扫描只统计直接调用，不等于完整调用图或字符串数据流证明。

## 与现有 Adapter 模式衔接

建议结构如下，均为待实现名称，不增加生产 Catalog 条目：

```text
Controller 的目标授权与接入
  → CoreCLR Profiler / ReJIT 支持组件
  → MonoGame TargetProcess Adapter + 托管桥接组件
  → NativeRuntimeHostV1.decide_utf16
  → 既有 Runtime 捕获 / Kernel 词典决策
  → 返回当代译文，在原绘制调用中使用
```

- 描述包沿用 `AdapterDescriptor`，标准绘制模型为 `InlineRender + TargetProcess`。原型先只声明 TextObserve；完成真实替换验收后再声明 TextReplace。
- Native 入口沿用 `glyphshift_adapter_entry_v1`、negotiate/activate/deactivate/request_refresh。CoreCLR Profiler 是接入支持组件，不独立冒充文字 Adapter；AttachProfiler 的控制面位置需原型确定，不在激活回调里未经验证地阻塞附加自身。
- 托管桥接把有界字符串复制或在单次调用内固定后交给原生接口，译文构造为新托管字符串；不保留裸托管指针、不修改游戏原字符串或 StringBuilder。异常、未知签名、缺字等返回原路径。
- 原生组件、Profiler、托管桥接必须作为同一构建的受验证依赖交付。当前 Registry 的 TargetProcess 绑定指向单个库，需核对 bundle 对伴随程序集的哈希和依赖校验；不得从游戏目录随意加载助手代码。
- 现有 ABI 足以研究基本原文/译文与 generation；它不传绘制位置、字体对象、调用点或可见区域，Runtime 还将 surface 固定为 native.surface。不能宣称已有接口解决上下文消歧。首个实验用唯一合成原文；如正式能力需要上下文，再设计带版本的扩展。

本地依据：[Native ABI](../../../../crates/adapters/platform/native-abi/src/lib.rs)、[Native Loader](../../../../crates/adapters/platform/native-host/src/lib.rs)、[Registry](../../../../crates/adapters/platform/registry/src/lib.rs)、[Runtime 决策回调](../../../../crates/runtime/targets/runtime/src/lib.rs)、[raylib 描述包参考](../../../../crates/adapters/implementations/framework/raylib/src/lib.rs)、[Unity Mono runtime gate](../../../../crates/adapters/implementations/framework/unity-mono-standard-ui-native/src/lib.rs)。

## 接入选择

优先验证官方 diagnostics AttachProfiler + ReJIT，在已运行 CoreCLR 内对选定托管方法插入调用。Profiler 的附加、helper 引导、程序集上下文、既有内联与停用行为仍需合成宿主实验；官方有 API 不等于当前游戏可直接成功附加。[CoreCLR 接入研究](coreclr-attachment.md)

hostfxr 是进入目标进程后可研究的助手装载机制，不是远程注入接口；startup hook 需要重启和启动配置，不能声称支持当前运行中目标。Harmony 可作为托管补丁工具的比较项，但不解决第一步如何进入既有 CoreCLR，也不要求借助 SMAPI。

停用必须先使桥接直接透传，再清空回调引用、等待在途调用退出，最后安全处理 ReJIT 恢复。顺序与锁关系需要合同验证，不能持有 Runtime 决策锁等待同一回调排空。Profiler/助手可能需驻留到进程结束，不能把“恢复原文”描述为“所有 DLL 已卸载”；现有 loader 的 Library 生命周期不可直接套用于仍被 JIT 代码引用的组件。

## 两类文字如何处理

### 标准 MonoGame 字体

Hook 目标是经签名核对的 SpriteBatch.DrawString（string/StringBuilder 重载）与 SpriteFont.MeasureString。上游源码说明 DrawString 会把文本逐字转成图集绘制，MeasureString 使用字体度量；这两个位置适合在丢失文本语义前介入。[SpriteBatch 源码](https://raw.githubusercontent.com/MonoGame/MonoGame/develop/MonoGame.Framework/Graphics/SpriteBatch.cs)、[SpriteFont 源码](https://raw.githubusercontent.com/MonoGame/MonoGame/develop/MonoGame.Framework/Graphics/SpriteFont.cs)

实现需处理包装重载的重入，避免同一字符串重复翻译；测量与绘制使用一致的译文、字体和代次。缺失译文字形时，需要 Adapter 自有字体图集与相应度量，GPU 资源在所属绘制线程创建/回收。原字体不可原地改写。上层已拆行、缓存布局或跨帧测量仍是额外边界，不能简单全局替换每次 MeasureString。

绘制热路径只查询本地结果，不调用远程翻译。现有 decide_utf16 内部会锁 Runtime 状态并分配，需先测性能；缓存须有有界容量与明确的 generation/停用失效机制。仅比较上次决策返回的 generation 不足以主动发现新版本，原型可先按帧有界复核，正式再决定是否补充无锁快照通知。

### 绕过 DrawString 的自绘文字

应把“托管渲染方法发现”作为独立研究能力：从 SpriteBatch.Draw 调用关系向上筛选仍持有字符串的候选，再观察该参数是否驱动字符选择和字形提交。静态字符循环、形参类型、纹理使用只能提供线索，动态关联也不能单独证明任意参数都可安全改写。

先输出只读候选和命中证据；只有能验证参数语义、绘制范围、测量关联与恢复的窄规则才允许替换。不能全局 Hook String.get_Chars，也不能抑制候选方法内所有 Draw——其中可能同时绘制背景、卷轴、图标。

对于星露谷的对话，还需解决字形来源、行宽/高度、显示字符进度及分页。自动识别这些语义目前没有证据。若最终依赖 SpriteText 类名、游戏方法签名或专用分页逻辑，必须标为游戏专属绑定，不符合本次通用实现验收；将它移入配置文件并不会自动变成通用能力。

## 最小实验与停止条件

1. **接入实验**：确定性 CoreCLR 宿主先调用文字函数使之 JIT，再从 Glyphshift 控制面附加；证明原文观测、首代替换、第二代更新、停用原文恢复。验证 helper 装载、GC、在途调用、重复连接和目标退出。失败则先解决接入，不能跳到真实游戏做模糊试错。
2. **MonoGame 实验**：合成宿主覆盖 string/StringBuilder、包装重载、MeasureString、阴影重复绘制、缺字、裁剪及帧时间；用明确像素断言验证替换。通过后再在星露谷中分别观察提示与对话路径，报告部分覆盖。
3. **自绘发现实验**：合成两个类名和结构不同的逐字纹理渲染器，加一个非文字精灵负例，不以游戏名称筛选。验证发现和观测后再尝试替换；发现误报、测量错配或背景抑制即保持只读。
4. **真实支持晋级**：星露谷对应场景取得动态命中、可见译文、更新和恢复，再用另一个独立 MonoGame 游戏验证复用。未完成自绘对话验证前，不能标记星露谷完整支持；未完成跨目标验证前，不能标记通用游戏支持。

本次只完成方案研究和静态入口检查，没有创建新生产 Adapter，没有运行归档 UIA/OCR，也没有构建或启动桌面应用。
