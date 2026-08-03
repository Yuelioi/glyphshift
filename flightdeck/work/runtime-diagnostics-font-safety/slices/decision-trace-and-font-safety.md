# Decision Trace 与字体安全

Status: Complete

## Outcome

现有 GDI/GDI+ 决策具备低成本解释值、可选有界记录和完整 Publication identity；Controller ACK
不再只确认 Generation。全量字体模式对用户明确暴露影响范围，GDI 符号字体基于真实 charset 保持
原字体。

## Decisions

- Decision Trace 使用 `status + text + font + generation + translation digest + font digest`，不复制
  Route、窗口、坐标或 Dictionary Context。
- Runtime Kernel 默认关闭 Trace；开启时使用 256 条环形缓冲，`try_lock` 失败或覆盖旧记录只增加
  dropped 数，RenderDecision 保持不变。
- Runtime Publication identity 对规范 JSON 做 SHA-256，包含 Route；它不同于已有 Translation/Font
  digest，也不成为第二份持久 Snapshot。
- Controller protocol `/2` 的 Runtime ACK 同时返回 Generation 和 publication identity；Host 任一
  不匹配都拒绝成功。
- GDI `SYMBOL_CHARSET` 是当前唯一已验证的字体保护信号；GDI+ 保持未支持，不使用字体名称黑名单。
- `all_observations` 不增加确认 Modal或自动超时，只在原位置显示紧凑、可访问且诚实的高影响警示。

## Delivery

- [x] DecisionResult 暴露 allocation-free Decision Trace。
- [x] Runtime Kernel 提供默认关闭的 bounded trace buffer 和 drain Interface。
- [x] Runtime Publication identity、Controller SDK/Host/Windows 与 TargetProcessHost ACK 校验贯通。
- [x] 修复 stale Publication 先切 Route 再失败的非原子行为。
- [x] ExtTextOutW、TextOutW、DrawTextW/ExW 共享 GDI 符号 charset 保护。
- [x] Workflow 全量字体模式增加中英文高影响警示。
- [x] Target Runtime/Controller 提供只读 Trace Query，Desktop 展示本地诊断。
- [x] Controller 激活拒绝保留稳定类别，Capture 失败会话可真正重连并显示可操作原因。
- [x] 授权目标软件验证字体写回、GDI glyph-index 安全及 GDI+ 剩余风险。

## Verification

- [x] 仓库原生预构建与全 Rust workspace tests 通过。
- [x] Clippy `-D warnings`、fmt、架构检查通过。
- [x] Desktop production build 与 36 项 Playwright 通过。
- [x] Impeccable detector 无发现；标准视口截图确认警示密度与滚动正常。
- [x] Trace Query/桌面诊断具备真实 Windows 注入、stdio Controller、Target Host、Desktop Runtime、
  Tauri 与 Playwright 端到端合同。
- [x] Native 字体安全像素合同证明普通 GDI 字体替换、GDI `SYMBOL_CHARSET` 保持、GDI+ Symbol
  字体风险和停止后完整恢复。
- [x] 授权 AE 的 Debug 与 Release Runtime Bundle 验收同时观测 GDI/GDI+；替换字体生效时菜单
  保持可读、图标完整，停止后的截图与基线逐像素一致。

## Current

底层 Trace、ACK、GDI 字体保护与桌面本地诊断已交付并验证。真实 AE 验收发现旧字体 glyph ID 会在
字体替换后造成菜单乱码；Adapter 现在只在字体确实切换时，把已解码原文作为 Unicode 重绘并清除
`ETO_GLYPH_INDEX` 与旧 spacing，失败时保留原参数。合成 Host 使用不同 glyph 映射的字体固定该
回归，并继续证明 GDI `SYMBOL_CHARSET` 保护及 GDI+ Symbol 的已知风险。授权 AE 中替换字体同时
作用于 GDI 菜单和 GDI+ 面板，停止后的画面与基线完全一致。

## Next

None.
