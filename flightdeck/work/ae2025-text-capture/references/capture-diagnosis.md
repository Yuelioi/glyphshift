# AE 2025 界面文字获取诊断

采集阶段结论：AE 2025 的新版界面改变了本轮面板文字的绘制路径，暴露了 Glyphshift DirectWrite Adapter 将“可观察”与“可替换”混在一起判断的缺陷。已在现有 Adapter 内修复采集，没有新增私有 Adobe Adapter，也没有发布或更新已安装桌面应用。后续用户试用推动了统一 Typography 的替换支持，当前实现与实机像素验收见[译文显示验证](typography-translation.md)；下文保留采集阶段的保护边界与结果。

## 修复结果

[`replacement_layout`](../../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs) 现在先发布实际绘制的原文，再在需要应用译文时检查布局格式。非空 Typography、局部格式等原有替换保护保留；不支持的替换不会声明下层绘制的替换作用域。没有将测量调用直接当作可见文字。

同源 Runtime Bundle 通过 13 个 Adapter 加载校验。AE 25.1 的原生产采集用例从 54 条 GDI 菜单观测变为 92 条（38 条 DirectWrite 面板及 54 条 GDI 菜单），原缺失的三个面板标签全部捕获。真实单观察、观察加替换模式均得到 178 次回调、38 种原文，原来的差异消失。

新增[原生复杂格式合同](../../../../crates/adapters/platform/native-host/tests/native_directwrite_capture_formatting_contract.rs) 在旧实现失败，修复后 x64/x86 均通过；每个架构验证 24 个组合，覆盖 Typography、局部字号、DC/位图目标、两种宿主接口、保持/替换/错误决策，要求原文可观察且原始像素保持。既有正常文字替换与停用合同也在两个架构通过。实机生产采集与模式切换两个 Playwright 用例均转绿。

下文保留修复前的诊断依据。此次解决的是获取内容，尚未扩展复杂排版的中文替换，也没有完成其像素、更新和恢复验收。

## 官方资料与实测范围

Adobe 官方宣布 AE 新界面采用 Spectrum，并在 Windows 使用 GPU 加速；25.0 发布汇总列出了现代化界面。设计系统声明本身不能证明具体文字 API，底层路径由下面的版本对照确认。[Adobe Beta 发布](https://blog.adobe.com/en/publish/2024/04/11/adobe-after-effects-beta-introduces-new-3d-workflows-user-requested-features-time-for-nab-2024)、[25.0 发布汇总](https://blog.adobe.com/jp/publish/2024/10/16/cc-creativecloud-october-2024-update-list)。完整公开资料见[官方来源研究](adobe-ui-research.md)。

实测使用 AE 24.6.2 与 25.1 的空白工程，启动后附加并对主窗口重复重绘。原始窗口、模块、字符串和日志只保存在本机忽略证据区。结果适用于已观察的面板与菜单，不外推到全部 25.x、插件面板、所有效果参数或 macOS。

| 本轮文字入口 | AE 24.6.2 | AE 25.1 |
| --- | --- | --- |
| Adobe Drawbot 的完整字符串入口 | 非零命中 | 非零命中 |
| GDI+ `GdipDrawString` | 721 次 | 0 次 |
| DirectWrite `CreateTextLayout` | 0 次 | 8310 次 |
| Direct2D `DrawTextLayout` | 0 次 | 534 次 |

这些计数用于确认路径，窗口布局与重绘次数不同，不能用于比较性能或覆盖百分比。AE 25.1 的生产 Runtime 捕获了 54 条唯一 GDI 菜单观测，三个可见面板标签均缺失；两轮结果一致。由此排除整体注入或采集传输完全失效。

## 修复前的最小故障

通过真实 Native ABI 对同一 DirectWrite Adapter、同一进程与同一重绘动作，只改变 feature bits；回调始终返回保持原文，未应用翻译：

| 激活能力 | 收到的绘制回调 | 唯一原文 |
| --- | ---: | ---: |
| `TextObserve` | 178 | 38 |
| `TextObserve + TextReplace` | 0 | 0 |

修复前，已安装的发布 Adapter 与当时 checkout 定向编译的 Adapter 表现相同。这不是仅存在于旧安装包的问题，也不要求启动前注入；临时观察探针在启动后就能取得完整原文。

本机 Playwright CLI 用例分别覆盖生产 Runtime 面板采集、两版本文字入口以及单能力与双能力切换。修复前失败的两个用例保持原有预期，修复后通过，没有将断言改成接受缺失结果。辅助工具及其配置仅供本机复现，不是仓库构建的必需资源。

可移植的产品测试入口为 `cargo test -p glyphshift-desktop-runtime --release --test windows_runtime_contract authorized_host::desktop_runtime_captures_an_authorized_real_host -- --ignored --exact --nocapture`，运行前按仓库要求初始化 Cargo 缓存及显式授权目标、Runtime Bundle、输出目录环境变量。此已有测试只报告采集数量，必须在外层补充可见面板原文的精确断言，不能把它单独通过视作兼容性通过。

## 修复前根因对应的代码

1. [`capture_runtime_spec`](../../../../crates/product/desktop-backend/src/software.rs) 为支持替换的 Adapter 同时请求 `TextObserve` 与 `TextReplace`，使收集与预览共享运行配置。
2. [`replacement_layout`](../../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs) 发现替换能力处于开启状态后，先调用 `layout_is_uniform`；检查失败立即返回，尚未执行 `decide(&record.source)`，观察回调也因此缺失。
3. `layout_is_uniform` 除了检查全文字体、范围等，还要求 `GetTypography` 返回空对象，即 `typography.is_none()`。
4. 对三个 AE 25.1 可见面板标签读取实际布局属性：所有被检查的属性范围覆盖全文，drawing effect 与 inline object 均为空；`Typography` 对象非空。因此这一条件足以解释它们被拒绝。

本轮发现的是“替换不支持时连采集也跳过”。不能据此声称所有带 Typography 的布局都可以安全替换；更不能仅移除保护而丢弃目标的排版属性。

## 排除的假设

- **仅因硬件 DeviceContext 使用不同绘制函数**：本轮比较发现 DeviceContext、DC 与兼容位图目标共用同一 `DrawTextLayout` 实现；不能用“新增一个硬件入口”解释或修复本次零观测。各别名计数不能相加。
- **必须启动前注入才能读到文字**：启动后重绘已捕获完整字符串，并可将创建的布局关联到实际绘制。
- **已全面换成 Skia、Qt 或 AGM**：没有这样的证据；AGM 相关导出存在但本轮未命中。Drawbot 在两个版本均存在，是可观测的上层入口，不能当作 2025 新增框架。
- **需要恢复归档 UIA/OCR**：现有 DirectWrite 观察模式已能读到目标文本，本轮未构建或执行归档组件。

## 后续替换支持的边界

采集修复已拆开原文观察和替换资格判断：复杂布局仍发布绘制观测，但不能替换时继续调用原始绘制。布局到原文的关联、重入防护和来源语义保留。

另行评估全文 Typography、locale、字体与布局属性的复制，确认可以等价保留后再扩大替换支持。诊断也应区分“已生成替换决策”和“实际采用替换布局”。当前研究没有做中文像素、下一代更新或停用恢复验收，因此没有宣称 AE 2025 已完成翻译支持。

采集已用合成宿主和真实 AE 25.1 面板复验。后续扩大替换支持还应覆盖新弹出对话框，并单独验证翻译可见、更新与停用恢复。需要桌面审阅时使用 `scripts/review-app.ps1` 同源构建。
