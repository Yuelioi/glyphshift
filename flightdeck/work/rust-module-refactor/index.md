# Rust 模块结构整理

Status: Open

## Goal

按职责和深模块边界整理过大的 Rust 文件，保持现有公开接口与产品行为稳定，让后续真实软件适配和桌面流程修改更容易定位、验证和维护。

## Current

已按深模块边界完成首批四处整理：

- `glyphshift-capture` probe workspace 拆为 model / store / query / export，入口只保留共享状态和公开面；capture lib 回归 30/30。
- 桌面 shell probe 拆为 types / runtime / entries / commands，产品编排仍留在 probe 主模块；desktop shell 回归 117/117。
- `desktop-backend` workflow 将领域模型与磁盘加载/迁移/状态持久化抽出，CRUD、编译解析和校验仍留在主模块；backend 3 个单测 + 20 个合同测试通过。
- `ai-translation` job 将 provider contract、job model 与执行器分离；AI translation 42 项单测/合同测试通过。

拆 probe 时发现前端实际调用的 `desktop_create_probe_from_sources`、`desktop_delete_probe_runs`、`desktop_update_probe_run` 未进入唯一的 Tauri handler，已补齐注册；同时删除从未注册且无生产调用方的三个死 wrapper / retain 方法。

## Next

继续按职责评估 `runtime/targets/runtime` 与桌面 workflow；只有能形成稳定知识边界时才拆。`dictionary/package` 虽然接近千行但当前内聚度较高，暂不为了行数拆分。Qt Painter 仍处于真实软件适配活跃期，等文本链路验证稳定后再整理。
