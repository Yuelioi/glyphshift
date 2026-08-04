# DirectWrite / Direct2D 文字 Adapter 原型

## Goal

以真实 Direct2D `DrawText` 调用和合成宿主证明一个窄 Adapter 能观察 UTF-16、在单次绘制内安全替换、
停用后立即恢复透传，并在解析、决策或重入失败时 fail-open；未获得目标软件命中与可见替换证据前
不进入生产 Bundle。

## Current

微软一手资料与 SDK 合同复核已经完成。`IDWriteFactory::CreateTextLayout` 虽然拥有完整 UTF-16，但译文
会烘焙进长寿命 COM layout：停用后已创建对象不会恢复，变长译文还会破坏 range formatting、hit-test
和 caret 语义。因此最初的 retained layout 替换实验已经删除，没有扩张 Native ABI。

当前原型改为 `ID2D1RenderTarget::DrawText`。它在每次实际绘制时同时提供 UTF-16、format、layout rect、
brush、options 与 measuring mode；Adapter 只替换字符串 pointer/length，其余参数原样透传。独立纯
Module、native DLL、DCRenderTarget 与 WIC 内存像素宿主已经通过 observe-only、replace、decision
fail-open、deactivate 与 reactivate 合同。两类 render target 在当前系统命中同一实现地址，但它尚未
进入生产 Bundle：该证据不能继续外推为 HWND、DXGI 或任意目标软件既有 render target 的覆盖承诺。

授权 AE 实测已完成：Runtime 可以正常激活并保持，但 `DrawText` 命中为零，没有可见替换证据。这与
现有 AE 文字路径结论一致，因此该原型不会为 AE 提升为生产能力，也不会进入当前 Runtime Bundle。

现有 Native Adapter Interface 足以复用 Runtime 的 UTF-16 决策、捕获、Feature 授权与停用语义；
DirectWrite/Direct2D COM 对象、vtable、格式和绘制生命周期必须全部留在新的 Adapter 实现内，不进入 Core、
Dictionary、Workflow 或通用 Runtime Interface。

## Decisions

- 首个可写 Seam 是 `ID2D1RenderTarget::DrawText`，Descriptor 为 `InlineRender + TargetProcess`。
- 原型只声明 `TextObserve + TextReplace`，不修改共享 `IDWriteTextFormat`，不声明 `FontSubstitute` 或
  `LayoutAdjust`。
- 原型由独立 host-independent Adapter Module、native DLL Adapter 和 Windows 合成宿主组成；调用者继续
  只认识既有 Native Adapter Interface。
- Detour 安装后可以保留，停用时 active feature bits 归零并直接调用原函数；失败和重入只透传一次。
- `CreateTextLayout`、`CreateGdiCompatibleTextLayout`、`DrawTextLayout`、`DrawGlyphRun` 与 DWriteCore 是不同
  Seam，不在一个原型里混成万能 DirectWrite Adapter。
- AE 2020 的既有结论不变：`GetGlyphIndices` 只提供诊断明文，当前真实替换出口仍是 GDI+。

## Verification

- 纯合同覆盖 observe-only、replace、非法/过长 UTF-16、重入、决策失败、绘制参数保持和 original
  恰好调用一次。
- 原生合同通过真实 Direct2D DCRenderTarget 与 WIC render target：observe-only 像素等于 baseline，
  replace 像素等于独立 replacement 渲染，decision error 与 deactivate 恢复 baseline，reactivate
  不重复安装。
- 授权 AE 实机合同通过 Runtime 激活与诊断流程，但 `DrawText` 命中为零；没有把零信号包装成支持。
- 原型 package、全 workspace、Clippy、fmt 与架构检查全部通过。

## Next

- 保持原型不进入生产 Bundle。只有另一个授权目标软件先证明真实 `DrawText` 命中、render target 类型
  与可见替换，再重新评估生产化；AE 不再作为该 Seam 的候选目标。
