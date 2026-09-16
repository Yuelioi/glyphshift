# 运行时文字 Adapter 的验证与晋级

运行时文字 Adapter 只有同时证明“入口确实命中、译文决策正确、目标像素可见、更新与停用可恢复”，
才能从技术原型晋级为产品支持。模块存在、Hook 安装成功、进程未崩溃或合成宿主通过，都不是充分条件。

## 三层证据

1. **技术 seam**：证明目标 ABI、公开入口或运行时元数据能稳定取得完整原文，并且失败时调用原路径。
2. **确定性合同**：在合成宿主中覆盖首代替换、同进程下一代更新、停用恢复、重入、非法输入、缺失
   能力与目标退出。
3. **授权真实目标**：在代表性动态链接目标中同时取得非零命中、可见像素变化、第二代变化与停用后的
   原文恢复；过程必须由机器可判定断言收口。

## 新 Adapter 立项门槛

新增通用 Adapter 在进入生产实现前，还必须先满足“代表目标价值”门槛：

- 至少有一个相对知名、公开可获得、当前仍有现实用户价值的真实软件作为主验证目标；
- 一手源码 / 官方资料与实机证据必须证明该软件确实走候选技术路径，不能只按框架标记、DLL 加载或品牌关系推断；
- 必须先证明目标存在现有 Adapter 无法覆盖的完整文字区域；如果最终仍落到已经支持的 GDI、DirectWrite、Qt、GTK 等入口，则判定为重复覆盖 No-Go；
- 小众软件可以作为第二样本、ABI 证据或技术 smoke，但不能单独承担“新增生产 Adapter”的收益证明；框架 Demo、合成宿主也不能替代真实代表目标；
- 代表目标只决定“值不值得做”，不能进入 Adapter 身份。Adapter ID、locator、运行时 gate 与实现分支仍必须按通用技术 seam 定义。

产品 Catalog 默认只接受完成第三层的窄能力。若用户明确决定先分发证据尚未闭环的通用候选，Catalog 必须
保留准确技术边界与证据债务，产品状态仍按“候选实时翻译”解释；不能因为已进入正式 Bundle 就把它升级成
“已验证实时翻译”，也不能按框架名称推导软件覆盖率。

## 通用实现约束

- **完整原文优先**：选择仍持有完整字符串的最窄入口，不从逐字形、顶点或纹理缓存反推文本。
- **静态链接 profile 要证明 callable 身份**：常量、循环和数据布局与框架源码相似，只能证明语义相关；若
  目标把 canonical 函数内联进调用者，或所需入口被链接器消除，就不能把某个相似 caller 当作通用 Hook。
  只有能用版本 / ABI / 类型和调用结构唯一证明的 callable seam 才激活，缺失或歧义时 fail-closed。
- **Capture key 与实时决策一致**：目录聚合若按 `SourceTextPolicy` 生成 key，目标进程的实时 lookup 也必须使用同一 policy。精确原文仍应优先；只有精确查找失败时才回退到规范 key。若规范化只去除布局用首尾空白，采用回退译文时要把原调用的首尾空白补回，避免“目录能翻译、像素查不到”或破坏布局。
- **不修改目标所有权**：优先复制 layout、字符串对象或字体状态并瞬时绘制译文；保留目标对象与原文，
  使停用恢复不依赖逆向修改。
- **关联跟随成功创建更新**：目标对象地址可以被复用；成功创建但内容为空或无法观察时也要移除旧原文关联，防止历史译文画进新对象。DirectWrite 合同应包含非空布局释放后被空布局复用的像素验证。
- **复杂格式 fail-open**：无法证明局部字体、locale、装饰、inline object、Pango attribute 或方向脚本
  可等价迁移时，调用原始路径。
- **代次显式切换**：首代、下一代与移除替换都通过 publication generation 驱动；相同代次去重，旧
  GPU/布局资源延迟到安全边界回收。
- **线程与生命周期归属明确**：UI 重绘进入目标 UI 线程；GPU 字体 atlas 绑定首次绘制线程，在帧尾或
  `CloseWindow` 前清理；Managed Runtime 的 setter、snapshot 与恢复受主线程和对象寿命约束。
- **零命中不是支持**：Adapter 激活成功但诊断为零，只能说明该次没有破坏目标。短命菜单和 tooltip
  还必须在 Runtime 活跃窗口内主动触发。
- **缺字也要验证译文**：跨文字体系的翻译验收必须包含原字体不含目标字形的用例，分别检查像素、
  测量宽度、后续新增字形和停用恢复。仅证明“缺字时保留原文、不崩溃”是失败保护合同，不能当作
  中文翻译成功；预先把中文字形装入合成字体也不能代替字体回退验收。
- **字体图集按实际格式读取**：MonoGame 的 GetData 返回原格式存储数据；对 DXT 图集要先读取压缩块并
  解码，再复制到 RGBA 图集。纹理回退的合成验收应包含压缩字体和混排，不能仅用自行生成的 Color
  纹理推导真实游戏字体可用。

## 技术栈边界

| 类型 | 已验证的实现方式 | 必须保留的边界 |
|---|---|---|
| DirectWrite | 在 `CreateTextLayout` 保存原文与 layout 关系，实际 `DrawTextLayout` 时复制并替换 | 复杂分段格式、inline object 与无法关联的 layout 透传 |
| GDI / GDI+ / Direct2D | 在公开绘制入口取得文本，精确保留编码、字体与原调用语义 | 模块存在不代表入口命中；glyph-index 必须显式解码 |
| Qt Widgets | 解析动态 Qt 5/6 `QString` ABI，拦截 `QPainter::drawText` 并请求 UI 线程重绘 | 静态 Qt、QML、scene graph 与 backing-store 缓存不自动覆盖 |
| Qt Translation Service | 在精确动态 Qt 6.10.3 MSVC x64 `QCoreApplication::translate` 保留完整 source/context/disambiguation/n，并用 `LanguageChange` 请求重译 | Dictionary 仍只按 source 唯一；context/disambiguation 只作观测证据与 AI 语义提示，不进入词典 key。同 source 的非复数调用共享一条译文；plural 当前只观察不替换；placeholder 多重集合必须保持。Qt 5.15.18 仅研究 feature、future-call-only，不进入默认生产 Bundle |
| GTK 3 / Pango | 复制 `PangoLayout`，仅对无属性或全文统一属性替换文本 | 局部 byte-range 样式透传；GTK 4 不沿用该 seam |
| SDL3_ttf | 复制 `TTF_Text` 的 engine/font 及公开布局属性后绘制译文对象 | 当前只有技术 smoke，没有代表性真实目标与产品 Adapter |
| Raylib | `DrawTextEx` 使用 fallback atlas，按帧回收旧资源并在关闭前清理 | 仅动态 raylib 5.5；静态链接、复制实现和业务纹理缓存不覆盖 |
| SideFX CV PaintBuffer | 在动态 `libCV.dll` 的 `CV_PaintBuffer::textWrappedInBox` 完整 UTF-8 字符串入口瞬时替换参数 | 当前只验证 Windows x64 与精确 MSVC 导出；已缓存绘图区是否重画仍由目标自身决定，缺模块或导出必须 fail-closed |
| Unity Mono | 同时通过 backend、标准 UI profile、live object 与主线程 gate，再管理 setter/snapshot/恢复 | NGUI、自绘 mesh、UI Toolkit、IL2CPP 与无主线程 dispatch 的目标拒绝 |
| Unity IL2CPP | 导出、metadata、live standard text object 与可证明主线程四重 gate；写回只调用标准 `set_text` | 已按明确产品决定作为 Windows x64 候选进入生产 Bundle；存在 TMP/uGUI 类型不等于有活动标准 UI，真实写回复跑与外部同后端自绘负例补齐前不得宣称“已验证实时翻译” |

归档 UIA/OCR 不属于当前产品能力、验证入口或替换方案；不得以旧观测原型补齐当前适配器覆盖。

## 结果解释

- DirectWrite 的观察资格与替换资格分别判断。已关联原文持续进入采集；全文统一 Typography 可以复制，局部样式及不支持的对象保持原样。格式和尺寸以绘制时的布局为准，复制范围按译文 UTF-16 长度重建；译文未采用时在原始绘制前释放作用域。见[复杂格式原生合同](../../../crates/adapters/platform/native-host/tests/native_directwrite_capture_formatting_contract.rs)与[真实显示验证](../../work/ae2025-text-capture/references/typography-translation.md)。
- `Matched + Replaced` 只证明生成了替换决策；Qt/GTK 等无返回值绘制入口不能据此证明最终像素。
- 真实测试必须区分“进程正常退出”“入口非零命中”“第一代可见”“第二代可见”“停用恢复”五类结果。
- 空 capture 且 drop 为零时，先检查授权进程族、实际 UI 宿主和后代路由，再判断 Adapter 能力。
- 上游框架源码用于定位 seam，正式结论应由 tracked 实现/合同和授权真实目标共同承担；本机日志、
  截图、运行库与目标副本只用于当次验证，结论落盘后即可删除。

## 权威实现入口

- [DirectWrite 原生实现](../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs)
  与[激活合同](../../../crates/adapters/platform/native-host/tests/native_directwrite_activation_contract.rs)
- [Qt 原生实现](../../../crates/adapters/implementations/framework/qt-painter-native/src/lib.rs)
- [Qt Translation Service 原生实现](../../../crates/adapters/implementations/framework/qt-translation-native/src/lib.rs)
- [GTK 3 / Pango 原生实现](../../../crates/adapters/implementations/framework/gtk3-pango-native/src/lib.rs)
- [Raylib 原生实现](../../../crates/adapters/implementations/framework/raylib-native/src/lib.rs)
- [SideFX CV PaintBuffer 原生实现](../../../crates/adapters/implementations/framework/sidefx-cv-paint-buffer-native/src/lib.rs)
- [Unity Mono writeback](../../../crates/adapters/implementations/framework/unity-mono-standard-ui/src/writeback.rs)
  与[真实运行时合同](../../../crates/adapters/implementations/framework/unity-mono-standard-ui-native/tests/shipping_like_runtime.rs)
- [Unity IL2CPP runtime gate](../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/runtime_gate.rs)
- [完整实验审计与清理判定](../../work/local-test-knowledge-distillation/references/local-experiment-findings.md)
