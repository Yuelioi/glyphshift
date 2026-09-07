# Alias / Qt Quick 适配器

Status: Open

## Goal

把 Alias 研究中验证的 Qt Quick 文字路线实现为通用原生 Adapter，接入 App 供用户实际验收；同时修正构建目录覆盖和整理本地测试导航。

## Current

新增 `qt-quick` / `qt-quick-native`，包标识为 `windows.qt.quick-text`。实现保留原文、宿主更新优先、动态标签发现、GUI 线程写回与停用恢复，DLL 驻留到目标退出。支持范围限动态 Qt 6.8.3、Windows x64、普通 QQuickText 标签，输入框/富文本不纳入。已接入活动包和 Runtime Bundle 呈现配置。

5 项纯状态合同、1 项真实 Qt 运行库上的合成原生合同及严格 Clippy 通过。原生合同复现并锁住了控制线程退出后钩子失效的问题，现改为进程内 DispatchMessageW detour。真实 Alias 的原生主界面、16 项文件菜单循环和 Tooltip 分项已经通过，停用后原文恢复。此前 Frida 原型证据与新 DLL 分开记录，不把原型结果当作产品全覆盖。

用户已自行修复重装后的 Autodesk Identity Manager 授权错误，当前授权恢复。随后用户明确要求停止逐项实机测试，把适配器放入 App，由用户手动验收。自动实机测试已停止，目标原文已恢复；已通过规定的 review-app 入口同步构建并启动 Debug 桌面 App 与 Runtime，继续读取用户工作区。生产 Loader 对原 Bundle 及打包副本校验 11 个 Adapter 均通过，已确认启动的是本次打包副本且进程保持响应。

开发、审阅、发布与 MonoGame 构建入口已遵循 Cargo 环境/配置，不再强制写入本地缓存；实际构建使用全局 target。本机 README、保留清单和默认只预览的清理脚本已备好，预览确认候选未被占用。执行策略两次拒绝删除，因此缓存尚未实际清理、没有报告释放空间。用户已确认 App 手动测试没有问题，并要求提交本轮源码与构建修正。

## Next

功能已由用户在 App 中手动验收通过；不再自动逐项操作 Alias。剩余仅为本机缓存清理：在执行策略允许或用户手动执行已准备的清理入口后更新状态，不能把提交等同于缓存已删除。

## References

- [工作范围](context.md)
- [原生实现与合同入口](../../../crates/adapters/implementations/framework/qt-quick-native/README.md)
- [实际测试结果](references/live-test-findings.md)
- [官方资料研究](references/primary-source-research.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
