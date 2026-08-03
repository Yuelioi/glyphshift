# LunaTranslator 产品与运行时调研

调研时间：2026-08-03
上游基线：LunaTranslator 官方仓库
[`aaef4d0cff03e54654a5428716fa3a75a659b84a`](https://github.com/HIllya51/LunaTranslator/tree/aaef4d0cff03e54654a5428716fa3a75a659b84a)

## 结论

LunaTranslator 最值得 GlyphShift 学习的不是功能数量，而是三条经过长期使用验证的产品规律：

1. 软件/游戏是持续工作的上下文锚点；重新进入同一目标时，应恢复它的捕获选择、历史和专用配置。
2. Hook 技术本身不能替代“选择哪一条文本流”。用户先看到候选流和样本文本，再选择真正有用的流，选择结果随后持久化并自动重匹配。
3. 捕获、清洗、翻译优化、显示与写回是不同阶段；复杂能力可以组合，但不应全部进入翻译字典。

这三条与 GlyphShift 当前的 Software / Workflow / Probe / Dictionary / Adapter / Runtime 分层方向一致，
同时暴露出一个明确缺口：当前 Probe 只能按 Adapter 保存证据，尚没有稳定的 **Observation Stream
Identity / Binding**。这项能力需要扩展 Runtime 协议和 Adapter 证据，不应伪装成 Dictionary
`location`，也不适合在当前产品交付 Work 中仓促实现。

路由结论：

- **当前 Work**：只增加 Probe 的 Adapter 筛选与重启/暂停/恢复验收，不扩大输入输出类型。
- **`runtime-capability-roadmap`**：优先加入 Observation Stream Binding、Process Family、Transform
  Profile；OCR、剪贴板、覆盖层作为后续可选 Adapter。
- **明确不采纳**：大而全的单体“软件专用设置”、任意 Python 插件、渲染 Hook 内等待在线翻译、
  用 Hook 流身份污染 Dictionary、用配置目录复制代替产品级 Workspace。

## 1. 资料边界与判读规则

本文只使用 LunaTranslator 官方文档、官方 GitHub 仓库、官方 README 与源码。带有“源码事实”的
段落描述可直接从实现确认的行为；带有“判断”的段落是基于事实对 GlyphShift 的产品路由，不表示
LunaTranslator 官方主张。

GlyphShift 的比较基线是当前已冻结的[Work 稳定约束](../context.md)与
[产品领域语言](../../../../CONTEXT.md)：Dictionary 保持纯 `source + translation`；Workflow Target
组合 Adapter、Dictionary 与 Font Policy；Probe Run 绑定一个软件、一个 Dictionary 和一组 Adapter；
Runtime 采用离线 Snapshot 与可验证 ACK。

## 2. 用户信息架构与持续工作流

### 2.1 软件目标是上下文锚点

**官方事实。** 在 HOOK 模式中，用户先选择运行中的进程；LunaTranslator 随后把目标加入软件库、
注入进程并打开候选文本选择窗口。拖入可执行文件则会完成“加入软件、转区启动、自动 Hook”这一
连续动作。也就是说，管理对象与当次运行会话由同一个游戏身份串起来，而不是每次从空白捕获器
开始。[官方基本用法](https://docs.lunatranslator.org/zh/basicuse.html#hook模式)

**源码事实。** 游戏专用设置被组织为“启动 / HOOK / 语言 / 文本处理 / 翻译优化 / 语音 / 预翻译”
七个页签，并全部以 `gameuid` 读取或修改专用数据。
[源码：游戏设置页签](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/gamemanager/setting.py#L437-L462)

**判断。** LunaTranslator 证明“目标软件是恢复工作的入口”是对的，但它把大量职责聚合进同一个
游戏配置。GlyphShift 应保留现在更清晰的资产分权：Software 只拥有身份与程序绑定，Workflow
拥有运行意图，Dictionary 可跨软件复用，Probe Run 拥有持续采集状态。应借鉴入口连续性，不复制
单体配置对象。

### 2.2 暂停/继续比开始/停止更符合持续作业

**官方事实。** LunaTranslator 的“自动模式”对当前输入源执行暂停/继续：HOOK 暂停读取游戏文本，
OCR 暂停自动识别，剪贴板模式暂停自动读取；手动执行仍可单次读取当前输入源。
[官方工具按钮](https://docs.lunatranslator.org/zh/alltoolbuttons.html#anchor-automodebutton)

**判断。** 这直接验证了 GlyphShift Probe Run 的 `running / paused / ready / interrupted` 方向。当前
Work 不需要再发明新的会话状态，但发布验收必须覆盖：暂停后不再增加 Observation、编辑/搜索/导出
仍可用；应用重启后任务仍存在；恢复操作是显式且结果可诊断的。

### 2.3 工具栏可定制不等于信息架构清晰

**官方事实。** LunaTranslator 允许工具按钮隐藏、换序、分左右中对齐、改色与改图标，并同时提供
游戏设置、选择文本、OCR、历史、游戏管理、编辑翻译记录等大量快捷入口。
[官方工具按钮说明](https://docs.lunatranslator.org/zh/alltoolbuttons.html)

**判断。** 这是高功能密度产品的适应性方案，不适合作为 GlyphShift 顶层导航模板。GlyphShift
应继续使用“Workflow / Software / Dictionary / Probe”稳定管理页，把诊断和少用操作留在详情或
Modal；不要把几十个运行按钮暴露成可自由排列的主界面。

## 3. 软件管理、Hook 选择与历史

### 3.1 “Adapter”与“选中的文本流”不是一回事

**官方事实。** LunaTranslator 注入后先展示若干候选文本行，用户选择符合游戏正文的行才开始
翻译；候选项支持“显示”和（能力存在时）“内嵌”两个独立选择。
[官方基本用法](https://docs.lunatranslator.org/zh/basicuse.html#hook模式)

**源码事实。** Hook 流的运行时键由 Hook Code、Hook Name 与 `ThreadParam(processId, addr, ctx,
ctx2)` 组成；只有出现在 `selectedhook` 的键才进入翻译管线，多选时按选择顺序合并。
[源码：Hook 键与选择集合](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L58-L72)
[源码：只分发已选流并按序合并](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L735-L769)

**源码事实。** 用户改变选择后，程序把所选 Hook 序列化进当前 `gameuid` 的 `hook` 配置；界面还会
立即查询每个流的近期历史文本作为反馈。下次自动启动时读取这组 Hook，并通过 Hook Code/Name 与
Context 的兼容性匹配自动选回，而不是把一次运行的 PID 当作永久语义。
[源码：保存选择并回放近期历史](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/selecthook.py#L932-L962)
[源码：自动读取并匹配已存 Hook](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L399-L420)
[源码：兼容性匹配](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L619-L655)

**判断。** 这是本次调研最重要的发现。LunaTranslator 的候选 Hook 行并不是 `location`，而是可观察、
可采样、可选择、可在下次运行中重新匹配的 **输入流身份**。GlyphShift 未来应建立：

```text
Adapter
  └─ Observation Stream Identity（Adapter 提供的不透明稳定指纹）
       ├─ sample history
       ├─ enabled / ignored
       ├─ match confidence
       └─ persisted Stream Binding
```

Dictionary 仍只保存 `source + translation`。Stream Binding 属于 Probe/Workflow 的捕获范围，不能
重新引入逐词条 `location`，也不能把函数地址、PID 或原始 Hook Code写进 Dictionary。

### 3.2 历史既服务选择，也服务长期翻译记录

**源码事实。** LunaTranslator 一方面由 Hook Host 提供线程历史查询，另一方面为当前游戏创建独立
SQLite 翻译记录，按 source 保存原始文本和各翻译引擎结果；记录写入由队列异步完成。
[源码：每游戏翻译记录文件](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L376-L425)
[源码：SQLite 结构与异步写入](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/textsourcebase.py#L70-L91)
[源码：原文和翻译结果更新](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/textsourcebase.py#L130-L184)

**判断。** GlyphShift 当前 Dictionary + Observation Index 的联合表比“另建一份翻译记录再同步回
词典”更适合字典生产。应保留单一翻译内容资产；未来若增加通用运行历史，也应是只读事件/证据层，
不能成为第二份可编辑译文真相。

## 4. 翻译、字典、预处理与后处理

### 4.1 LunaTranslator 是可排序处理管线

**官方事实。** LunaTranslator 区分：Hook 原文清洗、翻译前替换、专有名词翻译、翻译结果修正、
自定义优化；文本清洗支持多种去重、过滤、Unicode 正规化、字符串/正则替换，并允许组合和调整
顺序。[官方文本处理](https://docs.lunatranslator.org/zh/textprocess.html)
[官方翻译优化](https://docs.lunatranslator.org/zh/transoptimi.html)

**源码事实。** 游戏可取消“跟随默认”，保存自己的处理方法列表、顺序与参数；增删和上下移动实际
修改当前 `gameuid` 的 `save_text_process_info.rank` 与私有配置。
[源码：游戏专用处理链排序](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/gamemanager/setting.py#L1175-L1239)

**源码事实。** 主管线先执行 `POSTSOLVE`，再执行翻译前优化，调用一个或多个翻译器，最后执行翻译
后处理；原文与结果分别进入显示和历史记录。
[源码：输入处理与历史](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/LunaTranslator.py#L582-L668)
[源码：翻译前后处理](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/LunaTranslator.py#L668-L758)

**判断。** GlyphShift 应采纳“有序处理阶段”，但不采纳 LunaTranslator 把多类规则都称为翻译优化
词典的表面模型。Roadmap 应新增独立、版本化、可测试的 `Transform Profile`：

- capture normalization：去重、Unicode 正规化、去控制字符；
- match protection：排除、保护与白名单；
- result correction：必要的输出修正规则；
- 每条规则声明适用输入类型和执行预算。

它由 Workflow/Probe 组合，Dictionary 继续保持纯映射。任意 Python 脚本不进入首版 Transform
Profile。

### 4.2 全局词典 + 游戏专用词典不是逐词条 Context

**官方事实。** 游戏专用翻译优化可以取消“跟随默认”；启用“继承默认”时，游戏专用词典与全局
词典同时生效。[官方翻译优化：游戏专用设置](https://docs.lunatranslator.org/zh/transoptimi.html#游戏专用翻译优化)

**判断。** 这进一步支持 GlyphShift 的组合模型：作用域在“目标配置选择哪些资产”，而不是每条
Dictionary Entry 重复保存游戏、窗口、字体或 Hook 信息。GlyphShift 已用 Workflow Target 的有序
Dictionary 集合表达组合，因此不需要再增加 Luna 式“跟随默认”布尔继承；显式列表更容易诊断和
复现。

## 5. 字体、显示、覆盖层与嵌入写回

### 5.1 字体属于目标写回策略

**官方事实。** LunaTranslator 只有在某个候选 Hook 支持内嵌时才显示内嵌能力；内嵌出现乱码时，
用户可为游戏启用修改字体、选择字体并调整相对字号。内嵌翻译会在游戏显示函数处暂停、等待翻译、
修改内存再继续，所以慢翻译会造成卡顿，并提供等待时限。
[官方内嵌翻译](https://docs.lunatranslator.org/zh/embedtranslate.html)

**源码事实。** 字体 family、是否修改字体和相对字号保存在游戏的 `embed_setting_private`，运行时
通过 `set_settings_ex` 下发；它们不属于翻译词条。
[源码：游戏专用字体设置](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/gamemanager/setting.py#L1400-L1467)
[源码：向 Hook Runtime 下发字体设置](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L591-L613)

**判断。** 这验证了 GlyphShift 把 Font Policy 放在 Workflow Target、而不是 Dictionary 或独立顶级
资产页的决定。GlyphShift 还应坚持自己的更强语义：`dictionary_matches` 与 `all_observations`
coverage 必须显式，且只有对应 Adapter ACK 后才能宣称字体替换已启用。

### 5.2 外挂覆盖层与原位写回应是两种 Apply Model

**官方事实。** LunaTranslator 的翻译窗口支持鼠标穿透和背景透明；当原位内嵌受编码/字体限制时，
还可以清空游戏原文字并把翻译窗口覆盖在原位置，模拟内嵌效果。
[官方工具按钮：鼠标穿透与透明](https://docs.lunatranslator.org/zh/alltoolbuttons.html#anchor-mousetransbutton)
[官方内嵌翻译：清除游戏内文字](https://docs.lunatranslator.org/zh/embedtranslate.html#内嵌翻译设置)

**判断。** 覆盖层是有效的 fallback，但不是当前 GlyphShift Runtime Bundle 的同义词。若未来加入，
应作为独立 `overlay` / `external-window` Apply Model：拥有窗口绑定、位置跟随、点击穿透和生命周期
ACK；不能把“覆盖层正在显示”冒充 `inline-render` Adapter 已成功替换目标控件。

## 6. HOOK、OCR、剪贴板与其他输入输出模式

### 6.1 LunaTranslator 选择一个主输入源，并允许临时旁路

**源码事实。** 主程序把 OCR、剪贴板、HOOK、文件翻译、语音识别映射为五种 `textsource` 类，
从启用项中选择一个当前输入源实例。
[源码：输入源选择](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/LunaTranslator.py#L1126-L1155)

**官方事实。** 即使当前主源是 HOOK，也可单次读取剪贴板或临时框选 OCR；临时 OCR 完成后会回到
HOOK 自动提取，不要求用户切换整个模式。
[官方工具按钮：读取剪贴板与单次 OCR](https://docs.lunatranslator.org/zh/alltoolbuttons.html#anchor-copy_once)
[官方：HOOK 模式临时使用 OCR](https://docs.lunatranslator.org/zh/gooduse/useocrinhook.html)

**判断。** GlyphShift 不应复制“全局只能选一个输入模式”的核心模型。Adapter Plan 已允许多个技术
并行，更适合桌面软件；但“临时旁路观察”值得 Roadmap 保留，例如在 Hook 漏掉一个弹窗时单次 OCR，
结果进入当前 Probe Run 的证据层而不是静默更改 Workflow。

### 6.2 OCR 的价值在窗口绑定和变更触发，不只是识别引擎

**官方事实。** LunaTranslator OCR 支持周期执行、图像稳定/一致性判定、鼠标键盘触发后等待稳定，
并对 OCR 文本做编辑距离阈值过滤。[官方 OCR 自动化](https://docs.lunatranslator.org/zh/ocrparam.html)

**官方事实。** OCR 可绑定目标窗口，使截图只包含目标窗口、OCR 区域随窗口移动，并把窗口关联到
游戏专用语言、翻译优化和文本处理设置。
[官方 OCR 窗口绑定](https://docs.lunatranslator.org/zh/gooduse/gooduseocr.html)

**判断。** OCR 应进入 `runtime-capability-roadmap`，但拆成三个契约：`Window Capture Controller`、
`OCR Observe-only Adapter`、`Change Trigger Policy`。识别结果只能提升 observe 状态；没有可验证写回
时不能显示“正在翻译目标软件”。

### 6.3 输出流可以开放，但不是 Runtime 真相

**官方事实。** LunaTranslator 的网络服务提供主界面、历史、查词 Web 页面，并通过 WebSocket
持续输出原文和译文，也提供翻译/OCR/词典等 HTTP API。
[官方网络服务](https://docs.lunatranslator.org/zh/apiservice.html)

**判断。** 未来 GlyphShift 可把 Observation/Decision 订阅做成外部只读 Adapter 或诊断协议，但
WebSocket 收到文本不等于目标进程写回成功。Runtime generation、Adapter feature ACK 与外部事件流
必须继续分开。

## 7. 插件、扩展与配置持久化

### 7.1 扩展机制灵活，但依赖同进程 Python 动态导入

**源码事实。** LunaTranslator 按配置名称用 `importlib.import_module` 动态加载翻译优化、翻译器、
TTS、词典和元数据实现；翻译器还可复制为用户配置目录中的 Python 模块并复制对应参数。
[源码：动态加载翻译优化](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/LunaTranslator.py#L413-L438)
[源码：用户目录翻译器模块解析](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/myutils/utils.py#L86-L112)
[源码：复制翻译器模块与配置](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/myutils/config.py#L537-L565)

**判断。** 这种机制适合单体 Python 工具快速扩展，不适合作为 GlyphShift 目标进程 Runtime 的安全
边界。GlyphShift 应继续使用签名、hash、ABI、placement 与 Feature 明确的 Capability Adapter；
文本处理扩展首版只允许有界声明规则，不允许任意 Python、WASM 或脚本进入目标进程。

### 7.2 配置有原子替换与多目录 Profile，但不是领域 Workspace

**源码事实。** 配置保存先写 `.tmp` 再 `os.replace`，并分别保存全局配置、文本处理、翻译修正、
翻译器、OCR 与游戏数据；代码注释明确说明这是为了避免保存时意外退出清空配置。
[源码：配置保存](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/myutils/config.py#L465-L518)

**源码事实。** 主翻译窗口在正常关闭事件中集中调用 `saveallconfig()`；Hook 搜索结果表的候选样本
保存在 UI 的内存集合中，而用户最终选择的 Hook 才进入游戏配置。因此“选择结果可接续”与“完整
候选工作区可恢复”是两种不同能力，后者并未由这条实现路径提供。
[源码：关闭时集中保存](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/translatorUI.py#L1922-L1940)
[源码：内存候选表与筛选](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/selecthook.py#L903-L930)

**官方事实。** `--userconfig` 可让多个实例读取不同配置目录，也可复制旧目录后分叉配置。
[官方多配置文件](https://docs.lunatranslator.org/zh/gooduse/multiconfigs.html)
[源码：`--userconfig` 参数](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/main.py#L245-L268)

**判断。** 原子替换与崩溃恢复值得保留；“复制配置目录得到另一个 Profile”不应进入 GlyphShift 产品
模型。GlyphShift 已有命名 Workflow、Dictionary 和 Probe Run，应通过显式复制/组合命令创建新意图，
而不是要求用户管理整个应用配置目录。

## 8. 多进程、子进程、会话恢复与诊断

### 8.1 Process Family 是实际需求

**源码事实。** LunaTranslator Hook 启动可接收多个 PID；延迟注入后会重新按可执行路径枚举进程，
代码明确指出部分 CEF/V8 游戏会稍后启动渲染子进程，因此当 PID 集变化时重新等待并注入。提权注入
前还会收集仍在运行的相关 PID。
[源码：延迟发现子进程与多 PID 注入](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L437-L497)

**源码事实。** 程序为目标 PID 启动退出等待；全部连接进程退出后回到自动 Hook 监视，针对“进程
一闪而逝、尚未来得及连接”的情况会重试。
[源码：进程退出与重连监视](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L502-L524)

**判断。** 这应直接进入 `runtime-capability-roadmap` 的高优先级 `Process Family`，与现有 Roadmap
方向一致：一次用户授权对应一个软件目标族；Controller 负责发现与生命周期，子实例分别报告
Adapter/Feature 状态；局部退出或注入失败不能终止整个软件，也不能被一个绿色汇总状态覆盖。

### 8.2 恢复依赖持久配置与重新匹配，不是恢复旧 PID

**源码事实。** Hook 选择、专用 Hook Code、游戏配置与翻译记录都落盘；重新启动时根据游戏身份读取
已选 Hook，再以兼容性规则匹配新运行时流。配置写入使用临时文件替换。
[源码：保存 Hook 选择](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/gui/selecthook.py#L932-L954)
[源码：游戏配置落盘](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/myutils/config.py#L487-L514)

**判断。** GlyphShift 的正确恢复语义也应是“恢复 Intent，再解析当前 Target Instance”，而不是把
PID、地址或 Runtime handle 持久化。Probe Run 可以恢复为 `ready/interrupted` 并保留 Dictionary 与
Observation Index；只有 Controller 重新发现目标、Runtime 完成握手与 ACK 后才回到 running/active。

### 8.3 不复制单体工具的诊断粒度

**源码事实。** LunaTranslator 在注入路径中能显示等待 DLL、杀毒软件拦截等面向用户的提示，并在
异常时显示错误；Hook Host 还能提供候选流历史。但其 Hook 选择与注入代码围绕当前 text source 和
游戏状态运作，并未形成 GlyphShift 所需的逐 Adapter / Feature / Generation 公共状态合同。
[源码：注入等待提示与错误](https://github.com/HIllya51/LunaTranslator/blob/aaef4d0cff03e54654a5428716fa3a75a659b84a/src/LunaTranslator/textio/textsource/texthook.py#L437-L500)

**判断。** 可借鉴可执行的故障提示，不借鉴聚合状态模型。GlyphShift 必须继续分别报告 requested、
actual、applied generation、dropped diagnostics 和各 Adapter Feature；“有文本被捕获”不能证明替换或
字体写回成功。

## 9. 与 GlyphShift 模型的可比边界

| LunaTranslator 概念 | 最接近的 GlyphShift 概念 | 边界判断 |
| --- | --- | --- |
| `gameuid` + 游戏库 | Software | 只借鉴持久目标身份；专用配置拆给 Workflow/Probe/Dictionary。 |
| 当前 `textsource` | Observe/Input Adapter | Luna 一次选一个主源；GlyphShift 保留 parallel Adapter Plan。 |
| HOOK 引擎 | Controller + Hook Adapter | 引擎负责连接/注入；具体 GDI/GDI+ 等能力继续由 Adapter 明示。 |
| 候选 Hook 文本行 | Observation Stream | 是运行时输入流，不是 Dictionary location，也不等于 Adapter ID。 |
| 已选 Hook + 自动重匹配 | Stream Binding | Roadmap 新 seam；保存不透明指纹与匹配策略，不保存 PID 到资产。 |
| 游戏专用词典/优化 | Workflow 组合 Dictionary/Transform Profile | Dictionary 保持纯；规则与选择关系放在 Workflow。 |
| 内嵌显示 | `inline-render` Apply Model | GlyphShift 使用离线 Snapshot，不在绘制 Hook 内等待在线翻译。 |
| 修改游戏字体 | Workflow Target Font Policy | 当前模型已更清楚，并增加 coverage 与 ACK。 |
| 翻译窗口覆盖 | 未来 Overlay Adapter | 独立窗口生命周期，不能冒充原位写回。 |
| SQLite 翻译记录 | Probe Observation Index / 未来 Runtime History | 只作证据；Dictionary 是唯一可编辑翻译真相。 |
| Python 动态模块 | Capability Adapter / Transform Operator | 不直接对应；GlyphShift 必须保留签名、ABI、placement 与执行预算。 |

## 10. 三路路由建议

### 10.1 加入当前 GlyphShift 产品交付 Work

只加入不改变 Runtime 架构的闭环项：

1. **Probe 详情增加 Adapter 筛选并持久化视图状态。** 当前 Observation Index 已保存 Adapter，列表也
   已展示 Adapter；筛选能在四个 Adapter 同时捕获、数千条数据时快速隔离噪声，不需要先实现新的
   Stream Identity。
2. **把持续作业语义纳入发布验收。** 验证暂停不影响搜索、编辑与导出；应用重启后命名 Probe Run、
   Dictionary 绑定、分页/筛选状态可恢复；目标不存在时显示 interrupted/ready 和显式恢复入口，而
   不是伪装 running。
3. **保留当前四 Tab Workflow Editor 与分权模型。** Luna 的游戏专用设置说明目标上下文重要，但不
   构成把 Software、Dictionary、Font、Hook 重新塞进一个长配置页的理由。

不建议为了本次调研延后 Dictionary Catalog、生产 Runtime Bundle、授权实机验收或首个分发构建。

### 10.2 加入 `runtime-capability-roadmap`

按优先级排序：

1. **Observation Stream Identity / Binding（最高）。** Adapter 提供不透明稳定指纹、样本历史与匹配
   置信度；Probe 可选择/忽略/排序流；Workflow 可引用已验证 Binding；目标重启后重新匹配并报告
   matched/degraded/unmatched。绝不进入 Dictionary。
2. **Process Family（最高）。** 一次授权发现主进程、延迟渲染子进程与后续实例；分别激活、退出和
   降级；共享一个 Publication Owner 和可复用 Observation 订阅。
3. **Transform Profile（高）。** 把去重、过滤、正规化、保护、结果修正建成有序、版本化、有预算的
   数据规则，由 Workflow/Probe 组合；首版不支持任意代码。
4. **OCR Fallback（中）。** Window Capture Controller + OCR Observe-only Adapter + Change Trigger
   Policy；支持绑定窗口、区域跟随、稳定性/相似度触发与单次 OCR 旁路。
5. **Clipboard/File/Manual Input（中低）。** 作为 observe-only 数据入口，主要服务字典生产和诊断，
   不扩大为在线翻译聚合器。
6. **Overlay Apply Model（中低）。** 仅在原位写回不可行时提供，独立报告绑定、可见性、位置同步和
   点击穿透状态。
7. **外部只读事件/API（低）。** 允许诊断或第三方消费 Observation/Decision；不得代替 Runtime ACK。

### 10.3 明确不采纳

1. **不采用单体“软件专用设置”聚合所有职责。** 它会破坏资产复用、变更审计和 Target 级状态诊断。
2. **不把 Hook 候选、地址、PID、窗口区域或字体写入 Dictionary Entry。** 这些是运行证据或配置关系。
3. **不在 inline-render Hook 内等待在线翻译。** GlyphShift 的绘制回调继续只查有界、不可变、离线
   Snapshot；未命中立即 fail-open。
4. **不开放任意 Python/WASM/脚本到 Runtime 或核心 Transform 管线。** 扩展必须声明能力、签名、
   ABI、placement、预算与失败语义。
5. **不用“复制整个 userconfig 目录”表达工作区或版本。** 使用命名 Workflow、Dictionary、Probe
   Run 和显式复制操作。
6. **不采用“只在正常退出时集中保存”的持续工作区策略。** Probe 的 Observation Index、编辑与
   视图状态应按修订持续 checkpoint，崩溃恢复不依赖窗口收到 close event。
7. **不把“有捕获事件”或“覆盖窗口可见”当作翻译成功。** 写回、字体和观察状态继续独立。
8. **不追随功能大全。** 在线翻译器、TTS、语言学习、Anki、模拟器数据库、Magpie 等是 LunaTranslator
   的产品定位，不是 GlyphShift 当前交付目标。

## 11. 最终判断

LunaTranslator 对 GlyphShift 的价值很高，但主要是验证架构方向并揭示一个缺失 seam，而不是要求
复制功能。GlyphShift 当前界面和底层在“资产分权、纯 Dictionary、Adapter 能力合同、Runtime ACK、
持续 Probe 作业”上更适合通用桌面软件；LunaTranslator 更成熟的地方是 **用户可从候选 Hook 流中
选出真实内容，并在下次运行恢复这项选择**。

因此当前 Work 应保持收敛，只补 Adapter 筛选和恢复验收；真正值得投入的新架构主题是 Roadmap 中的
`Observation Stream Binding + Process Family + Transform Profile`。这三项完成后，GlyphShift 才能
在不污染 Dictionary 的前提下，达到“像 Luna 一样持续工作，但比 Luna 更可组合、更可诊断”的目标。
