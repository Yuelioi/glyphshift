# Windows 生产 OCR 显式回退选项

> 调研日期：2026-08-07
> 范围：Windows 桌面端、一次性用户授权取帧、本地 OCR。本文只使用 Microsoft 官方文档/官方代码库与 Tesseract 官方资源，不包含任何实机原始证据。

## 结论先行

**可以开始工程验证，但尚不能提升为默认产品回退。** 任一授权软件的可见文字都足以验证 Capture、ROI、OCR、坐标和生命周期；“至少两个明确授权的真实目标同时满足 UIA 无文本、但目标像素中文字可见”应作为产品收益与发布门槛，而不是阻塞技术实现。已有 Visual/OCR fixture 已证明合同、裁剪与坐标组合，下一步可接生产候选验证。

如果后续补齐证据，推荐的技术路线是：

1. 用 `Windows.Graphics.Capture` 的 `IGraphicsCaptureItemInterop::CreateForWindow` 仅从已授权 `HWND` 取一个有效帧，Windows 基线为 10 1903 / build 18362。
2. 只在 UIA 明确无结构化文本且用户点击“尝试 OCR”后开始；保留系统捕获边框，不开启无边框捕获。
3. 将物理光标点附近的小块 ROI 裁剪到内存，校验非空/非受保护帧后再交给 OCR，立即关闭 capture session。
4. 当前无 package identity 的分发模式下，以 **Tesseract + 精简 `tessdata_fast` 语言集** 作为首个可再分发候选，但须先通过真实目标识别率、延迟、制品与完整依赖许可审核。
5. `Windows.Media.Ocr` 仅在产品接受 MSIX/package identity 且接受“语言包由 Windows 管理”时再评估；新 `Microsoft.Windows.AI.Imaging.TextRecognizer` 仅适合将来的可选 NPU 高能力 Provider，不适合当前通用回退。

## 1. 已知授权 HWND 的官方取帧方案

### 1.1 Windows.Graphics.Capture（WGC）

Win32 桌面应用可以从 `GraphicsCaptureItem` activation factory 查询 `IGraphicsCaptureItemInterop`，再用 `CreateForWindow(HWND, IID_GraphicsCaptureItem, ...)` 创建只指向该窗口的 capture item。该 desktop interop 的最低支持版本是 Windows 10 1903 / build 18362；Microsoft 官方 Win32 样例也以该版本为基线。[[`IGraphicsCaptureItemInterop` 参考](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nn-windows-graphics-capture-interop-igraphicscaptureiteminterop)] [[`CreateForWindow` 参考](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow)] [[Microsoft Win32 样例](https://github.com/microsoft/Windows.UI.Composition-Win32-Samples/tree/master/cpp/ScreenCaptureforHWND)]

典型单帧链路是：创建 D3D11 device → 包装为 WinRT `IDirect3DDevice` → 创建 `Direct3D11CaptureFramePool` → `CreateCaptureSession` → `StartCapture` → 在 `FrameArrived` 中用 `TryGetNextFrame` 取帧 → 复制 `Surface` 的有效子矩形到 CPU 可读内存。应尽快释放 checked-out frame，不在回调里做 OCR 重工作。[[Microsoft screen-capture walkthrough](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)] [[D3D11/WinRT interop](https://learn.microsoft.com/en-us/windows/win32/api/windows.graphics.directx.direct3d11.interop/nf-windows-graphics-directx-direct3d11-interop-createdirect3d11devicefromdxgidevice)]

后台 Worker 可用 `Direct3D11CaptureFramePool::CreateFreeThreaded`：它不依赖 `DispatcherQueue`，`FrameArrived` 在 frame pool 内部线程触发。该 API 自 Windows 10 1809 可用，但完整 HWND 路线仍受 `CreateForWindow` 的 1903 下限约束。[[`CreateFreeThreaded` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframepool.createfreethreaded)]

Rust 不是此路线的阻塞：Microsoft 官方 `windows` crate 可投影 Win32、COM 与 WinRT API，包括 capture interop。但 Rust 层仍需显式封装 D3D 资源生命周期、HRESULT、异步事件和取消。[[Rust for Windows](https://github.com/microsoft/windows-rs)] [[Rust `IGraphicsCaptureItemInterop`](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/WinRT/Graphics/Capture/struct.IGraphicsCaptureItemInterop.html)]

### 1.2 显式用户意图与授权边界

`GraphicsCapturePicker` 是最强的系统级用户选择证据：用户在安全系统 UI 选窗口/显示器，系统为正在捕获的 item 绘制可见边框。桌面应用初始化 picker 时需传入它的 owner `HWND`。[[Screen capture 的 picker 流程](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)]

`CreateForWindow(HWND)` **本身不弹出 picker，也不能被描述为系统已征得用户同意**。Glyphshift 必须在产品层保证：

- 本次 UIA 请求已绑定到已授权目标；
- UIA 返回稳定的“无结构化文本”，而不是密码/保护属性拒绝；
- 用户对本次结果表面的“尝试 OCR”作出单次显式动作；
- capture item 的 `HWND` 与该授权目标一致，不接受任意前台窗口；
- session 只活到取得首个有效帧或超时/取消。

不建议去掉系统捕获边框。从 Windows build 20348 开始，禁用边框前需 `GraphicsCaptureAccess.RequestAccessAsync(Borderless)` 获得用户同意，并在 package manifest 声明 `graphicsCaptureWithoutBorder`。[[`IsBorderRequired` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.graphicscapturesession.isborderrequired)] [[App capability declarations](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/app-capability-declarations)]

### 1.3 受保护、黑帧、最小化与失败语义

Microsoft 官方 Win32 样例明确说明：最小化窗口会被枚举，但不会被捕获。因此首帧必须有短超时，最小化/没有有效帧应映射为 `capture_unavailable`，不能是 `ocr_no_text`。[[Microsoft Win32 样例](https://github.com/microsoft/Windows.UI.Composition-Win32-Samples/tree/master/cpp/ScreenCaptureforHWND)]

目标软件可用 `SetWindowDisplayAffinity` 让公开 OS capture API 看不到内容：`WDA_MONITOR` 会让捕获中的该窗口无内容；Windows 10 2004+ 的 `WDA_EXCLUDEFROMCAPTURE` 则将它从捕获中排除。Microsoft 也明确指出这不是 DRM 或绝对安全保证。[[`SetWindowDisplayAffinity` 参考](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity)]

WGC 的公开 frame 合同没有与 Desktop Duplication 的 `ProtectedContentMaskedOut` 等价的标记。因此工程上应对全黑、全透明、空尺寸或超时帧执行 fail-closed，统一对外表达为 `protected_or_blank_capture`；不要猜测是“受保护内容”还是“目标正好绘制黑色”，更不能继续解释为 OCR 空文本。

### 1.4 DPI、坐标、resize 与 HDR

`Direct3D11CaptureFrame.ContentSize` 是该帧渲染时的内容尺寸。目标 resize 时应按新尺寸 `Recreate` frame pool；复制像素时只使用 `ContentSize` 有效子矩形。[[`ContentSize` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframe.contentsize)] [[`Recreate` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.graphics.capture.direct3d11captureframepool.recreate)]

WGC 不提供“屏幕点 → frame pixel”的一键映射。`GetWindowRect` 可被 DPI 虚拟化，而 `DWMWA_EXTENDED_FRAME_BOUNDS` 是屏幕空间可见边界且不作 DPI 调整。实现应将 capture Worker 设为 Per-Monitor-V2 DPI aware，把光标点、目标可见边界与 `ContentSize` 统一到物理像素后再比例映射与 clamp；不要直接用 CSS DIP 或其他逻辑坐标裁 D3D surface。[[DPI and device-independent pixels](https://learn.microsoft.com/en-us/windows/win32/learnwin32/dpi-and-device-independent-pixels)] [[`GetWindowRect` 参考](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowrect)] [[跨进程 DPI 坐标转换](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-logicaltophysicalpointforpermonitordpi)]

多显示器、负虚拟桌面坐标、不同目标 DPI awareness、标题栏/阴影偏移不能靠固定常数，必须在后续实证矩阵内验证。

HDR 环境下若一律当作 `B8G8R8A8_UNORM` 可能过曝/泛白。Microsoft 建议 HDR capture pipeline 考虑 `R16G16B16A16_FLOAT` 并做 HDR-to-SDR tone mapping。OCR 输入最终仍应明确转成受测 BGRA8/Gray8。[[Screen capture 的 HDR 注意事项](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture)]

### 1.5 其他官方取帧 API 对照

| API | 优点 | 与本场景的冲突 | 结论 |
| --- | --- | --- | --- |
| DXGI Desktop Duplication | Windows 8+ 的高效整显示器帧；`ProtectedContentMaskedOut` 能指示受保护内容已被涂黑。[[概览](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/desktop-duplication-api)] [[frame info](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/ns-dxgi1_2-dxgi_outdupl_frame_info)] | 捕获单元是显示器，即使之后裁窗口，系统也已向进程交付了相邻窗口像素，与“仅已授权 HWND”最小数据面冲突。 | **No-Go** |
| `PrintWindow` | 早期 Windows 也可用，按 HWND 请求窗口绘制到 DC。 | 调用是同步/阻塞的，并由目标应用处理 `WM_PRINT`；这不是对任意 GPU/composition surface 的可靠生产捕获。[[`PrintWindow` 参考](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-printwindow)] | **不作为主路或默认 fallback** |
| GDI screen DC / `BitBlt` | 实现简单。 | 依赖桌面当前合成/遮挡状态，且难以从 API 能力上保证只读授权窗口。 | **No-Go** |

## 2. Microsoft 本地 OCR API

### 2.1 `Windows.Media.Ocr.OcrEngine`

`OcrEngine` 自 Windows 10 build 10240 可用，标注为 Agile 且 `ThreadingModel.Both`；`RecognizeAsync(SoftwareBitmap)` 返回文本、行/词和词坐标。这是设备内的本地识别，不需要把图像发往云端。[[`OcrEngine` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr.ocrengine)] [[`RecognizeAsync` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr.ocrengine.recognizeasync)]

但其对桌面应用有决定性限制：Microsoft 当前 namespace 文档明确指出，`Windows.Media.Ocr` **仅支持拥有 package identity 的桌面应用**，即从 MSIX 包安装/运行。普通 unpackaged Win32/Rust 进程不是受支持的部署方式。[[`Windows.Media.Ocr` namespace 备注](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr)]

语言不是应用自带模型：

- `AvailableRecognizerLanguages` 只列出设备已安装的 OCR 语言；用户可在 Windows Settings 安装新 OCR 语言包。[[API 参考](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr.ocrengine.availablerecognizerlanguages)]
- 语言包是 Windows Language Feature on Demand，不是 Glyphshift 可随应用复制的模型。[[Windows language OCR FOD](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/features-on-demand-language-fod)]
- `TryCreateFromUserProfileLanguages` 在用户语言无法匹配已安装 OCR 语言时会失败/返回空。产品必须显示“未安装识别语言”，不能悄悄换到不同语种。

图像的宽、高都必须不超过运行时 `OcrEngine.MaxImageDimension`。该上限应在运行时读取，不应把历史常见数值硬编码到合同里。Microsoft 官方 OCR 样例同样在识别前分别检查宽高。[[`MaxImageDimension` 参考](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr.ocrengine.maximagedimension)] [[Microsoft OCR 样例](https://github.com/microsoft/Windows-universal-samples/tree/main/Samples/OCR)]

Rust 可通过 Microsoft `windows` crate 消费 WinRT API，所以“没有 Rust binding”不是阻塞；但 binding 不会绕过 package identity、WinRT async 生命周期和语言包前置。[[Microsoft `windows-rs`](https://github.com/microsoft/windows-rs)]

**许可/部署结论：** 这是 Windows OS 功能，未找到允许开发者抽取并随应用再分发 Windows OCR 模型的官方授权。因此只能把它设计成“调用系统已安装组件”，不能把 Windows OCR 模型复制进应用 Bundle。这是对官方部署合同的保守推论，不是法律意见。

### 2.2 新 Windows AI `TextRecognizer`

Windows App SDK 已提供 `Microsoft.Windows.AI.Imaging.TextRecognizer`，可返回字符/词/行、多边形边界和置信度；Microsoft 将它描述为比 legacy `Windows.Media.Ocr` 更快、更准。[[Text recognition 概览](https://learn.microsoft.com/en-us/windows/ai/apis/text-recognition)] [[`TextRecognizer` API](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.windows.ai.imaging.textrecognizer)]

但截至本次调研，它不是通用 Windows OCR 回退：

- Microsoft 当前硬件矩阵中，Text Recognition 仅支持 Copilot+ PC/NPU，不支持通用 GPU 或 CPU。[[Windows AI APIs 硬件矩阵](https://learn.microsoft.com/en-us/windows/ai/apis/)]
- Windows AI APIs 需要 package identity；unpackaged app 配置不再受支持，可用 external-location identity 作为另一项分发工程。[[Microsoft Windows AI sample 前置](https://learn.microsoft.com/en-us/samples/microsoft/windowsappsdk-samples/windowsaisamples/)] [[为现有应用授予 package identity](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/grant-identity-to-nonpackaged-apps)]
- 应用必须先用 `GetReadyState` 分支 `Ready` / `NotReady` / `DisabledByUser` / `NotSupportedOnCurrentSystem`；模型需安装时，应先告知用户下载与网络影响，再调 `EnsureReadyAsync`。[[Windows AI get started](https://learn.microsoft.com/en-us/windows/ai/apis/get-started)]

它的精确 OS build、驱动、模型下载和 package manifest 组合**未在本 Slice 实机验证**。结论是将它保留为将来可选 `windows-ai-ocr` Provider，不让它决定当前的最低硬件或分发基线。

## 3. Tesseract 对照

Tesseract 为 C++ OCR 引擎，官方代码采用 Apache-2.0；其直接依赖 Leptonica 为 BSD 2-clause 风格，可选图像/压缩库还需逐项审计。[[Tesseract 仓库与许可](https://github.com/tesseract-ocr/tesseract)] [[Tesseract `LICENSE`](https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE)]

官方训练数据库也使用 Apache-2.0：

- `tessdata_fast`：整数化 LSTM 模型，官方定位是速度/准确度折中，且说明多数用户会选它。[[仓库](https://github.com/tesseract-ocr/tessdata_fast)]
- `tessdata_best`：更准但更慢、模型更大。[[仓库](https://github.com/tesseract-ocr/tessdata_best)]
- `tessdata`：混合 legacy 与 LSTM 数据，不适合为新桌面小 ROI 识别的默认最小 Bundle。[[仓库](https://github.com/tesseract-ocr/tessdata)]

安装体积不能只看 OCR DLL。以官方仓库当前单文件元数据为例：

| 模型 | `tessdata_fast` | `tessdata_best` |
| --- | ---: | ---: |
| English | 3.92 MB | 14.7 MB |
| Simplified Chinese | 2.35 MB | 12.5 MB |
| 两者合计（约） | 6.27 MB | 27.2 MB |

直接元数据：[[fast English](https://github.com/tesseract-ocr/tessdata_fast/blob/main/eng.traineddata)] [[fast Simplified Chinese](https://github.com/tesseract-ocr/tessdata_fast/blob/main/chi_sim.traineddata)] [[best English](https://github.com/tesseract-ocr/tessdata_best/blob/main/eng.traineddata)] [[best Simplified Chinese](https://github.com/tesseract-ocr/tessdata_best/blob/main/chi_sim.traineddata)]

这些数字不包含 Tesseract、Leptonica、MSVC runtime 以及实际启用的可选图像库；最终 Bundle 体积必须从 release 产物实测。Windows 官方编译指南推荐 vcpkg x64 动态或静态构建，并明确 Tesseract 需要 Leptonica，这意味着 Rust 项目不只是添加一个纯 Rust crate。[[Tesseract compilation guide](https://tesseract-ocr.github.io/tessdoc/Compiling.html)]

FFI 成本是真实的：官方 C API 要求管理 `TessBaseAPICreate` → `Init` → `SetImage` → `GetUTF8Text` → `TessDeleteText` → `End/Delete` 的完整生命周期；图像和返回 UTF-8 字符串的内存归属也有明确边界。[[Tesseract C API](https://github.com/tesseract-ocr/tesseract/blob/main/include/tesseract/capi.h)]

生产封装至少需要：

- Rust RAII wrapper 与 panic-safe cleanup；
- 每个 Worker/并发槽位独立 engine instance，不把可变 C++ handle 无锁共享；
- BGRA8/Gray8 到 Tesseract stride/bytes-per-pixel 的精确适配；
- 语言模型路径、缺失/损坏模型、超时/取消和长文本有界输出；
- 正式制品中的 Apache-2.0/BSD notice 与所有 transitive dependency 许可清单。

**Tesseract 是“可进入证据验证”，不是已被选定。** 必须用两个 gate 目标比较 `fast`/`best` 的准确率和冷/热延迟，再决定是否承担 FFI 与 Bundle 成本。

## 4. 推荐与 No-Go

| 选项 | 当前判定 | 理由 |
| --- | --- | --- |
| WGC `CreateForWindow` + 单帧小 ROI | **推荐为 Target Frame Provider 候选** | 按已知 HWND 约束数据面，官方 Win32 支持，可用 Rust 调用；但必须保留边框、超时和黑帧 fail-closed。 |
| Tesseract + 最小 `tessdata_fast` | **推荐为首个 OCR 评估候选** | 可随应用离线分发，引擎/官方模型为 Apache-2.0；但尚未通过本项目的准确率、延迟、体积和依赖许可验证。 |
| `Windows.Media.Ocr` | **当前 No-Go；分发转为有 identity 后可重评** | 普通 unpackaged desktop 不受支持；语言可用性由 Windows FOD 决定，不是应用可自带的可预测模型集。 |
| Windows AI `TextRecognizer` | **当前 No-Go；未来可选 Provider** | package identity + 当前 NPU 专用硬件矩阵，无法覆盖通用 Windows 安装基础。 |
| Desktop Duplication / GDI screen capture | **No-Go** | 捕获整个显示器/桌面，数据面大于已授权 HWND。 |
| 从 UIA 空结果自动启动 OCR | **No-Go** | 违反显式用户意图；也会把密码/保护区域的 fail-closed 语义错误变成像素旁路。 |
| 跨目标或全屏 OCR | **No-Go** | 超出授权范围和已有 Visual Acquisition 合同。 |

## 5. 必须先取得的真实目标证据

下列 gate 决定是否将 OCR 提升为默认产品回退，不阻塞使用普通授权可见文字完成工程验证；不应以“技术可行”代替“产品有收益”。

### 5.1 两个独立授权目标

至少两个不同渲染/控件架构的真实目标，每个均要证明：

1. 用户显式授权目标和单次取点；
2. UIA Point 和可用 Text Range/Name/Value 通道在该点没有可发布文本，并且不是密码、保护属性或权限拒绝；
3. 同一点的授权窗口帧中能人工确认文字像素可见；
4. WGC 实际返回非空、非全黑/透明帧，且 ROI 不含授权 HWND 以外像素；
5. 候选 OCR 在有界 ROI 中返回与可见文字相符的结果，并记录置信度、冷/热延迟与峰值内存。

仅有“UIA 拿不到”或仅有“截图中看到字”都不够；两者必须在同一授权交互点成对证明。

### 5.2 边界与故障矩阵

- DPI：至少 100% 与一个非 100% 缩放；
- 多显示器：包含一次负虚拟桌面坐标；
- 窗口状态：前台、被遮挡、resize 期间、最小化；
- 捕获保护：至少一个受保护/空帧 fixture 或授权实例，证明不调 OCR 且稳定 fail-closed；
- 生命周期：用户取消、首帧超时、目标关闭、应用退出均释放 session/D3D/OCR 资源；
- 组合保证：UIA 成功时捕获和 OCR 调用次数必须为 0；
- 隐私：默认不落盘帧/ROI/原文，原始本机证据只能进入本地测试证据区。

### 5.3 发布与许可 gate

- 明确产品是继续 unpackaged，还是引入 MSIX/external-location package identity；
- 若用 Tesseract：锁定版本、建立可重复 Windows x64/arm64 构建、生成 SBOM/notice，审核 Leptonica 和所有链接依赖；
- 只随包携带产品真正支持的语言模型，并为每个额外语言显示下载体积和许可来源；
- 对正式 Bundle 实测安装体积、冷启动、首次 OCR、热 OCR、峰值内存和取消延迟；
- 若用 Microsoft OCR：证明 package identity、目标 Windows 版本、语言包缺失与用户修复流程，不随包复制 OS 模型。

## 最终决策

**现在：工程验证 Go；产品提升 No-Go。**

**取得两个成对的真实目标证据后：** 先做 `WGC CreateForWindow + bounded ROI + Tesseract tessdata_fast` 的可抛弃生产验证，与现有 Visual Acquisition 合同对接；识别率、延迟、体积和许可全部达标后才可选定引擎。

**如果项目之后决定引入 package identity：** 在同一真实证据集上加测 `Windows.Media.Ocr`，再用“准确率/延迟/分发可预测性/语言可用性”决定是否替代 Tesseract。新 Windows AI `TextRecognizer` 继续是 runtime-capability-gated 的可选加速 Provider，不是基线。
