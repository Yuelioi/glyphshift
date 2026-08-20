# 稳定上下文

## 确认语义

- 每次从词典或探针发起 AI 翻译都先显示候选规模、AI 配置、模型与请求策略。
- 确认是任务创建前的固定安全边界，不再属于 App Settings，也不存在旧值为 false 时的兼容旁路。
- 取消确认不得创建后台任务或修改译文。

## 数据根语义

- 安装版默认使用当前用户 Roaming AppData 下的 `Glyphshift/workspace` 品牌目录。
- `scripts/review-app.ps1` 是同步构建审阅工具，会通过显式环境变量使用 `local-test/` 下的隔离数据根；
  它不读取或复制安装版 AI Profile，尤其不能把明文 API Key 带入测试证据。
- bundle identifier 只承担安装与 Tauri 内部身份，不决定产品工作区目录。

## 验收

- 设置页不再显示“翻译前询问”。
- 后端/浏览器遗留设置中即使存在 `confirmAiTranslation: false`，词典与探针仍显示确认 Modal。
- 中英文文案、产品契约、领域语言和测试不再把翻译确认描述为可配置偏好。
- Review 构建仍保持本地隔离；安装版默认数据位置合同不变。
- 不运行归档 UIA 测试。
