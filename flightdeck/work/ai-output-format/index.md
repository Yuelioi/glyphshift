# AI 批次输出格式错误

Status: Finished

## Goal

检查 `provider text was not a translation result` 的原因，并修复已确认的程序侧处理缺陷。

## Current

后续稳定性增强已完成：提示词直接描述唯一允许的对象/数组结构，要求把输入视为数据、正确转义且
不输出 Markdown；对完整包裹 JSON 的单个代码块做保守解包；Responses 文本分段按顺序合并，
避免只读取第一段。额外说明、多个结果、截断数据、对象型译文和数量不符仍不接受。

使用增强提示词与每批 20 条通用测试内容进行了 5 批实测，全部 completed、数量正确；其中一批
格式占位符全部保留。样本不足以统计真实工作负载失败率，不宣称已把用户的约两成失败率降至零。
建议 GLM 先尝试 20 条/批、最多重试 2 次；较小批次增加请求次数，本轮未改动用户现有 Profile。

历史记录确认两次 GLM Responses 请求各有一批 50 条失败，其余批次成功；解析错误标记为
不可重试，失败批次均只请求一次。历史未保存原始响应，无法证明具体是 JSON 语法、结构偏差
还是截断。使用同配置的三组 50 条通用测试词条均返回正确数量、completed 状态和合法 JSON；
没有发送真实捕获文本，也未修改用户配置。

已修复：HTTP 内层译文 JSON 解析/结构错误使用现有有界重试预算；零重试配置仍只请求一次，
持续失败到预算即结束，不发布错误结果。新增安全诊断区分空/不完整 JSON、无效 JSON 和结构不符，
仅附行列和字节数，不附响应内容。协议明确报告 token 上限或拒绝时单独分类且不盲目重试。
译文数量和类型严格验证保持不变，没有通过猜测或 JSON 修补接受未知文本。

AI package 37 项测试通过；新增重试回归在旧实现失败，修复后覆盖恢复成功、零重试与预算耗尽。
本机响应和采样脚本只保存在忽略目录。代码尚未重新构建进当前运行的桌面应用；本次仅做本地提交，不推送。

## Next

None。若新版再次出现格式错误，按新增诊断区分具体返回形态；历史两次原始格式仍为未知。

## References

- [上下文](context.md)
- [OpenAI Structured Outputs 说明](https://openai.com/index/introducing-structured-outputs-in-the-api/)
- [OpenAI SDK 对未完成结构输出的处理](https://github.com/openai/openai-node/blob/main/docs/structured-outputs.md)
