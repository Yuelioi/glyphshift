# Unity Mono Standard UI 生产接入

Status: Complete

## Goal

把已经通过双真实正例、同后端负例和恢复门禁的 Unity Mono TMP/uGUI Standard UI Adapter 接入正式
Catalog 与 Runtime Bundle，使 Probe 能选择、解释并部署该技术；生产接入不得扩大到 IL2CPP、UI Toolkit、
NGUI 或 TMP `SetText(...)`，也不得为未发布的旧原型名增加兼容分支。

## Delivery

- [x] 对照一个现有 framework Adapter 走完整生产路径：package artifact、Catalog descriptor、Runtime Bundle、
  中文技术名称/边界和官方文档 URL 均使用 `unity-mono-standard-ui` 身份。
- [x] Probe/软件技术选择只在 Windows x64 Target Process 场景暴露 `TextReplace`；激活失败继续使用 runtime
  真实 ACK，不因检测到 Unity/Mono 模块就冒充支持。
- [x] 正式 Bundle 构建与目录/manifest 合同能发现 native DLL，release/dev 两种包不引用本机路径或测试证据。
- [x] 运行 Catalog、Bundle、Desktop API/UI 的最小定向回归，并用仓库 Playwright CLI 验证技术文档入口；
  不默认运行全仓测试。
- [x] 使用正式 Bundle 复核至少一个 Standard UI 正例的首代/二代/停用恢复和 NGUI-only 拒绝，再完成
  生产接入自审。

## Current

生产接入已完成。Debug/Release Runtime Bundle 均构建成功并通过 Desktop Runtime 实际打开；Catalog 将其识别
为 Windows x64 Target Process 的 `TextObserve` / `TextReplace` Adapter，前台显示“Unity Mono 标准界面”、
精确支持边界和 Unity 官方 Mono 文档入口。architecture package 白名单、总测试脚本的 native artifact 预构建、
native-host 非 Mono 拒绝合同均已同步。

正式 Debug Bundle 在一个 Standard UI 正例完成首代、generation 2 和停用恢复的可见验收，捕获 10 条唯一
原文并正常退出；NGUI-only 同后端负例继续拒绝并正常退出。所有真实证据仍只在 `local-test/`。

## Next

- Unity Mono Standard UI lane 已可交付。保持现有边界，等待新的授权真实软件缺口，不主动扩大到 IL2CPP
  或 TMP `SetText(...)`。
- IL2CPP attach 仍等待一份同后端自绘文字负例；不能用本次 Mono 负例代替。

## Decisions

- 生产身份固定为 `windows.unity.mono.standard-ui`，Apply Model 为 `RetainedObject`，能力为可协商的
  `TextObserve` / `TextReplace`，默认实时翻译流程只请求 `TextReplace`。
- 技术说明必须明确“Unity Mono + TMP/uGUI 属性 `set_text` + Windows x64”，不能只写“Unity”。
- 原型已在发布前改名，不保留 `standard-ui-observe` 或 `unity-mono-observe` alias。
- 文档入口使用 Unity 官方当前 Mono scripting backend 手册；它说明后端身份，产品 summary 另行明确
  TMP/uGUI 属性范围与所有排除项。

## Verification

- architecture checks：通过。
- native-host `native_unity_mono_standard_ui_activation_contract`：1 passed，非 Mono 进程可靠拒绝。
- 相关 4 个 package 的 Clippy `--all-targets -- -D warnings`：通过；Unity/ABI 定向测试 32 passed。
- Debug 与 Release Runtime Bundle：构建成功；Desktop Runtime 本地目录合同均确认可打开、可列出 Adapter、
  可识别 `TextReplace` 与官方文档 URL。
- Playwright CLI `tests/settings.spec.ts`：6 passed，中文技术卡与文档按钮可见并打开预期 HTTPS URL。
- 正式 Debug Bundle：Standard UI 正例首代/第二代/停用恢复可见，10 条捕获、正常退出；NGUI-only 负例
  `activation_rejected=true`、正常退出。
- 未运行全仓测试；机器输入、日志、截图、Bundle 产物和原文均留在 `local-test/`。

## Boundaries

- 本 Slice 不实现 IL2CPP、UI Toolkit、NGUI、TextMesh、自绘 Mesh/Sprite/texture 或 TMP `SetText(...)`。
- 不把 `local-test/`、真实程序路径、窗口标题、截图或捕获文本写入 tracked source/docs。
