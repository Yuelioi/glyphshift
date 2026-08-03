# 可分发桌面构建与可见验收

Status: Complete

## Deliverable

生成首个包含 Release Runtime Bundle 的 Windows 桌面安装候选，并证明 Release 桌面无需开发服务器
即可启动、读取内置 Adapter Catalog。随后在用户已授权的 AE 上执行可见翻译与停止恢复验收；安装
候选只构建和检查，不在未获得额外系统变更授权时运行安装器。

## Constraints

- 安装候选、临时 Tauri 配置、Release Runtime、日志和截图全部只进入 `target/local-test/`。
- 跟踪的 Tauri 基础配置不依赖本机绝对路径；构建脚本在本地生成资源映射并传给 Tauri CLI。
- 安装候选内必须包含 Runtime Bundle `/2` 的精确 Release 文件集，不能包含 `test-target.exe`。
- 首个候选只做 Windows NSIS，不增加自动更新、账号、在线发布或证书管理。
- 未配置代码签名证书时明确产出本地 unsigned candidate，不冒充已签名公共发行版。
- 不静默安装或写入系统卸载项；执行安装器需要用户另行授权。
- 可见 AE 验收继续使用用户选择程序与显式 Workflow，不将真实路径、窗口或截图写入 Git。

## Steps

1. 建立 Release 桌面构建脚本，生成本地 Tauri config 并嵌入 Runtime 资源。（完成）
2. 生成 NSIS candidate，检查产物、资源声明与不含测试宿主。（完成）
3. 启动未安装的 Release 桌面进行 Adapter Catalog 与基础 GUI smoke。（完成）
4. 使用授权 AE 完成可见翻译、停止恢复和本地证据。（完成）
5. 完成全量门禁并记录签名/安装剩余风险。（完成）

## Current

`scripts/build-desktop-release.ps1` 已以本地临时 Tauri override 生成包含 Release Runtime Bundle 的
unsigned NSIS candidate，基础配置不含本机路径或发布资源映射。candidate manifest 记录桌面程序、
安装器与 Runtime 清单摘要；安装器资源检查确认只有 Controller、Target Runtime、四个 Adapter 与
Runtime Bundle `/2` 清单，没有测试宿主。未安装 Release 桌面已在隔离数据根下启动通过。

授权 AE 可见验收已证明启用工作流后顶层 `File` 变为 `文件`，停止后恢复为 `File`，目标软件继续
运行。Rust workspace test、Clippy、fmt、架构检查、五项 Runtime 实机合同与 36 项 Playwright
均通过。原始安装候选、截图、日志和真实软件信息只保存在忽略的本地证据目录。

candidate 仍是明确的本地 unsigned 构建；未执行安装器，也未建立代码签名或公共发布流程。

## Next

本 Slice 已完成。后续只有在用户明确授权系统安装变更后才执行 installer smoke；否则提交当前实现
或切换到运行时诊断与字体安全 Work。
