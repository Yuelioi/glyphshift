# 简化词典文件交换

Status: Complete

## Deliverable

让用户从词典管理页导入或导出一个标准 Dictionary `/2` JSON 文件。导出的文件就是可分享、可归档、
可再次导入的发布文件；不建立发布中心、账号、审批、证书管理或远程后台。

## Constraints

- 只接受当前 `glyphshift.dictionary/2` JSON，不迁移旧 schema，不新增 CSV 或自定义压缩包。
- 导入复用 `glyphshift-dictionary-package` 校验；同 ID 已存在时明确拒绝，不静默覆盖。
- 导出复用当前本地工作副本，写出可重新导入的标准 JSON，不夹带安装来源、路径或 Runtime 状态。
- 文件选择继续使用现有 Tauri Dialog；UI 只增加一个导入入口和单行导出动作。
- “发布”在本 Slice 中只表示产生可分发文件，不包含 Catalog 上架、签名服务或网络上传。
- 授权模型不扩展：软件文件由用户显式选择，工作流/探针由用户显式启动，不建立权限数据库。

## Steps

1. 在 Dictionary Package/Desktop Backend 增加有界导入和原子导出合同。（完成）
2. 增加两个 Tauri 命令、稳定错误码及桌面 API 版本。（完成）
3. 在本地词典模式接入导入和逐行导出，保持现有单表布局。（完成）
4. 覆盖合法导入、非法格式、重复 ID、导出后重新导入及中英文 UI 回归。（完成）

## Current

Desktop Backend 已提供有界 `glyphshift.dictionary/2` JSON 文件导入和原子导出：导入复用 Package
校验并派生 unmanaged 状态，非法 JSON、旧 schema 和重复 ID 明确拒绝；导出当前工作副本且可由
Package 重新解析。Desktop API v15 只增加 import/export 两个命令和四个稳定错误码。

本地词典页头只增加一个次要“导入”按钮，每行只增加一个导出图标；文件选择复用现有 Tauri
Dialog。没有新页面、向导、包格式、发布服务或授权存储。全 Rust workspace、36 项 Playwright、
生产构建、Clippy、fmt 和架构检查通过；视觉复核确认桌面与紧凑视口保持原布局。

## Next

该 Slice 已完成。回到主工作页推进生产 Runtime Bundle 与授权实机验收；授权继续复用用户显式
选择程序并启动工作流/探针的现有路径，不增加账号或审批层。
