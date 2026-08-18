# B6 Profile 推理控制

## Deliverable

每个 AI Profile 显式保存推理策略；翻译默认关闭推理，用户可选择自动或更高强度。Provider 请求真实
携带对应字段，Codex 不再硬编码 low。用最新 DeepSeek V4 记录解释高推理根因并完成真实小批对照。

## Feedback loop

```powershell
cargo test -p glyphshift-ai-translation --test provider_protocol_contract first_release_protocols_use_distinct_wire_shapes_and_decode_structured_results -- --exact
```

当前失败：默认 Responses 请求的 `/reasoning/effort` 为 `None`，期望 `none`。

## Steps

- [x] Profile schema 增加推理策略并迁移旧数据，默认关闭。
- [x] Responses、Chat/Compatible 和 Codex 映射关闭/自动/低/中/高/最高。
- [x] Profile 表单以一个紧凑选择器展示推理策略，并明确 DeepSeek 只支持关闭/high/max。
- [x] Rust wire/profile 合同与 Playwright Profile 回归通过。
- [x] 用真实 ds Profile 运行合成小批，对比 reasoning Token、耗时和最终译文。
- [x] 更新发布文档、隐私扫描并按用户授权提交。

## Current

Profile schema 3 增加 `reasoningEffort`；旧 OpenAI/Compatible/Codex Profile 迁移为 disabled，不支持当前
显式映射的协议迁移为 automatic。Responses 默认发送 none，普通 Chat 默认 reasoning_effort none；
DeepSeek Chat 使用 thinking disabled，high/max 只暴露真实档位；Codex disabled 使用当前模型支持的
none。提示词删除重复 MUST/never 约束，保留索引、数量、占位符和纯 JSON 合同。

## Evidence

- 红色反馈合同原先捕获 `/reasoning/effort = None`，修复后为 `none`；DeepSeek Chat 与 Profile 迁移
  合同通过。
- 旧真实 ds 16 条：input 356、output 12,376、reasoning 12,315、elapsed 110,623ms。
- 新真实 ds 16 条合成 UI：input 227、output 55、reasoning 0、elapsed 967ms，16/16 完成。
- 新真实 Codex 2 条：reasoning 0，结构化译文 2/2 完成。
- Rust 相关三个 package 全部通过；前端 build、AI Playwright 12/12、计时用例重复 2/2；全套运行
  其余 94 条通过，唯一旧计时边界随后已修正并复验。
- Profile 960×640 截图无溢出，机械 detector 无 finding，独立 UI review PASS；最新 Release App
  响应正常、嵌入页面加载且 stderr 为空。

## Next

None.
