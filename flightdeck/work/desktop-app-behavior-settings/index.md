# 桌面应用行为设置

Status: Finished

## Goal

让 Windows 用户能从明确的设置页处理目标软件权限不匹配，并持久配置管理员启动、开机启动与关闭
窗口行为；管理员启动默认关闭，用户开启后持续生效直到手动关闭。

## Current

桌面应用行为设置已经交付并完成自动回归与独立 Release 实机冒烟。持久 UAC 开关、后续普通权限
恢复、当前用户开机启动、任务栏最小化和彻底退出均按合同工作；本机设置与启动项已恢复默认关闭，
测试实例已退出。

## Next

None

## Progress

- 管理员启动是默认关闭的持久开关；当前权限是独立运行时事实，不能由开关状态推断。
- 已确定“最小化”指保留在任务栏；当前没有系统托盘，因此不使用“最小化到托盘”文案。
- `AppSettings` 向后兼容旧的 schema 1 文件，新字段默认关闭开机启动并彻底退出。
- 开机启动写入当前用户 Run 项；设置文件落盘失败时会尽力回滚启动项，避免界面与系统状态分叉。
- Windows Controller 隔离查询权限与 `runas` 启动能力，桌面壳不直接扩散 Win32 `unsafe`。
- IPC 合同证明开启时先保存偏好再请求提升，关闭时只保存且不触发第二次重启。
- 前端构建、桌面壳 43 项测试、Windows Controller 18 项测试和桌面 Playwright 59 项测试通过；另有
  1 项需要真实授权目标的既有测试按设计忽略。
- 深色/浅色、1440×900/960×640 设置页已由仓库 Playwright runner 截图检查；本机证据只保存在
  `target/local-test/`。
- 真实 UAC 冒烟发现旧 `ShellExecuteW` 在父进程立即退出时没有可靠完成异步交接；已改为
  `ShellExecuteExW` 同步创建、保留子进程句柄并确认提升后的 GUI 进程没有在启动阶段退出。
- 提升进程不能依赖继承本地测试环境变量；数据根和 Runtime 根现在通过正确引用的内部参数显式
  传递，避免误读另一套工作区。新版 Release 已确认提升后窗口存在且响应正常。
- 独立 Release 冒烟必须使用 `scripts/build-desktop-release.ps1` 产出的候选；直接 Cargo 编译的 EXE
  保留开发服务器入口。窗口可访问性反馈环能够稳定区分连接拒绝页与内嵌产品界面。
- Tauri `onCloseRequested` 在未阻止关闭时通过 `destroy()` 销毁窗口，因此 capability 必须同时授权
  `allow-close` 与 `allow-destroy`。新增 Playwright 配置合同先红后绿，修复版正式 Release 已由
  UI Automation 连续触发关闭并确认窗口和进程退出。
- 用户接受剩余原生行为冒烟；最终只读检查确认管理员启动与开机启动均关闭、当前用户启动项不存在，
  关闭行为恢复为彻底退出，修复版实例通过真实标题栏按钮完整结束。

## References

- [产品契约](../../../PRODUCT.md)
- [设计系统](../../../DESIGN.md)
- [Nuxt UI 与 Tauri 前端约定](../../knowledge/gui/nuxt-ui-tauri-vite-setup.md)
