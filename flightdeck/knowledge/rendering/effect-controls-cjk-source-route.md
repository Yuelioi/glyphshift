# After Effects 面板与效果的 GDI+ 路径

After Effects 面板、时间轴、效果控件和多类对话框通过 `gdiplus.dll!GdipDrawString` 绘制明文 UTF-16。菜单则走 GDI `ExtTextOutW`，两者是同一 DLL 中的不同 Hook Path。

## 当前处理链

```text
GdipDrawString raw call
  → bounded strict UTF-16 decode
  → After Effects label normalization
  → TextObservation(site=shared-ui, scope=graphics identity)
  → sequential-heading route
  → effects/<header>，未命中再 app
  → Replacement Decision
  → 原 GdipDrawString 恰好一次
```

AE 的尾随空格和点号由 `glyphshift-hosts::after_effects::normalize_shared_ui_label` 处理；Windows Adapter 不知道 `Glow`、`Threshold` 等宿主词汇。

## Effect context

Effect header 只在 `headers` 集合命中时更新 context。context 位于 TLS `HookSession`，并按 opaque `ScopeToken` 隔离，不再使用全进程 `CURRENT_EFFECT`。

不同 graphics surface 不共享 context：同一 scope 的参数先查 `effects/<effect>`，另一个 scope 的同名参数只能回退 `app`。真实 AE 中 header 与 parameter 是否稳定复用 graphics identity，仍需按 [AE Runtime smoke](../../work/hook-core/slices/ae-runtime-smoke.md)验证；若不稳定，应寻找更可靠的 scope 证据，不能恢复全局状态。

## Fail-open 与捕获

- `len = -1` 与正长度输入都限制为最多 1024 UTF-16 units。
- 非法 UTF-16、超长输入、Runtime 未就绪或 TLS 重入都直接 Pass。
- Adapter 的 FnOnce dispatch 让 Pass/Replace 都只能调用一次原函数。
- 日志和未翻译文本通过有界非阻塞队列交给后台线程；队列满时丢诊断，不阻塞宿主。
- 当前捕获只标记 `[PANEL]`；更细的 effect context 需要未来 Runtime Event 暴露。

## 已排除路线

DWrite `CreateTextLayout`、`DrawGlyphRun`、`CreateGlyphRunAnalysis` 和 `GetGlyphRunOutline` 在历史 AE 2020 面板重绘探针中没有提供可替换绘制出口。GDI+ 是已验证的实际绘制路径，详见 [AE 2020 文字渲染图](ae2020-text-render-map.md)。
