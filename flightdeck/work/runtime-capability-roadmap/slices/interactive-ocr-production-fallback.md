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

- [x] 至少两个明确授权的真实目标证明 UIA 无文本而像素文字可见。
- [x] OCR 引擎的识别率、离线制品、许可证、体积与性能满足产品发布要求。
- [x] UIA 成功时不 Capture；UIA 无结果后仍须由用户明确选择 OCR。

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
- WGC 单帧样板也已在同一授权窗口通过：`CreateForWindow` 返回的帧与窗口边界尺寸一致，随后离线 OCR
  仍得到 60 个文本块、平均置信度 0.962，并命中导航与横幅文字。当前系统不支持显式设置捕获边框和
  secondary-window 属性，生产 Provider 必须在这些可选属性不可用时保持 WGC 默认语义，而不是降级为
  全屏/GDI 捕获或误报主捕获能力不可用。
- 独立 Windows Frame Provider 已实现并通过包级测试与授权窗口烟测：它只枚举授权 PID 的可捕获顶层
  窗口，按选区交集选择目标，复验进程启动时间与窗口归属，使用 WGC 取得一次性内存帧，并对目标漂移、
  尺寸漂移、空白/透明帧失败关闭。窗口遮挡层不会再通过 `WindowFromPoint` 抢走目标归属。
- Frame Provider slice 自审通过：包级测试 1/1、授权窗口烟测 1/1、包级 Clippy 通过；没有持久化帧、
  OCR 原文或本机进程信息。短超时、取消和进程 grant 仍由后续监督 Worker 集成负责，不能由 Provider
  单独宣称完成。
- Tesseract 5 的候选引擎边界已接入独立 Worker crate：它通过稳定 C ABI 从已验证绝对路径动态加载，
  依赖搜索只允许 DLL 所在目录与 System32，拒绝当前目录和用户 PATH；语言名、模型存在性、ABI 主版本、
  词块数量、UTF-8 总量、坐标与置信度均有界。构建期不下载原生源码或语言模型。
- 使用固定 vcpkg 端口的 Tesseract 5.5.2 与固定提交的 `tessdata_fast` 中英模型完成本地真实链路：引擎
  初始化约 109 ms，首轮 WGC + OCR 约 1.28 s，热轮约 1.22 s，两轮均返回 170 个窗口内词块。
  确定性测试 5/5、真实链路 1/1、包级 Clippy 通过；原文和本机路径未进入跟踪文件。
- 当前官方 vcpkg 动态依赖闭包为 18 个 DLL、约 15.8 MB；中英模型约 6.6 MB，合计约 22.4 MB，
  只保留为可运行上界。固定 vcpkg 与 `tessdata_fast` 提交的精简构建关闭 curl、libarchive、OpenCL、
  ScrollView、调试字体和无关图片格式，只保留 Tesseract、Leptonica、libpng、zlib 四个 DLL 与中英模型；
  六个运行制品约 12.2 MB，比上界缩小约 45%。每个模型固定 SHA-256，构建脚本拒绝源码修改、版本漂移、
  额外 DLL 和本地目录之外的输入输出。
- Windows process grant 校验已下沉为 UIA/OCR Worker 共用 Module；两条路径都先校验 PID、创建时间与
  integrity，再接触目标内容。OCR Worker 只接受 `Region + VisualOnly`，并由既有一次性 Host 负责
  8 秒超时、取消和异常进程回收。
- 正式 Host → wire → grant → WGC → Tesseract 的本地链路已通过：候选 Bundle 共 21 个文件，单次
  返回 170 个有界词块，约 2.04 秒。原始文字、路径和进程身份只保存在本地 evidence。
- 未发布 Runtime Bundle 已直接升级为 `/3`：每个 acquisition worker 必须显式声明 `support_files`，
  executable 与所有依赖逐文件校验路径和 SHA-256，名称按 Windows 大小写语义去重。没有保留 `/2`
  兼容分支；当前体积偏大的 vcpkg 闭包仍不进入标准发布 Bundle。
- Worker/Bundle slice 自审通过：OCR 包确定性测试 5/5、集成测试 2/2、真实 Host 链路 1/1；Bundle
  合同 10/10；共享 grant 2/2；UIA Worker 单元与真实合成控件回归各 1/1；相关包 Clippy 与构建脚本
  PowerShell 语法检查通过。

## Implementation order after gate

1. 先以授权目标的可见文字建立 Capture/OCR 基线；运行参数和原始证据只保存在本地测试证据区。
2. [x] 在独立 Worker 边界实现 WGC `CreateForWindow` 单帧与有界 ROI；不污染平台中立 OCR crate。
   边框与 secondary-window 配置按运行时能力可选，核心 HWND 捕获能力与可选属性分别报告。
3. 用同一证据集比较候选引擎的识别率、冷/热延迟、峰值内存、模型体积与许可证。Tesseract 上界基线
   已完成；精简制品、峰值内存和产品门槛样本仍待补齐。
4. [x] 将候选引擎接入现有一次性 Acquisition Worker / Bundle，保留目标 grant、取消和 fail-closed
   合同；发布 Bundle 仍等待精简原生制品与许可清单。
5. [x] 只在 UIA 稳定无结构化文本后向用户展示“尝试 OCR”，结果复用现有翻译表面；标准 Bundle
   未包含候选引擎时说明原因且不展示不可执行按钮。

## Product integration review — 2026-08-07

- Desktop Acquisition 现由内部 selection 调度统一 Point/Region：Point 固定 `StructuredOnly`，Region
  固定 `VisualOnly`；每次仍发现短期 Runtime、重新签发 grant，不借用 Workflow/Probe Session。
- 后端保存一次性 OCR eligibility。只有同一软件和词典的上一笔结构化请求明确返回 `acquisition.no_text`
  才能武装 `visualOcr`；成功、取消、普通重试、关闭或选择变化都会清除资格。Arm request 必须显式携带
  acquisition mode，不接受缺字段的旧形态。
- OCR 使用光标周围 640×240 逻辑像素的有界 Region，并继续由 Worker 与授权窗口求交；没有自由框选、
  全屏 Capture、截图落盘或自动 OCR。结果表面按 provenance 显示 UIA/OCR 来源。
- Runtime Bundle capability 决定 OCR 按钮是否可见。标准构建仍只含 UIA Acquisition Worker；本地开发
  只有显式 `-IncludeOcrCandidate -OcrSupportRoot <verified-ocr-support-root>` 才把候选 Worker、DLL 闭包
  和中英模型逐文件写入 `/3` manifest。Release 构建不会继承该候选开关。
- Slice 自审通过：Desktop Runtime acquisition 15/15、Shell interactive translation 11/11、相关两包
  Clippy、前端 production build、交互页面 Playwright 4/4、OCR 候选 Bundle 构建与 Loader 合同 1/1。
  候选 Bundle 为 35 个文件，其中 OCR Worker 支持文件 20 个；标准 Bundle 仍为 14 个文件且只声明 UIA。

## Slim artifact and licensing review — 2026-08-07

- 两个过度裁剪方案已明确淘汰：完全移除图片库会产生大量 Leptonica PNG 告警；直接保留 Tesseract
  `ImageData` 内存 Pix 虽可消除告警，但真实单轮延迟退化到约 6.8 秒，逼近 8 秒监督超时。
- 最终候选只恢复 libpng 与 zlib，并关闭调试字体。授权 WGC 样本连续两轮均返回 170 个词块，初始化约
  106 ms、首轮约 1.30 秒、热轮约 1.18 秒；缺失图片编解码器的告警已消除，Tesseract 自身仍可能输出
  字距估算诊断。监督 Host 链路返回 167 个词块，约 1.46–1.99 秒，一次采样的 Worker 峰值工作集约
  99.4 MiB。
- `build-ocr-runtime-support.ps1` 现从固定且干净的本地 checkout 生成支持集，只允许 4 个 DLL、2 个模型、
  5 份许可证、第三方 notice 与来源/摘要清单。所有机器输入输出都留在本地测试目录，生成文本不含本机
  路径。Runtime Bundle 构建要求这 13 个支持文件全部存在并逐件写入 `/3` manifest。
- Slice 自审通过：支持集 13 个文件、12,236,070 bytes；候选 Bundle 29 个文件、OCR support 13 项；
  Bundle Loader 1/1、真实监督 Host → wire → grant → WGC → Tesseract 1/1，构建脚本语法与
  `git diff --check` 通过。标准 Bundle 仍不包含 OCR。

## Windows capture boundary review — 2026-08-07

- 确定性真实 WGC Fixture 暴露并修复了普通带边框窗口的尺寸基准错误：WGC 帧对应 `GetWindowRect`，
  不能拿 DWM extended frame bounds 校验，否则常规窗口也会被误报 Provider unavailable。
- Per-Monitor V2 进程中，部分位于负坐标的窗口完成一次内存抓帧并保持全局 anchor；设置
  `WDA_EXCLUDEFROMCAPTURE` 的窗口会在抓帧前通过 display affinity 稳定返回 Permission denied，不向
  OCR 引擎暴露像素。两种 Fixture 均在 3 秒有界退出合同内完成清理，边界测试 2/2 通过。
- 修改后授权真实目标的 WGC + Tesseract 回归 1/1：167 个词块，首次约 1.26 秒、热轮约 1.18 秒；相关
  三包 Clippy 通过。当前机器只有一个物理显示器，因此真实跨显示器排列及不同缩放组合仍未验证。

## Paired UIA/OCR evidence — 2026-08-07

- 新增默认忽略的参数化证据测试：先在授权窗口内执行生产 WGC + Tesseract，再只对 OCR anchor 中心执行
  生产 UIA Point。测试只输出数量，不输出识别原文、路径、PID 或窗口身份；机器参数和日志均留在本地
  测试目录。
- 第一个独立授权 PyQt 目标取得 167 个 OCR 块；抽样 64 个同点 UIA 请求，其中 32 个明确返回
  `NoText`、32 个返回结构化文字、其他错误为 0。它证明同一应用中结构化和像素路径需要按点击位置
  互补，而不是按进程品牌二选一。
- 第二个独立授权图片查看目标加载确定性文字图后取得 6 个 OCR 块；同点 UIA 中 5 个明确返回
  `NoText`，另 1 个无法归属，不计入缺口证据；6 个预期词也全部被 OCR 命中。两个独立目标的产品提升
  样本门槛完成；相关测试与 OCR Worker 全 target Clippy 通过。

## Verification

- [x] 结构化 UIA 成功时不会调用截图或 OCR。
- [x] 用户显式回退后可从合成与授权真实区域取得有界 OCR 结果。
- [ ] DPI、多显示器、负坐标、权限、密码/受保护区域和退出清理通过。
- [x] OCR 引擎制品、许可证、Bundle 校验、安装体积与性能预算明确。

## Next

工程、产品提升样本、精简制品、许可/性能门禁及单显示器负坐标/受保护窗口/退出边界已完成。下一步只
在可用环境补真实跨显示器排列与不同缩放组合；完成前 OCR 不进入标准 Release Bundle。发布门禁前不
实现自由区域框选、语言包管理，也不扩张持续 Probe payload。
