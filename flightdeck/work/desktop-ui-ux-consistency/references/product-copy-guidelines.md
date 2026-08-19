# 面向普通用户的产品文案规范

## Research Read

问题：Glyphshift 的普通操作界面仍要求用户理解 Profile、Provider、Runtime、Credential、Adapter、
revision 等实现概念。目标表面是设置、AI 翻译、任务、词典、工作流、探针和帮助；用户任务是第一次配置
翻译、看懂状态并在失败时恢复。默认读者熟悉 Windows 基本操作，但不熟悉模型接口或运行时架构。

## Source Matrix

- [Microsoft Windows writing style](https://learn.microsoft.com/en-us/windows/apps/design/style/writing-style)：
  重要信息先说，强调动作，使用熟悉、简短、友好的语言；错误不责怪用户并提供下一步。
- [Microsoft technical terms](https://learn.microsoft.com/en-us/style-guide/word-choice/use-technical-terms-carefully)：
  日常词足够时不用技术词；必须使用的术语在首次出现时就地解释，并在全产品保持一致。
- [Microsoft UI text](https://learn.microsoft.com/en-us/windows/win32/uxguide/text-ui)：删除重复和过度解释；
  用户决定下一步后就会停止阅读，低频细节应进入帮助或渐进披露。
- [Atlassian content design](https://atlassian.design/get-started/content-design)：按钮说明操作结果，表单、
  消息和对话框共享内容规则；全称、语法和大小写应支持可访问性与本地化。
- [GOV.UK error messages](https://design-system.service.gov.uk/components/error-message/)：错误用日常语言
  说明发生了什么和如何修复，不使用内部代码、笼统“无效”或用户无法行动的技术原因。
- [Carbon content](https://carbondesignsystem.com/guidelines/content/overview/)：内容是产品组件的一部分；
  语气可随场景变化，但字段标签、动作和状态必须保持稳定的底层词汇。

## Patterns

1. **先说用户结果**：标题和首句回答“现在能做什么/发生了什么”，实现原因仅在改变处理方式时补充。
2. **一个概念一个词**：同一可复用 AI 连接统一称“AI 配置”；不在 Profile、连接、供应商配置之间切换。
3. **普通界面隐藏实现**：Provider、Runtime Bundle、Credential、revision 和内部 schema 不进入日常页面。
4. **技术名只在决策处保留**：API Key、Token、模型名称是用户需要获取或付费的真实概念；首次出现用
   一句人话解释。兼容方式的底层 Adapter 名仅在“技术与兼容”中渐进展示。
5. **动作写结果**：使用“添加 AI 配置”“删除配置”“重新连接”等动词 + 对象，不使用“确认”“执行”。
6. **帮助文字回答隐含问题**：不重复字段，不写自证式安全承诺；只说明留空、替换、删除、费用等会改变
   用户决定的后果。
   description 只在补充后果、前置条件、作用范围、费用、隐私、不可逆风险或失败恢复时出现；标题、
   字段和选项已经表达的用途不再换句话复述。
7. **错误 = 事实 + 恢复**：先命名失败对象，再给最短可行动恢复；无法确认的内部原因不写成结论。
8. **危险操作说清范围**：删除对话框明确同时删除什么、保留什么；不可逆动作使用对象名作为确认按钮。

## Local Application

| 实现者语言 | 普通界面 | 允许出现的位置 |
| --- | --- | --- |
| AI Profile | AI 配置 / AI connection | 只保留在代码和开发文档 |
| Provider | AI 服务 / AI service | 代码和供应商协议文档 |
| Protocol | 连接方式 / Connection type | AI 配置高级字段 |
| Base URL | 服务地址 / Service URL | AI 配置字段 |
| Model ID | 模型 / Model | 精确值仍由用户填写 |
| Credential | API Key | Windows 凭据管理器只在无法保存密钥的恢复错误中出现 |
| Runtime Bundle | 运行组件 / app components | 版本诊断详情可披露具体构建信息 |
| Adapter | 兼容方式 / compatibility method | 技术与兼容页首次定义“兼容方式（Adapter）” |
| revision | 不显示 | 数据冲突恢复消息可说“内容已变化” |
| Token | Token | 任务用量与费用帮助，保留输入/缓存/输出/思考分类 |

不改变产品模型、模型和接口的真实选项、错误安全边界、Token 实报数据或技术文档链接。代码与 IPC 名称
继续使用既有领域术语，避免为了界面文案进行无价值的内部重命名。

## Next Step

先移除 AI 设置中的自证式说明和开发者状态，再统一 AI 配置及任务术语；随后处理词典 revision、工作流
Runtime/Adapter、探针与错误恢复文案。全部修改完成后一次性运行中英文、宽窄窗口和定向回归。
