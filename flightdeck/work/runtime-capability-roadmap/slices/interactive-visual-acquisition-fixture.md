# Target Frame + OCR Visual Fixture

Status: Finished

## Outcome

以第二个真实 Acquisition Adapter 证明既有 Interface 能容纳 Visual 路线：Adapter 只消费 Controller
授权目标的内存帧，将用户 Region 裁剪到该目标表面，把 OCR 局部块正规化为虚拟桌面物理像素 anchor，
并返回有界置信度；任意桌面截图、跨目标区域和图像持久化均不进入 Interface。

## Scope

- 定义不含窗口句柄、PID、文件路径的授权 Target Frame 输入。
- Region 必须与当前 target frame 相交并被裁剪；完全落在目标外时返回目标不匹配。
- OCR Fixture 只接收裁剪后的内存像素视图，返回局部文字矩形与 bounded confidence。
- Adapter 将局部 OCR 块映射为 `Provenance::Visual + Granularity::Region`，继续复用
  `InteractiveTextAcquisition` 的正规化、去重、文本上限和回退语义。

## Non-goals

- 不选定生产 OCR 引擎，不联网，不写截图、日志或 Observation Index。
- 不实现 Windows Desktop Duplication、PrintWindow、Graphics Capture 或 UI。
- 不接 Translation Provider、Dictionary 或 Presenter。

## Verification

- [x] Region 只向 OCR 暴露授权 target frame 的交集，逐行复制后的输入不含交集外像素。
- [x] 多显示器负坐标、Per-Monitor DPI 和 crop-local → desktop anchor 映射正确。
- [x] 跨目标、受保护帧、空 OCR、bounded confidence 和部分块均有稳定结果。
- [x] 56 包 Workspace test、全目标 Clippy、fmt、architecture checks 与代码自审通过。

## Review

- 新增 `adapters/implementations/fallback/ocr`，实现既有 `AcquisitionAdapter`；没有向调用者增加第二套 request /
  result Interface。
- `TargetFrame` 不实现 `Clone` 或 `Debug`；OCR 引擎只借用交集内的 RGBA8、宽和高，不接触 target token、
  桌面坐标或交集外像素。
- 只有 Region 进入 Visual Adapter；Point / Text Range 不会被静默改写。完全越界和 frame target 不匹配
  返回目标不匹配，受保护帧返回权限拒绝。
- 生产截图和 OCR 引擎仍未选定，因此本切片只声明 Fixture 能力，不进入正式 Bundle 或 UI catalog。

## Next

继续 [一次性 Acquisition Worker transport](interactive-acquisition-worker-transport.md)，先固定有界 wire
request/result、授权 grant 绑定、超时/取消与进程退出语义；没有真实授权目标和明确 OCR 引擎前，不把
Fixture 包装成生产支持。
