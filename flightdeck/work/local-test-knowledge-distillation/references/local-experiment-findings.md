# 本机实验技术发现与清理判定

## 范围与证据规则

本文审阅仓库根 `local-test/` 中的手写实验源码、入口脚本和必要的一手日志，并与 tracked
正式源码、测试及构建脚本交叉核对。它只记录可移植结论、验证范围和清理判定，不转录真实界面
文本、机器路径、进程信息、窗口信息、截图内容、原始日志内容或第三方样本内容。

证据强度按以下顺序解释：

1. tracked 合约测试或本机断言型 harness 通过，可证明其断言覆盖的行为；
2. 真实宿主日志同时有非零命中、替换/更新/恢复断言，可证明该次受控宿主路线；
3. 只有“进程正常退出”或“测试通过但允许零命中”，只能证明兼容或未崩溃；
4. 只有截图、目录名、文件名或候选清单而没有机器可判定断言，结论标为未知。

清理判定使用：`Delete`、`Source only`、`Evidence only until distilled`、`Promote`。研究稿完成后，
所有 `Evidence only until distilled` 均应删除。

## 总体判定

| 实验族 | 当前可信结论 | 建议 |
|---|---|---|
| DirectWrite | 正式实现与真实宿主均验证创建/绘制关联、两代替换和停用恢复 | 本地实现删；保留最终真机触发脚本源码 |
| Direct2D | tracked 合约验证原始/WIC 目标；本机真实宿主为零命中 | 本地材料删；零命中边界提升 |
| GDI / GDI+ | tracked 合约覆盖文本、字形索引、字体和 fail-open；真实短命菜单证明触发时机重要 | 保留最终菜单/字体验收脚本源码，其余删 |
| Qt | 动态 Qt 5/6 四个 `QPainter::drawText` 重载和真实两代更新已验证 | 保留最终真机验收源码；证据与运行副本删 |
| GTK 3 / Pango | 离屏真实宿主两代更新通过；局部属性范围安全放行 | 保留 smoke 源码；依赖、上游源码和证据删 |
| SDL3_ttf | 独立注入 smoke 验证两代替换与停用透传，尚无正式适配器 | 保留全部最小 hand-written smoke 源码 |
| Raylib | 正式适配器与真实 Runtime 验证 atlas 更新、停用、重新激活和带活动 atlas 正常关闭 | 只保留正式 Runtime smoke 源码；旧原型删 |
| Unity Mono | tracked 状态机/原生实现和本机 harness 覆盖标准 UI 观察、写回、刷新、恢复 | 保留合成与真实 attach harness 源码 |
| Unity IL2CPP | tracked 原型仅 observe-only；本机候选没有 live attach 成功证据 | 本机候选与样本全删；保持“未验收” |
| 目标进程宿主 | generation、publication identity、artifact hash、部分激活和 capture ownership 已由 tracked 合约覆盖 | 本地生成物全删 |
| Web 会话 | 源码明确了 CDP 前置条件与隔离 profile；真实可附着和 pipe transport 未被持久证据证明 | 保留两个诊断脚本源码；浏览器 profile 立即删 |
| Runtime Bundle / 采集 | tracked builder/loader 已覆盖 authority、内容哈希、worker 分层；tracked capture 覆盖有界队列、A/B checkpoint 与 drop | 本地 Bundle、checker、catalog 和证据全删 |

## DirectWrite、Direct2D、GDI 与 GDI+

### 目的与手写入口

- `local-test/directwrite-layout-observer/Cargo.toml`
- `local-test/directwrite-layout-observer/src/lib.rs`
- `local-test/build-directwrite-bundle.ps1`
- `local-test/build-direct2d-bundle.ps1`
- `local-test/run-notepadpp-directwrite-formal.ps1`
- `local-test/notepadpp-submenu-runtime-repro.ps1`
- `local-test/ae-font-acceptance.ps1`

DirectWrite 原型把 `IDWriteFactory::CreateTextLayout` 得到的 layout 指针与源文本、format、最大宽高
关联，直到 `ID2D1RenderTarget::DrawTextLayout` 真正绘制时才请求决策并构造替换 layout；它有线程重入
保护、文本长度上限和布局表上限。这个模型避免把“创建但从未绘制”的布局误报为观察结果。一手来源：
`local-test/directwrite-layout-observer/src/lib.rs`。

tracked 正式实现完整吸收了这条路线，并增加了更严格的边界：保留 `IDWriteTextFormat` 的 COM 引用、
有界淘汰旧 layout、拒绝非统一字体/locale/装饰/inline object/typography 的复杂 layout，并使用正式
Adapter 标识。见[正式 DirectWrite 原生实现](../../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs)
和[纯逻辑合约](../../../../crates/adapters/implementations/native/directwrite/src/lib.rs)。因此本地原型不是
仍需保存的唯一源码。

本机真实宿主的最终 DirectWrite 合约通过，日志记录 35 个诊断命中、第一代 16 次替换、第二代
18 次替换，并由同一测试验证停用恢复。一手证据：
`local-test/evidence/notepadpp-directwrite-formal/cargo-test.stdout.log`。tracked 原生宿主合约还分别
验证主 render target 与 compatible render target 的观察、替换、决策失败透传和停用透传，见
[DirectWrite 激活合约](../../../../crates/adapters/platform/native-host/tests/native_directwrite_activation_contract.rs)。

Direct2D tracked 合约验证 `ID2D1RenderTarget::DrawText` 在普通和 WIC render target 上都能观察、
替换、fail-open 与停用恢复，见
[Direct2D 激活合约](../../../../crates/adapters/platform/native-host/tests/native_direct2d_activation_contract.rs)。
但本机真实宿主结果虽标记 `passed=true`，诊断命中为零；它最多证明该次激活未导致宿主失败，不能
证明翻译路线命中。一手证据：`local-test/evidence/direct2d-real-host/*/result.json`。

GDI 正式合约覆盖 Unicode 文本、glyph-index 解码、字体替换、长度/编码/重入/决策失败透传，并把
`ExtTextOutW`、`TextOutW`、`DrawTextW/DrawTextExW` 作为不同 Adapter seam；原生激活合约还验证
文本与字体能力可独立启用并在停用后恢复。见
[GDI 逻辑合约](../../../../crates/adapters/implementations/native/gdi/tests/gdi_contract.rs)、
[GDI 原生实现](../../../../crates/adapters/implementations/native/gdi-native/src/lib.rs)和
[GDI 激活合约](../../../../crates/adapters/platform/native-host/tests/native_gdi_activation_contract.rs)。

真实短命菜单实验中，最终 capture 在 Runtime 活跃期间主动展开菜单后观察到全部十个目标子项；
替换实验有 123 个诊断命中和两次第一代替换，但第二代替换为零。因此可提升的结论是“短命 surface
必须在 Runtime 活跃窗口内主动触发”，而不是“菜单热更新已完整验证”。一手证据：
`local-test/evidence/notepadpp-submenu-runtime/result.json`、
`local-test/evidence/notepadpp-submenu-runtime/runtime.stdout.log` 和
`local-test/evidence/notepadpp-submenu-replace/runtime.stdout.log`。

真实字体策略测试多次以零退出码完成激活与恢复合约；它与 tracked GDI/GDI+ 合约共同证明受支持
seam 上的停用恢复。仅凭其他真实宿主的零命中记录，不能推断 GDI+ 覆盖该宿主。一手证据：
`local-test/evidence/font-safety-real-host/*/runtime.stdout.log`；正式边界见
[GDI+ 合约](../../../../crates/adapters/implementations/native/gdiplus/tests/gdiplus_contract.rs)。

### 失败边界与残余风险

- DirectWrite 只覆盖能关联 `CreateTextLayout` 与 `DrawTextLayout` 的路径；复杂分段格式必须透传。
- Direct2D、GDI、GDI+ 的模块存在或 Adapter 激活成功不等于绘制入口被调用。
- 短命菜单、tooltip 等 surface 的可观察性依赖 Runtime 活跃时机和主动触发。
- 零命中、零替换或只验证进程正常退出的结果不得提升为产品支持。

### 与 tracked 实现的关系及判定

- `Delete`：`local-test/directwrite-layout-observer/`、两个本地 Bundle 拼装脚本、Direct2D 真实宿主
  零命中证据、旧 Bundle、DLL、截图和重复失败日志。DirectWrite 本地实现已被更严格的 tracked
  实现覆盖。
- `Source only`：最终真机交互仍有独立复现价值，可保留
  `local-test/run-notepadpp-directwrite-formal.ps1`、
  `local-test/notepadpp-submenu-runtime-repro.ps1`、`local-test/ae-font-acceptance.ps1`；这些脚本含旧
  本地根约定，重用前需按当前 `local-test/` 布局校正，不能把现状视为可直接运行。
- `Evidence only until distilled`：上述成功日志与最终菜单 result；本文落盘后删除。
- `Promote`：创建/绘制关联模型、复杂 layout 透传、短命 surface 主动触发，以及“零命中不算覆盖”。

## Qt、GTK、SDL 与 Raylib

### Qt

手写入口：

- `local-test/cavalry-qt-live-update-repro.ps1`
- `local-test/cavalry-product-live.spec.cjs`
- `local-test/capture-cavalry-window.ps1`
- `local-test/assert-cavalry-product-pixels.ps1`
- `local-test/playwright-live.config.cjs`
- `local-test/qt-msvc-abi-probe.cpp`

正式 Qt Adapter 解析动态 Qt 5/6 的 `QString` ABI，拦截四种常用 `QPainter::drawText` 重载，创建
临时替换 `QString` 后调用原函数；更新时通过 Qt Widgets repaint seam 请求重绘。见
[Qt 原生实现](../../../../crates/adapters/implementations/framework/qt-painter-native/src/lib.rs)和
[Qt 逻辑实现](../../../../crates/adapters/implementations/framework/qt-painter/src/lib.rs)。configured
Qt 合约在真实 Qt Core/Gui 上验证四个重载的第一代、第二代和停用恢复，并验证 publication refresh
能驱动同一 widget 再次绘制，见
[Qt 激活合约](../../../../crates/adapters/platform/native-host/tests/native_qt_painter_activation_contract.rs)。

真实动态 Qt 宿主的产品级实验中，默认目标与交互 placeholder 两组均满足：内部合约通过、两代均
产生可见变化、停用后与基线一致；另一组静态名称目标虽内部 Runtime 合约通过，但没有可见变化。
因此动态模块与精确源匹配是成功前提，静态链接、QML/scene graph 或未加载相应 QtGui/Core 的
进程不应归入该路线。一手证据：`local-test/evidence/cavalry-qt-live-update/result.json`、
`result.interactive-placeholder.json` 与 `result.static-name.json`。

残余风险是 backing store 可能重放缓存而不重新进入 `QPainter`。生产脚本通过交互、轻微 resize 或
主题刷新触发实际重绘；这证明 refresh 需要进入目标 UI 线程，但不保证所有 Qt 应用的缓存策略相同。

判定：

- `Source only`：保留上述五个真机/产品验收脚本；它们仍是 tracked configured-Qt 合约之外的真实
  Runtime 验收入口。
- `Delete`：`qt-msvc-abi-probe.cpp` 及其 asm/obj、Qt 目标副本、上游包、运行截图、日志和旧失败
  Bundle。ABI probe 已被正式实现和测试固化。
- `Promote`：动态 Qt 识别边界、精确源匹配、backing-store 刷新要求。

### GTK 3 / Pango

手写入口：

- `local-test/gtk3-pango-smoke/Cargo.toml`
- `local-test/gtk3-pango-smoke/src/main.rs`

本地 smoke 动态加载 GTK 3/Pango/Cairo，在离屏 widget 上断言：无属性与覆盖全文的统一属性可替换；
只有局部范围属性时保持原 layout；停用后渲染回到基线。一手来源：
`local-test/gtk3-pango-smoke/src/main.rs`。正式实现也只在属性为空或首个范围覆盖全文时复制 layout
并替换文本，否则 fail-open，见
[GTK 原生实现](../../../../crates/adapters/implementations/framework/gtk3-pango-native/src/lib.rs)和
[GTK 逻辑合约](../../../../crates/adapters/implementations/framework/gtk3-pango/src/lib.rs)。

离屏真实宿主的两组 Runtime 合约均通过两代更新，分别记录非零诊断、第一代和第二代替换。真实
widget factory 的部分入口可替换，但多次要求特定文本的 live-update 尝试失败，说明大型 GTK
应用的实际文本与重绘时机仍需逐目标验证。一手证据：
`local-test/evidence/gtk3-pango/formal-offscreen-live-update.stdout.log`、
`injected-offscreen-live-update.stdout.log`、`injected-widget-factory-entry.stdout.log` 及失败 stderr。

边界：这里只证明 Windows x64、动态链接 GTK 3 的 `gtk_render_layout`；不证明 GTK 4，也不允许
改变局部 Pango attribute range 的语义。

判定：`Source only` 保留 `Cargo.toml` 与 `src/main.rs`；`Delete` Cargo 输出、MSYS2/GTK 运行副本、
完整上游源码和全部证据；`Promote` 局部样式透传与 GTK 3/4 明确边界。

### SDL3_ttf

手写入口：

- `local-test/sdl3-ttf-smoke/Cargo.toml`
- `local-test/sdl3-ttf-smoke/src/lib.rs`
- `local-test/sdl3-ttf-smoke/src/bin/injector.rs`

原型拦截 `TTF_DrawRendererText`，用原 `TTF_Text` 的 engine/font 创建替换对象，并复制 direction、
script、color、position、wrap width 与空白显示策略；必要导出缺失、布局无法读取、属性复制失败或
替换非法时均调用原始绘制。注入器通过显式进程授权参数把 DLL 加载到目标进程。一手来源：上述
三份源码。

最终事件证据包含安装成功、第一代替换、第二代替换、停用透传和三阶段捕获；更早一次运行因
unsupported layout fail-open，随后兼容路径成功。一手证据：
`local-test/evidence/sdl3-ttf-showfont-events-*.log`。仓库没有 tracked SDL3_ttf Adapter 源码或 Bundle
条目，因此这只能称为技术 smoke，不能称为产品化支持。

判定：`Source only` 保留上述三份源码；`Delete` 上游 SDL/SDL_ttf、字体副本、构建输出、BMP、
控制文件与事件日志；`Promote` 属性复制和缺失能力 fail-open 约束，产品化状态保持“未实现”。

### Raylib

本地实验包括纯状态机原型、独立注入 smoke 和正式 Runtime smoke。正式 tracked 原生实现已经吸收
前两者的关键不变量：只在首次绘制线程绑定 render thread；按所需 codepoint 构建/扩充 fallback
atlas；旧 atlas 到 `EndDrawing` 后回收；停用后延迟回收；`CloseWindow` 前清理活动与退休字体；
任一 ABI、glyph、atlas、thread 或字体条件不满足时透传。见
[Raylib 原生实现](../../../../crates/adapters/implementations/framework/raylib-native/src/lib.rs)和
[Raylib 逻辑合约](../../../../crates/adapters/implementations/framework/raylib/src/lib.rs)。

真实正式 Runtime smoke 完成第一代、第二代、停用、重新激活，并在仍有活动 atlas 时正常关闭；
后两次最终运行的两个 generation 均记录有界 trace 批次，且目标正常退出。一手来源：
`local-test/raylib-production-smoke/src/main.rs`；一手证据：
`local-test/evidence/raylib-production-*/summary.txt`。

边界：已验证的是 Windows x64 动态 raylib 5.5 `DrawTextEx`；静态链接、复制实现、非 `DrawTextEx`
路径或应用自身缓存纹理文字均未知。

判定：

- `Source only`：只保留 `local-test/raylib-production-smoke/Cargo.toml` 与
  `local-test/raylib-production-smoke/src/main.rs`，作为完整 Controller/Runtime/diagnostics/关闭链的
  真实验收入口。
- `Delete`：`raylib-fallback-prototype/` 与 `raylib-fallback-smoke/` 已被正式实现覆盖；同时删除上游
  目标源码、资源、运行副本、截图、日志和旧运行。
- `Promote`：render-thread 归属、帧尾延迟回收和 `CloseWindow` 前清理顺序。

## Unity Mono 与 IL2CPP

### Mono Standard UI

手写入口：

- `local-test/unity-mono-harness/managed/Fixture.cs`
- `local-test/unity-mono-harness/managed/ManagedFixture.csproj`
- `local-test/unity-mono-harness/native/Cargo.toml`
- `local-test/unity-mono-harness/native/src/main.rs`
- `local-test/unity-real-attach-harness/Cargo.toml`
- `local-test/unity-real-attach-harness/src/main.rs`
- `local-test/capture-window-by-pid.ps1`

合成 managed fixture 同时提供 TMP 和 uGUI `text` setter，断言初始替换、运行时 setter 新源、第二代
刷新、停用恢复，以及停用后新写入不再发布；native harness 还覆盖 Mono domain/assembly/class/
method 查找与 GC handle round-trip。一手来源：`local-test/unity-mono-harness/`。

正式纯逻辑 writeback 状态机覆盖：初始 snapshot 只写最终值、setter 使用最新 app source、相同代次
去重、代次更新与移除替换、非法替换透传、停用恢复最新 app source、对象回收后忘记状态。见
[Mono writeback](../../../../crates/adapters/implementations/framework/unity-mono-standard-ui/src/writeback.rs)。
原生 shipping-like 合约还拒绝 NGUI-only、未知 UI、IL2CPP、setter 早于主线程 snapshot、二次 attach、
缺少可验证主线程 dispatch 及没有 live text object 的目标，见
[Mono shipping-like 合约](../../../../crates/adapters/implementations/framework/unity-mono-standard-ui-native/tests/shipping_like_runtime.rs)。

真实 attach harness 使用正式 Windows Controller、目标 Runtime、Adapter binding、publication update、
observation query、deactivate 和正常关窗路径；本地留下多组 active/updated/restored 与非空 observation
证据，但没有单一机器可读 summary 证明每组视觉断言，因此“视觉写回正确”不从文件存在性推断，
保留为未知。一手来源：`local-test/unity-real-attach-harness/src/main.rs`；证据边界：
`local-test/evidence/unity-mono-*/`。

适用范围是 Windows x64 Unity Mono 的 TMP/uGUI 标准 `text` surface。UI Toolkit、NGUI、自绘 mesh
text、被裁剪元数据、缺少主线程 dispatch，以及不经过标准 setter 的快捷入口均不能自动归入覆盖。

判定：`Source only` 保留上述两套 harness 与通用捕获脚本；删除 managed/native 构建输出、运行库
副本、真实程序集、截图、日志和 observations 原文；`Promote` backend gate、标准 UI profile gate、
setter/主线程边界和恢复最新 app source 的原则。

### IL2CPP Standard UI

正式 IL2CPP Adapter 明确是未发布、Windows x64、`ObserveOnly`，不声明替换能力，见
[IL2CPP descriptor](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui/src/lib.rs)。
原生实现要求完整 IL2CPP 导出、可定位 TMP/uGUI `m_text`/`m_Text` 字段、非空初始 live-object
snapshot，并在有界时间内完成 attach；空初始结果、缺失导出、Mono runtime、错误平台/架构都拒绝
激活。见
[IL2CPP runtime gate](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/runtime_gate.rs)、
[metadata gate](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/metadata.rs)和
[observe-only 原生实现](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/native_adapter.rs)。

本地候选材料只能证明若干应用可启动、存在 IL2CPP runtime/metadata 和可见动态界面；现存 result
主要记录 capture 是否产生及模块盘点，没有持久化的 live attach、observation、replace、热更新或
恢复断言。因此真实 IL2CPP 全链路结论为未知。相反，tracked custom-mesh negative harness 明确保留
TMP/uGUI 类型但保证 live standard text object 为零，用于证明“有类型不等于有可观察标准 UI”，见
[IL2CPP negative fixture](../../../../test-support/unity-il2cpp-custom-text-negative/README.md)和
[构建/静态校验脚本](../../../../scripts/build-unity-il2cpp-negative-harness.ps1)。

判定：`Delete` 全部本机 IL2CPP 候选 ZIP、解压程序、metadata 副本、日志、截图、capture profile 和
候选 inventory；不需要保留本地 IL2CPP 源码，因为确定性 negative fixture 已 tracked。`Promote`
backend 与 live-object 双重 gate；产品状态保持“observe-only 原型，真实 attach 未验收”。

## 目标进程宿主

`local-test/target-process-host/`、`controller-runtime-contract/`、`target-runtime-contract/` 与多个
regression target 目录只含测试生成的零长度/变异 DLL、capture A/B JSON 或 Cargo 输出，没有独立
手写实现。

tracked `TargetProcessHost` 合约验证：Controller ack 的 generation 与 publication identity 必须匹配；
Runtime/Adapter artifact 以声明哈希绑定；只报告实际 active 的 Adapter 子集；不匹配身份或代次拒绝
handshake；diagnostics 和 observation batch 通过同一目标会话返回。见
[process-host 合约](../../../../crates/runtime/targets/process-host/tests/target_process_host_contract.rs)。
Windows 端到端合约进一步验证真实目标注入、两代 publication、停用恢复、变异 Runtime 在注入前被
拒绝以及有界 observation drain，见
[Windows target-process 合约](../../../../crates/runtime/controller/windows/tests/target_process_hook_contract.rs)。

capture owner 位于目标 Runtime：file 模式持有 `FileCaptureSink`，batch 模式持有
`CaptureBatchProducer`，并由 Runtime 负责 pause、drain 和 finish，见
[目标 Runtime capture](../../../../crates/runtime/targets/runtime/src/capture.rs)和
[目标 Runtime 合约](../../../../crates/runtime/targets/runtime/tests/target_runtime_contract.rs)。

判定：`Delete` 全部本地目标宿主 DLL、JSON、regression target 和输出；它们可由 tracked 测试重新
生成。`Promote` generation + publication identity 双校验、注入前 hash 校验、部分激活报告与目标
Runtime 的 capture ownership。没有测试日志的本地 mismatch 文件名不单独算验证证据。

## Web 会话

手写入口当前位于：

- `local-test/evidence/yotta-web-session/connection-diagnostic.ps1`
- `local-test/evidence/yotta-web-session/dev-host-control.ps1`

`connection-diagnostic.ps1` 把“当前实例可附着”定义为：授权目标有后代 WebView 进程，且启动时配置
的远程调试端口可达并返回 WebSocket endpoint。它会检测 pipe 参数，但 pipe 未参与成功条件，因此
不能据此声称 pipe transport 已实现。一手来源：该脚本。

`dev-host-control.ps1` 从授权外部源码构建一次性 host，在启动前设置隔离 storage、隔离 WebView
profile 和临时 CDP 端口，并要求 `/json/list` 可达且存在 page target。一手来源：该脚本。现存 host
日志只证明 WebView environment 创建，未保存脚本最终 JSON；真实生产会话可附着与 dev-host endpoint
成功均标为未知。

`local-test/evidence/web-session-pipe/` 与 dev-host profile 包含浏览器 History、Cookies、Login Data、
Local Storage、Session Storage、cache 和数据库。这些既没有可移植技术价值，也有高隐私风险。

仓库没有 tracked web-session transport 实现。因此：

- `Source only`：把上述两个脚本移动到 `local-test/web-session-transport/` 后保留；不要保留其运行
  输出。
- `Delete immediately`：整个 WebView profile、`web-session-pipe/`、dev-host 二进制、storage、日志、
  数据库与当前空的 transport 目录内容。
- `Promote`：远程调试必须在 host 启动前配置、profile 必须隔离；pipe transport 和真实实例 attach
  均保持未知。

## Runtime Bundle 与采集

### Runtime Bundle

tracked builder 把 Controller、目标 Runtime 和各正式 Adapter 构建到 staging，以完整 SHA-256 生成
内容寻址文件名，写入固定 schema/first-party authority，并在发布目录前调用生产
`RuntimeBundle::open` verifier。它限制输出和 Cargo target 都位于 `local-test/`。见
[Bundle builder](../../../../scripts/build-runtime-bundle.ps1)和
[生产 verifier](../../../../crates/runtime/desktop/src/bin/verify_runtime_bundle.rs)。

生产 loader 在加载 native code 前验证 schema、authority、单文件相对路径、完整 hash、Controller
protocol、Adapter descriptor、acquisition worker 与每个 support file；拒绝父目录路径、重复文件、
缺失/篡改 artifact、未知 authority 和非 HTTPS 文档地址。见
[Bundle loader](../../../../crates/runtime/desktop/src/bundle.rs)和
[Bundle 合约](../../../../crates/runtime/desktop/src/tests/bundle.rs)。

`local-test/bundle-check/src/main.rs` 只是再次 inspect manifest Adapter 并调用同一个
`RuntimeBundle::open`，正式 verifier 已完整覆盖；本地 DirectWrite/Direct2D/Console bundle 脚本也只是
旧的 manifest 拼装。它们没有独立保留价值。

判定：`Delete` `local-test/bundle-check/`、`runtime-bundle/`、`runtime-build/`、所有旧 Bundle、manifest
副本、许可证副本和构建输出。`Promote` first-party authority、内容寻址、加载前完整校验和 worker/
support-file 显式声明。

### Capture / Acquisition

tracked capture producer 使用有界、非阻塞 ingress，保留 sequence、generation、累计 drop，并由唯一
owner drain；file sink 使用 A/B checkpoint、单调 revision、去重、容量上限、pause/resume 与恢复。
见[batch 实现](../../../../crates/core/capture/src/batch.rs)、
[catalog 实现](../../../../crates/core/capture/src/catalog.rs)及
[capture 合约](../../../../crates/core/capture/src/tests/)。桌面 acquisition 对每次点选/区域请求重新
授权，按 source policy 选择 structured/visual provider，并把权限失败作为终止错误而不是偷偷回退；
见[acquisition engine](../../../../crates/runtime/acquisition/src/engine.rs)、
[桌面 acquisition 合约](../../../../crates/runtime/desktop/src/tests/acquisition.rs)和
[Windows UIA 合约](../../../../crates/adapters/implementations/accessibility/uia-worker/tests/windows_uia_contract.rs)。

本机同一授权应用的两个 catalog 提供一个重要反例：按既定进程族 capture 得到零条、较宽目标范围
得到 263 条，且两者 drop 都为零。这是从一手 catalog 计数作出的推断：空 capture 可能是授权族/
UI 宿主选择错误，而非 Adapter 没有能力；不能据此确定哪一个具体后代进程承载 UI。一手证据：
`local-test/evidence/cloudmusic-family-capture/catalog.*.json` 与
`local-test/evidence/cloudmusic-full-capture/catalog.*.json`。

判定：`Delete` 所有 desktop data、quick-probe session、capture workspace、原始 catalog、A/B JSON、
OCR/UIA 运行输出和截图；真实文本不进入文档。`Promote` 有界队列、drop 可见性、A/B checkpoint、
每次请求重新授权，以及“零条记录先检查目标族路由”的诊断顺序。

## 可直接执行的最小源码保留清单

以下是建议清理后仍留在 `local-test/` 的完整实验源码集合。未列出的构建输出、依赖、第三方源码、
Runtime、Bundle、日志、截图、catalog、profile 和样本均可删除。

这里的 `Source only` 是“保留技术复现源码”，不是“当前即可运行”。若脚本或 Cargo manifest 仍按迁移
前的目录层级推导仓库根或 crate 路径，应在下一次复现前统一修正并补一份短 README；不要为了让旧
入口继续工作而保留旧 Runtime、旧 target 或第三方目标副本。

### 工作区控制文件

- `local-test/README.md`
- `local-test/machine.ps1`（只保留本机配置，不进入 Git）

### DirectWrite / GDI 真实交互入口

- `local-test/run-notepadpp-directwrite-formal.ps1`
- `local-test/notepadpp-submenu-runtime-repro.ps1`
- `local-test/ae-font-acceptance.ps1`

这三份脚本重用前需校正迁移前遗留的本地根推导；只保留源码，不保留其目标副本或证据。

### Qt 真实 Runtime / 产品验收

- `local-test/cavalry-qt-live-update-repro.ps1`
- `local-test/cavalry-product-live.spec.cjs`
- `local-test/capture-cavalry-window.ps1`
- `local-test/assert-cavalry-product-pixels.ps1`
- `local-test/playwright-live.config.cjs`

### GTK 3 / Pango

- `local-test/gtk3-pango-smoke/Cargo.toml`
- `local-test/gtk3-pango-smoke/src/main.rs`

### SDL3_ttf

- `local-test/sdl3-ttf-smoke/Cargo.toml`
- `local-test/sdl3-ttf-smoke/src/lib.rs`
- `local-test/sdl3-ttf-smoke/src/bin/injector.rs`

### Raylib 正式 Runtime smoke

- `local-test/raylib-production-smoke/Cargo.toml`
- `local-test/raylib-production-smoke/src/main.rs`

### Unity Mono

- `local-test/unity-mono-harness/managed/Fixture.cs`
- `local-test/unity-mono-harness/managed/ManagedFixture.csproj`
- `local-test/unity-mono-harness/native/Cargo.toml`
- `local-test/unity-mono-harness/native/src/main.rs`
- `local-test/unity-real-attach-harness/Cargo.toml`
- `local-test/unity-real-attach-harness/src/main.rs`
- `local-test/capture-window-by-pid.ps1`

### Web 会话诊断

清理时把以下两份源码从 evidence 目录移动到 `local-test/web-session-transport/`：

- `connection-diagnostic.ps1`
- `dev-host-control.ps1`

移动后删除原 `local-test/evidence/yotta-web-session/` 和全部浏览器 profile。

## 已被 tracked 覆盖、可整删的实验

- `local-test/directwrite-layout-observer/`
- `local-test/raylib-fallback-prototype/`
- `local-test/raylib-fallback-smoke/`
- `local-test/bundle-check/`
- `local-test/target-process-host/`
- `local-test/controller-runtime-contract/`
- `local-test/target-runtime-contract/`
- `local-test/target-runtime-regression/`
- `local-test/runtime-bundle/`
- `local-test/runtime-build/` 及所有 `*-target`、`build`、`tauri-build`、`desktop-release` 输出
- `local-test/unity-sample-gate/` 的 IL2CPP 候选、下载与解压内容
- `local-test/notepadpp-directwrite-test/`（仅 Cargo 输出）
- `local-test/qt-msvc-abi-probe.asm`、`.obj` 和 `.cpp`
- `local-test/build-directwrite-bundle.ps1`
- `local-test/build-direct2d-bundle.ps1`
- `local-test/build-console-observer-bundle.ps1`

其余顶层一次性迁移、截图、启动、provider smoke、旧 Playwright 检查、字典转换、图标分析和旧路径
修复脚本不承担上述最小源码的构建入口，也不被任何 tracked 资源依赖；在确认没有正在进行的本机
任务引用后可删除。所有 `local-test/evidence/` 原始内容在上述 Web 两份源码移出后均可删除。
