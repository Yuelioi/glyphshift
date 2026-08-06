# 生产 OCR 回退

Status: In progress

Research: [Windows 生产 OCR 显式回退选项](../references/windows-production-ocr-options.md)

## Outcome

只有用户明确选择且 UIA 无法提供结构化文字时，Glyphshift 才截取目标授权区域，在内存中执行 OCR，
并复用既有翻译与外部呈现流程。

## Scope

- 实现授权 Target Frame Provider 与生产 OCR Adapter，复用已完成的 Region、裁剪、坐标、置信度、
  超时、取消和部分成功合同。
- 默认只处理点击附近的有界区域；跨目标、受保护窗口、黑屏和截屏权限失败关闭。
- 截图默认不落盘，OCR 原文遵守与 UIA 相同的长度、隐私和本地证据规则。
- UIA 与 OCR 返回同一产品结果类型；来源和置信度可见，但用户不需要进入另一套页面。

## Engineering entry gate

- [x] UIA 取词产品入口已经交付并通过定向产品与 Windows 合同。
- [x] 至少一个授权运行目标有可人工确认的可见文字，可用于 Capture、ROI 与 OCR 基线验证。
- [x] 已选定 WGC `CreateForWindow` 与 Tesseract `tessdata_fast` 作为首个工程验证候选；这不等同于
  已批准进入发布 Bundle。

## Product promotion gate

- [ ] 至少两个明确授权的真实目标证明 UIA 无文本而像素文字可见。
- [ ] OCR 引擎的识别率、离线制品、许可证、体积与性能满足产品发布要求。
- [ ] UIA 成功时不 Capture；UIA 无结果后仍须由用户明确选择 OCR。

## Gate review — 2026-08-07

- 现有 Visual/OCR Fixture 已证明 Region 交集裁剪、坐标、DPI、负坐标、置信度与失败语义，但没有证明
  生产 OCR 对真实目标有收益。
- 当前授权证据中，已有目标能由 UIA 取得公开文本；其余已知失败分别属于进程架构或激活问题，不能
  推断为“UIA 无文本、像素文字可见”。
- WGC `CreateForWindow` 可作为已授权 HWND 的单帧 Frame Provider 候选；Desktop Duplication、GDI
  全屏捕获会扩大数据面，不进入本路线。
- `Windows.Media.Ocr` 要求 package identity；新 Windows AI Text Recognizer 还要求 package identity
  与当前 NPU 能力门槛。Tesseract 可进入离线验证，但尚未通过真实识别率、延迟、Bundle 体积与完整
  依赖许可审核，不能视为已选定。

Decision: **Go for engineering validation；No-Go for product promotion。** 可以实现隔离的 Capture Worker、
OCR 适配和本地 Bundle 验证；在提升门槛满足前不默认发布，也不以自动 OCR 绕过 UIA 的权限、密码或
受保护内容失败语义。

### Candidate screening

- BlueArchiveAutoScript 的官方源码表明其 GUI 使用 PyQt5 与 PyQt-Fluent-Widgets。授权运行实例的单个
  UIA observe-only 烟测在 5 秒内取得 95 条唯一公开文本，Worker health 为 Healthy，定向测试 1/1
  通过；原文与本机身份只保存在本地测试证据区。
- 该结果证明应用整体已有较强的结构化文本暴露，但不妨碍它作为 OCR **工程样板**：可对已知可见文字
  强制执行视觉路径并校验 Capture、ROI、识别、坐标和清理。只有产品提升时，才要求某个 Fluent 自绘
  或画布区域同时证明 UIA Point / Text Range 无可发布文本、同一点像素文字可见。
- 本地 OCR 样板已跑通：授权窗口内取得 60 个文本块，平均置信度 0.962，普通导航文字与横幅图片文字
  均有命中。截图、OCR 原文和结果坐标只保存在本地 evidence；该样板证明目标内容适合视觉验证，但
  使用的是本地原型捕获与引擎，不替代后续 WGC、生产引擎、Bundle 和生命周期合同。

## Implementation order after gate

1. 先以授权目标的可见文字建立 Capture/OCR 基线；运行参数和原始证据只保存在本地测试证据区。
2. 在独立 Worker 中实现 WGC `CreateForWindow` 单帧、短超时和有界 ROI；不污染平台中立 OCR crate。
3. 用同一证据集比较候选引擎的识别率、冷/热延迟、峰值内存、模型体积与许可证。
4. 选定引擎后接入现有一次性 Acquisition Worker / Bundle，保留目标 grant、取消和 fail-closed 合同。
5. 只在 UIA 稳定无结构化文本后向用户展示“尝试 OCR”，结果复用现有翻译表面。

## Verification

- [ ] 结构化 UIA 成功时不会调用截图或 OCR。
- [ ] 用户显式回退后可从合成与授权真实区域取得有界 OCR 结果。
- [ ] DPI、多显示器、负坐标、权限、密码/受保护区域和退出清理通过。
- [ ] OCR 引擎制品、许可证、Bundle 校验、安装体积与性能预算明确。

## Next

先以 BlueArchiveAutoScript 的可见文字完成 WGC + 候选 OCR 本地基线，再寻找两个独立授权目标的
UIA 真实缺口作为产品提升证据。发布门禁前不实现区域框选、语言包管理，也不扩张持续 Probe payload。
