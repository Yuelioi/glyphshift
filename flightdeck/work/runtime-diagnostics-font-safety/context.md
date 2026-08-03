# Work Context

## 已有事实

- Workflow Target 是 Software、parallel Adapter Plan、有序 Dictionary 和可选内联 Font Policy 的
  唯一组合根。
- Dictionary `/2` 只保存非空且 source 唯一的 `source + translation`，不接受 Location、Context、
  字体、Hook、Adapter 或探针证据。
- Font Policy 默认 coverage 是 `dictionary_matches`；`all_observations` 会作用于所选且支持字体替换
  的 Adapter 捕获的全部文字。
- Runtime Publication 必须完整、原子地应用后才 ACK Generation；内部已有 Translation Snapshot 和
  Font Policy 内容摘要。
- Capture 已使用有界非阻塞队列、定时 checkpoint 和丢弃计数；Native Adapter 已覆盖重入和 panic
  fail-open。
- Probe Run 绑定一个 Dictionary，Observation Index 只保存技术证据；行内翻译直接修改绑定的
  Dictionary。
- Decision Trace 不分配 Adapter、原文或 Route 字符串；Trace 状态与 Text/Font 结果是可复制的
  稳定值，Runtime Kernel 只在显式开启后复制 Observation 到 256 条环形缓冲。
- Runtime Publication identity 是规范编码的 SHA-256，覆盖 Route、Translation Snapshot、Font
  Policy 和 Generation；Controller ACK 必须同时匹配 Generation 与 identity。
- GDI Adapter 在实际 `LOGFONT` 报告 `SYMBOL_CHARSET` 时保留原字体；GDI+ 目前没有等价的可靠
  原字体类别信号。
- 完整 Native 注入合同以同一 `all_observations` Font Policy 验证：普通 GDI 文字换字体、GDI
  `SYMBOL_CHARSET` 像素保持、GDI+ Symbol 字体像素改变，停止 Runtime 后三类绘制全部恢复。
- GDI `ExtTextOutW` 使用 `ETO_GLYPH_INDEX` 时，输入是当前字体的 glyph ID 而不是 Unicode；只有在
  新字体实际选入后，Adapter 才能使用已解码原文、清除 glyph-index 标记与旧 spacing 重新绘制。
- Target Runtime 诊断使用固定上限 JSON 输出缓冲；Controller 每次查询只取走最近有界批次，
  dropped 数跨层保留。Desktop 仅在用户打开工作流诊断期间开启采集，每秒读取并在前端保留最近
  256 条。
- Controller 拒绝以有限稳定类别跨过 stdio、Protocol、Host 与 Desktop 边界；普通产品 UI 不显示
  原始插件错误串，但必须区分目标访问、组件加载/兼容、超时和一般拒绝。
- Capture 激活失败后不得保留未激活 Runtime；下一次“连接并继续”必须重新发现目标和 Controller。

## 本主题决定

- Decision Trace 是诊断 Interface，不参与匹配，不改变 Replacement Decision，也不成为持久业务
  状态。
- Trace 必须有界、非阻塞、可关闭；队列满时丢诊断，不得阻塞宿主绘制线程或改变 fail-open 结果。
- Trace 优先保存稳定 reason code、Adapter、Publication identity、字典命中结果和字体决策结果；
  只有 Adapter 已可靠提供的额外证据才能进入可选 envelope。
- 窗口句柄、DC、调用栈、坐标和内部 Route 不进入 Dictionary，也不成为跨重启匹配主键。
- `dictionary_matches` 保持默认。`all_observations` 使用明确的高影响提示，不采用自动超时，因为
  Workflow 是持续期望状态。
- 图标或符号字体保护必须建立在 Adapter 可观察、可测试的字体身份上；没有真实信号时只诚实提示，
  不以猜测黑名单伪装保护能力。
- 字体未成功替换、glyph-index 无法可靠解码或原字体受到保护时，GDI Adapter 必须保留原参数
  fail-open；不得让新字体解释旧字体的 glyph ID。
- 已有 digest 不再复制成第二套 Snapshot Artifact；只有当它能区分 Generation 相同但内容异常时，
  才把 publication identity 扩展到 Runtime ACK 和桌面诊断。
- 普通产品 UI 不展示 PID、内部 Adapter ID、Route、DLL 路径或原始调用信息；详细 Trace 只进入本地
  诊断表面，实机原始证据只存 `target/local-test/evidence/`。
- 诊断表面把内部 Adapter ID 映射为 Runtime Bundle 的公开名称；缺少展示信息时使用通用占位，
  不回退显示内部 ID。Publication identity 可作为本地诊断事实显示，Translation/Font digest 保留在
  查询合同中但不占用普通表格列。

## 不在范围

- Process Family、多进程发现、DirectWrite、UIA、OCR 和新图形 API Adapter。
- Target Execution 的单写多读租约或多观察订阅模型。
- 模糊匹配、substring fallback、AI 运行时匹配和通用 Renderer。
- Probe Overlay、Dictionary Context、事件溯源和仅为减少 crate 数量进行的 Module 合并。

## 完成标准

- 合成 Host 能解释 Pass、Text-only、Font-only 和 Text+Font，并证明诊断关闭或溢出不影响决策。
- Runtime mismatch 能报告足以定位发布内容的稳定身份，而不是只显示模糊的“未应用”。
- Workflow UI 对两种 coverage 的影响范围表达准确；危险模式不能被误认为只影响已翻译文字。
- 若交付字体排除能力，必须有图标/符号字体和普通字体的合同测试；否则明确记录为未支持。
- Rust workspace、fmt、Clippy、架构检查和仓库 Playwright 全部通过；真实目标软件证据保持本地忽略。
