# AI 配置与模型列表

Status: Implemented and packaged

## Scope

API Key 与思考模式同排，安全提示在输入框下方，只保留“请只在你信任的电脑上使用”。模型独占下一行，可直接输入或从获取的列表选择。获取使用当前表单，无需先保存配置；不会改写当前模型。

保留 DeepSeek、通义千问、硅基流动预设，增加 OpenAI、Claude、Gemini、OpenRouter、Groq、Mistral、xAI、Ollama。新增预设不写死默认模型。Codex 订阅继续手动输入模型，没有 HTTP 模型列表按钮。

## Implementation

- 桌面异步 GET 接口按协议处理 OpenAI 兼容 data/id、Claude 分页、Gemini 分页及 generateContent 能力、Ollama 本地模型。模型排序去重。
- 必需 Key 未填时就地提示；区分认证、权限、限流、地址不支持、网络、超时、格式和空列表。错误不返回服务原始内容。
- 服务地址、协议、Key 或弹窗状态改变即清空列表并废弃旧请求结果。请求不跟随重定向；限制总耗时、页数、响应大小和模型数量。
- InputMenu 复用既有输入框样式与高于弹窗的下拉层级。

## Verification

4 项 Rust 合同覆盖实际本地 HTTP 请求、鉴权头、路径、分页、去重、能力过滤与安全报错。2 项 Playwright 测试覆盖手动输入、鼠标选择、Key 提示对齐、认证错误、旧请求废弃和紧凑宽度。截图已检查。另 8 项 AI 配置回归通过，生产构建及 `scripts/review-app.ps1 -UseUserData` 打包启动通过；两份 Runtime Bundle 均通过 13 个适配器校验。

真实供应商 Key 未用于测试；模型可见性和调用权限由供应商账号决定。

## Protocol references

- [Claude Models API](https://platform.claude.com/docs/en/api/models)
- [Gemini Models API](https://ai.google.dev/api/models)
- [Ollama local models](https://docs.ollama.com/api/tags)
- [OpenRouter models](https://openrouter.ai/docs/api/api-reference/models/get-models)
- [Groq API](https://console.groq.com/docs/api-reference)
- [Mistral models](https://docs.mistral.ai/api/endpoint/models)
- [xAI models](https://docs.x.ai/developers/rest-api-reference/inference/models)

## 下拉完整列表修正

用户发现获取数量为 2，重新展开却只见当前模型。原因是 autocomplete 把已选模型名继续用于筛选。新增交互合同先复现失败，再调整为关闭菜单后清除筛选状态；重新展开显示全部模型，原生输入事件才开启筛选。获取成功与服务切换也重置筛选。2 项相关 Playwright 测试通过，补充截图复跑通过。修复版已通过评审脚本重新打包启动，生产构建及两份 Bundle 的 13 个适配器校验通过；窗口响应、哈希一致、标准错误为空。
