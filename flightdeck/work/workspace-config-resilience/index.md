# 工作区配置容错

Status: Finished

## Goal

在首次发布前把默认工作区固定为品牌目录，并让所有本地持久配置采用局部容错：未知字段不影响读取，
非法字段只影响自身，单条坏记录不拖垮同文件其他记录，单个坏文件不阻止其余工作区启动。

## Current

默认工作区现为 `<roaming-app-data>/Glyphshift/workspace`，bundle identifier 不再决定用户数据目录。
Settings、AI Profile/历史、软件、词典/安装记录、工作流、探针/观测与快速探针会话都已改为字段、记录、
文件三级容错；新建和保存仍走严格合同。首次发布前的系统凭据、keyring 与 schema 迁移分支已删除。

## Next

None

## Progress

- 用户明确要求取消未发布产品的历史兼容判断，采用一次性开发数据升级。
- 容错原则确定为字段、记录、文件三级隔离；不能安全补默认的身份/引用记录允许跳过，但不得清空其他内容。
- 8 组红灯证明原实现会因局部错误丢失 Settings、AI Profile、AI 历史、词典、软件、工作流、探针或
  快速探针会话；全部在各 store 读取 interface 修复。
- 当前开发工作区已一次性移动到品牌目录，13 个文件的相对路径、大小与 SHA-256 在移动前后完全一致；
  产品代码没有旧目录探测或迁移分支，搬迁后留下的空旧目录已删除。
- 已删除 Credential Manager/keyring seam 和 Profile/History/Settings 的预发布迁移判断；API Key 只有
  Profile 明文 artifact 一个生命周期。
- 回归通过：Settings 6、Quick Probe 4、Capture 20、Dictionary 5、Dictionary Install 10、AI 32、
  Desktop Backend 12、Desktop Shell 81、Settings Playwright 9；前端生产构建、Rust 格式与架构检查通过。
- 最终正式候选 Runtime Bundle 验证 9 个 Adapter，静默安装成功；安装版真实工作区 Playwright 1/1
  通过并显示 2 个 AI 配置；验收端口关闭后已重新以普通模式启动。未运行归档 UIA 测试。

## References

- [稳定上下文](context.md)
- [阶段计划](plan.md)
- [安装版启动修复](../installed-release-startup-exit/index.md)
- [本地 artifact 容错读取](../../knowledge/storage/tolerant-local-artifact-loading.md)
