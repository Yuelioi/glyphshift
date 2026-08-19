# 本机实验技术发现与清理判定

## 范围与证据规则

本文将本机实验与 tracked 源码、合同测试和构建脚本交叉核对，只保留可移植结论、验证范围、失败
边界和清理判定。原始日志、截图、真实文本、授权输入、浏览器状态、机器路径、进程与窗口信息均不
进入 tracked 文档。

证据按以下强度解释：

1. tracked 合同或断言型 harness 只能证明其明确断言的行为；
2. 受控真实宿主同时验证命中、更新和恢复时，只证明该宿主与该次路线；
3. 只有正常退出或允许零命中的结果，只能证明兼容或未崩溃；
4. 只有截图、目录名、文件名或候选清单而没有机器断言时，结论保持未知。

本机材料使用 `Delete`、`Source only`、`Evidence only until distilled` 和 `Promote` 四种判定。研究结论
落盘后，原始证据统一删除；仍有独立复现价值的复杂实验只保留手写源码，不成为仓库必需资源。

## 总体结论

- 正式 tracked 实现和合同已经覆盖的简单原型、重复 Bundle、生成物与一次性诊断全部可再生成，删除。
- 复杂真实交互若没有等价 tracked harness，可在被忽略的本机工作区保留最小手写入口；其存在不代表
  产品已支持对应软件。
- 跨任务结论只有在正式源码、合同或多项独立证据共同支持时才提升为
  [Runtime 文本 Adapter 验证](../../../knowledge/rendering/runtime-text-adapter-validation.md)。
- 零命中、零替换、进程正常退出和“文件存在”都不能单独证明覆盖。

## DirectWrite、Direct2D、GDI 与 GDI+

DirectWrite 的关键模型是把 `IDWriteFactory::CreateTextLayout` 创建的 layout 与源文本及格式关联，直到
`ID2D1RenderTarget::DrawTextLayout` 真正绘制时再决策。正式实现还保留 format 的 COM 引用、有界淘汰
旧 layout，并让非统一字体、locale、装饰、inline object 或 typography 的复杂 layout fail-open。见
[DirectWrite 原生实现](../../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs)和
[纯逻辑合约](../../../../crates/adapters/implementations/native/directwrite/src/lib.rs)。

Direct2D、GDI 与 GDI+ 的 tracked 合同覆盖普通绘制、替换、决策失败透传和停用恢复。GDI 还把
`ExtTextOutW`、`TextOutW`、`DrawTextW/DrawTextExW` 作为不同 seam，并保护 glyph-index 与 symbol font
语义。见 [Direct2D 激活合约](../../../../crates/adapters/platform/native-host/tests/native_direct2d_activation_contract.rs)、
[GDI 逻辑合约](../../../../crates/adapters/implementations/native/gdi/tests/gdi_contract.rs)、
[GDI 激活合约](../../../../crates/adapters/platform/native-host/tests/native_gdi_activation_contract.rs)和
[GDI+ 合约](../../../../crates/adapters/implementations/native/gdiplus/tests/gdiplus_contract.rs)。

可移植边界：

- DirectWrite 只覆盖能关联创建与绘制的 layout；复杂分段格式必须透传。
- 模块存在或 Adapter 激活成功不等于绘制入口被调用。
- 菜单、tooltip 等短命 surface 必须在 Runtime 活跃期间主动触发，才能评价可观察性。
- 本机真实交互入口只保留源码；重复原型、Bundle、目标副本和原始证据删除。

## Qt、GTK、SDL 与 Raylib

### Qt

正式 Qt Adapter 解析动态 Qt 5/6 的 `QString` ABI，拦截常用 `QPainter::drawText` 重载，并通过 Widgets
repaint seam 请求重新绘制。configured Qt 合同验证首代、次代和停用恢复，见
[Qt 原生实现](../../../../crates/adapters/implementations/framework/qt-painter-native/src/lib.rs)、
[Qt 逻辑实现](../../../../crates/adapters/implementations/framework/qt-painter/src/lib.rs)和
[Qt 激活合约](../../../../crates/adapters/platform/native-host/tests/native_qt_painter_activation_contract.rs)。

动态模块与精确源匹配是成功前提；静态链接、QML/scene graph 或未加载相应 QtGui/Core 的进程不能
自动归入覆盖。backing store 也可能重放缓存而不再次进入 `QPainter`，因此刷新必须实际抵达目标 UI
线程。真实产品验收入口可保留源码，ABI 探针、上游包、目标副本和原始证据删除。

### GTK 3 / Pango

GTK 3 路线只在属性为空或统一属性覆盖全文时替换 layout；局部 attribute range 必须保持原语义并
fail-open。见 [GTK 原生实现](../../../../crates/adapters/implementations/framework/gtk3-pango-native/src/lib.rs)
和 [GTK 逻辑合约](../../../../crates/adapters/implementations/framework/gtk3-pango/src/lib.rs)。

验证边界是 Windows x64 动态链接 GTK 3 的 `gtk_render_layout`，不证明 GTK 4，也不证明大型应用的
每一种 widget 与重绘时机。最小 smoke 源码可保留；运行库副本、完整上游源码、输出和证据删除。

### SDL3_ttf

技术 smoke 表明，替换 `TTF_Text` 时必须复制 direction、script、color、position、wrap width 与空白
显示策略；必要导出、布局读取或属性复制失败时调用原始绘制。仓库尚无 tracked SDL3_ttf Adapter 或
Bundle 条目，因此产品状态保持“未实现”。最小源码可保留，第三方副本、构建输出和事件证据删除。

### Raylib

正式实现只在首次绘制线程绑定 render thread，按需扩充 fallback atlas，在 `EndDrawing` 后回收旧
atlas，并在 `CloseWindow` 前清理活动与退休字体；ABI、glyph、atlas、线程或字体条件不满足时透传。
见 [Raylib 原生实现](../../../../crates/adapters/implementations/framework/raylib-native/src/lib.rs)和
[Raylib 逻辑合约](../../../../crates/adapters/implementations/framework/raylib/src/lib.rs)。

边界是 Windows x64 动态 raylib 的 `DrawTextEx`；静态链接、复制实现、其他绘制路径或应用缓存纹理
文字仍未知。完整 Runtime smoke 可保留最小源码；早期状态机原型、独立注入原型及证据删除。

## Unity Mono 与 IL2CPP

### Mono Standard UI

正式 writeback 状态机覆盖初始 snapshot、setter 使用最新 app source、同代去重、代次更新与移除、
非法替换透传、停用恢复和对象回收，见
[Mono writeback](../../../../crates/adapters/implementations/framework/unity-mono-standard-ui/src/writeback.rs)。
原生 shipping-like 合同还拒绝 NGUI-only、未知 UI、IL2CPP、setter 早于主线程 snapshot、二次 attach、
缺少可验证主线程 dispatch 或没有 live text object 的目标，见
[Mono shipping-like 合约](../../../../crates/adapters/implementations/framework/unity-mono-standard-ui-native/tests/shipping_like_runtime.rs)。

支持范围是 Windows x64 Unity Mono 的 TMP/uGUI 标准 `text` surface。UI Toolkit、NGUI、自绘 mesh、
缺少主线程 dispatch 或绕过标准 setter 的入口不能自动归入覆盖。合成与真实 attach harness 可保留
最小源码，真实程序集、Runtime 副本和原始证据删除。

### IL2CPP Standard UI

IL2CPP Adapter 明确是未发布、Windows x64、`ObserveOnly`，见
[IL2CPP descriptor](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui/src/lib.rs)。
原生实现要求完整导出、可定位的 TMP/uGUI 字段、非空 live-object snapshot 和有界 attach；缺失导出、
Mono runtime、错误平台/架构或空初始对象都拒绝激活。见
[runtime gate](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/runtime_gate.rs)、
[metadata gate](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/metadata.rs)和
[observe-only 实现](../../../../crates/adapters/implementations/framework/unity-il2cpp-standard-ui-native/src/native_adapter.rs)。

现有本机材料不足以证明真实 IL2CPP 全链路 attach。tracked negative fixture 只证明“类型存在不等于有
可观察标准 UI”，见 [negative fixture](../../../../test-support/unity-il2cpp-custom-text-negative/README.md)和
[静态校验脚本](../../../../scripts/build-unity-il2cpp-negative-harness.ps1)。产品状态保持“observe-only
原型，真实 attach 未验收”；候选程序、副本和原始证据删除。

## 目标进程宿主

`TargetProcessHost` 要求 Controller ack 的 generation 与 publication identity 同时匹配，Runtime 与
Adapter artifact 由声明哈希绑定，并只报告实际 active 的 Adapter 子集。见
[process-host 合约](../../../../crates/runtime/targets/process-host/tests/target_process_host_contract.rs)和
[Windows target-process 合约](../../../../crates/runtime/controller/windows/tests/target_process_hook_contract.rs)。

Capture owner 位于目标 Runtime，由其负责 pause、drain 和 finish，见
[目标 Runtime capture](../../../../crates/runtime/targets/runtime/src/capture.rs)和
[目标 Runtime 合约](../../../../crates/runtime/targets/runtime/tests/target_runtime_contract.rs)。本机重复
宿主、变异 DLL、JSON 与构建输出均可由 tracked 测试重建，删除。

## Web 会话

远程调试必须在 host 启动前配置，profile 必须隔离；“可附着”至少需要授权目标的 WebView 后代和
可达的调试 endpoint。检测到 pipe 参数不等于 pipe transport 已实现。当前仓库没有 tracked Web
session transport，因此真实实例 attach 与 pipe transport 都保持未知。

连接诊断与一次性 host 控制可保留最小手写源码；WebView profile、History、Cookies、Login Data、
Local/Session Storage、cache、数据库、二进制和日志全部删除。

## Runtime Bundle 与采集

Bundle builder 把 Controller、目标 Runtime 和 Adapter 构建到 staging，使用完整 SHA-256 内容寻址，
写入固定 schema/first-party authority，并在发布前调用生产 loader。loader 在加载 native code 前验证
schema、authority、相对路径、完整 hash、协议、descriptor、worker 与 support file。见
[Bundle builder](../../../../scripts/build-runtime-bundle.ps1)、
[生产 loader](../../../../crates/runtime/desktop/src/bundle.rs)和
[Bundle 合约](../../../../crates/runtime/desktop/src/tests/bundle.rs)。重复 Bundle 检查器、旧 manifest、
Runtime 副本和构建输出删除。

Capture producer 使用有界非阻塞 ingress，保留 sequence、generation 与累计 drop，由唯一 owner
drain；file sink 使用 A/B checkpoint、单调 revision、去重、容量上限和恢复。见
[batch 实现](../../../../crates/core/capture/src/batch.rs)、
[catalog 实现](../../../../crates/core/capture/src/catalog.rs)和
[capture 合约](../../../../crates/core/capture/src/tests/)。桌面 acquisition 对每次点选/区域请求重新
授权，并把权限失败作为终止错误而非隐藏回退，见
[acquisition engine](../../../../crates/runtime/acquisition/src/engine.rs)和
[桌面 acquisition 合约](../../../../crates/runtime/desktop/src/tests/acquisition.rs)。

空 capture 应先检查目标进程族与 UI 宿主路由，不能直接判定 Adapter 无能力；也不能从机器专属
catalog 反推具体承载 UI 的后代进程。原始 catalog、会话、运行输出和截图删除。

## 清理结果

本机工作区只保留仍有独立复现价值的手写源码与控制文件；其精确清单留在本机，不进入 Flightdeck。
可再生成的目标目录、依赖缓存、第三方源码、Runtime/Bundle 副本、授权输入、浏览器状态、日志、
截图、catalog 和其他原始证据均已删除。后续实验继续遵守同一原则：结论进入 tracked 文档，机器
材料只留在被忽略的本机工作区。
