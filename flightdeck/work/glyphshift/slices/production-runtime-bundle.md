# 生产 Runtime Bundle

Status: Complete

## Deliverable

把当前只由 `dev-app.ps1` 临时拼装的 Debug Runtime 提取为一个可复用的 Debug/Release Bundle
构建流程。Release Bundle 必须只包含桌面运行所需的 Controller、Target Runtime、四个 Native
Adapter 与强校验清单，并能由现有 `RuntimeBundle` 真实打开及驱动授权软件；合成测试宿主不得进入
Release Bundle。

## Constraints

- 所有本机 Bundle、日志与验收证据只写入 `target/local-test/`，不跟踪二进制或真实软件信息。
- Bundle 清单只接受产品内置的第一方 authority；不能由清单自报一个任意 signer 后自行获得信任。
- Debug 开发启动与 Release 构建必须复用同一个组装实现，不维护两份 artifact 列表或 manifest 逻辑；
  Debug 允许保留被已运行目标锁定的旧 hash artifact，Release staging 必须是精确清单且无陈旧文件。
- 保持现有显式授权：用户选择程序并启动 Workflow/Probe；不增加账号、审批或权限数据库。
- 合成测试可携带 `test-target.exe`，Release Bundle 不携带测试宿主。
- 本 Slice 不建立安装器、自动更新、在线签名服务或证书管理；首个可分发桌面构建由下一 Slice 接管。
- 实机失败保持 fail-open，不终止、不重启用户目标软件；原始日志和截图只保留在本地证据目录。

## Steps

1. 建立 Debug/Release 共用的 Runtime Bundle 构建脚本和清洁 staging。（完成）
2. 固定第一方 Bundle authority，并覆盖未知 authority、artifact hash 和路径拒绝合同。（完成）
3. 让 `dev-app.ps1` 复用构建脚本，保持现有合成 Runtime 合同。（完成）
4. 构建 Release Bundle，证明不含测试宿主且可由 `RuntimeBundle` 打开。（完成）
5. 对已授权运行中的 AE 执行激活、保持与恢复验收，记录便携结论。（完成）

## Current

`scripts/build-runtime-bundle.ps1` 现在是 Debug/Release 的唯一 artifact 列表与 manifest 构建入口。
它只允许输出到本地测试根，先在 staging 生成并验证精确文件集合与所有 SHA-256；Debug 可显式
携带合成宿主并保留被目标进程锁定的旧 hash artifact，Release 恰好包含清单和六个生产 artifact。
同一代输出按内容无操作复用，不同代输出在修改旧目录前拒绝，避免已加载 DLL 导致半更新。

Runtime Bundle 已直接升级到 `/2`，manifest 使用固定
`app.glyphshift.runtime.first-party` authority；未知 authority、越界路径和不匹配 runtime bytes 均在
Native Adapter 加载前拒绝。`dev-app.ps1` 已复用新构建器，五项真实 DLL/合成进程合同通过，新桌面
正在使用 `/2` Bundle 运行。

Release Bundle 已验证为 7 个文件、4 个 Adapter、无 `test-target.exe` 且所有 hash 一致；同一
Bundle 对已授权且运行中的 AE 完成发现、TextReplace 激活和 pass-through 恢复，未终止或重启目标。
全 Rust workspace、Clippy、fmt 与架构检查通过。原始 Bundle、日志和实机信息只在本地忽略目录。

## Next

该 Slice 已完成；后续的可分发桌面构建与人眼可见验收也已闭合。当前执行选择以 Work 页为准，
安装器执行仍需单独的系统变更授权。
