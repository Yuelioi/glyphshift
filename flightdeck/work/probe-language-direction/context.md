# 稳定上下文

## 产品事实

- Glyphshift 支持任意源语言到任意目标语言；应用界面语言与词典语言方向相互独立。
- Dictionary Metadata 拥有 `sourceLocale` 与 `targetLocale`。Probe Run 只绑定 Dictionary，不另存第二份
  语言方向。
- 复用已有 Dictionary 时沿用其语言元数据；只有创建临时 Dictionary 时需要在 Probe 创建器中填写。
- 当前创建流程没有可靠的源语言自动识别能力，不能把固定字符串 `auto` 表述为已经完成检测。

## 决策

- 临时 Dictionary 的源语言和目标语言都属于必填常显字段，使用与 Dictionary 表单一致的 locale ID
  输入语义。
- 新建和取消后重置默认值为 `en-US → zh-CN`；用户可以改成任意合法 locale ID。
- 创建请求必须显式携带 `sourceLocale` 与 `targetLocale`，后端校验并原样写入临时 Dictionary。
- 中英文文案只说明两项语言的作用，不声称自动识别。

## 验收

- 中英文与紧凑视口均能看到两个语言字段，标签、顺序和必填语义清楚。
- 修改两项语言后创建命令收到相同值；取消再打开恢复默认值。
- 后端合同证明临时 Dictionary 持久化两项显式语言，非法任一项均返回可恢复错误。
- 只运行 Probe/Desktop 定向门禁；归档 UIA 不属于本 Work。
