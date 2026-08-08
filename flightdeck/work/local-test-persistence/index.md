# 持久本地测试根

Status: Finished

## Goal

把机器专属配置、授权样本、可复用运行数据和原始证据从可被构建缓存清理删除的 `target/` 中分离，
统一放入仓库根的 `local-test/`；该目录始终被 Git 忽略，也不能成为产品或测试的必需仓库资源。

## Current

Git ignore、仓库协作约定、产品文档、PowerShell 启动/构建脚本、Playwright 输出和 Rust 本地合同均已
改用 `local-test/`。现有持久数据已迁入新根并校验没有复制失败；旧根只剩可再生成的构建、Bundle 与
合同缓存，以及不含持久文件的空目录。新旧路径不再由 tracked 内容共同支持，不提供兼容回退。

PowerShell 入口语法检查、Runtime wire 隐私合同 4 项和 Playwright 单文件测试列举均通过；机器配置与
Debug Runtime Bundle 也能从新根发现。tracked/staged 路径、Git ignore、旧路径引用和本机绝对路径扫描
通过；未运行全仓测试。

## Next

None.

## Progress

- 明确 `target/` 是可再生成缓存边界，`local-test/` 是被忽略的持久本机工作区。
- 所有 tracked 旧根引用已移除；脚本不提供旧路径兼容。
- 迁移过程保留新根中的持久副本，旧根不再含唯一的机器输入或证据。
- 自审确认新根被 Git 忽略且没有 tracked 文件；机器配置、可复用 Runtime Bundle 与定向路径合同可用。

## References

- [上下文](context.md)
- [计划](plan.md)
