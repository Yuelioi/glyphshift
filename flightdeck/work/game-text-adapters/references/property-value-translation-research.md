# 属性与动态数值的翻译匹配调查

日期：2026-09-11。范围：当前源码只读调查与设计建议；未修改产品、运行目标软件或执行归档 UIA。

## 结论

`Hidden:0`、`Selected:0`、`Total:33` 应作为「固定属性 + 保留原样的动态值」处理。当前整句精确匹配不能表达这个需求：保存 `Hidden:0 → 隐藏:0` 只覆盖这一串，数值变成 `1` 就不再匹配；只保存 `Hidden → 隐藏` 也不能命中完整的 `Hidden:0`。

建议新增显式的「保留数值」词条模式，让用户从采集行创建规则，编辑属性译名；运行时捕获数值并原样拼回。第一版限定单个冒号后的无符号十进制整数，足以覆盖给定示例。不能把以下建议语法当作当前已支持的功能。

| 规则的概念表示 | 运行时原文 | 结果 |
| --- | --- | --- |
| `Hidden:{value} → 隐藏:{value}` | `Hidden:0` / `Hidden:12` | `隐藏:0` / `隐藏:12` |
| `Selected:{value} → 已选:{value}` | `Selected:0` / `Selected:5` | `已选:0` / `已选:5` |
| `Total:{value} → 总计:{value}` | `Total:33` / `Total:104` | `总计:33` / `总计:104` |

## 源码证据

- [TranslationSnapshot](../../../../crates/core/translation/src/snapshot.rs#L12) 用 `location → source → translation` 的 `BTreeMap` 存放译文；[lookup](../../../../crates/core/translation/src/snapshot.rs#L230) 调用 `entries.get(source)`，没有子串、模板或正则分支。
- [lookup_for_adapter](../../../../crates/core/translation/src/snapshot.rs#L238) 在精确查找前执行适配器范围检查；[lookup_context](../../../../crates/core/translation/src/snapshot.rs#L252) 把 location、context kind/key、完整 source 组成键。动态规则必须继承这些作用域，不能旁路检查。
- [DecisionEngine::decide_at](../../../../crates/core/decision/src/lib.rs#L384) 直接消费上述查找；[decision_from_parts](../../../../crates/core/decision/src/lib.rs#L428) 产生完整的 `TextDecision::Replace`。目前没有「仅替换某段并保留其坐标」的决策表示。保留数值内容与保留数值像素位置是两项不同能力。
- [DictionaryEntryCreate](../../../../crates/dictionary/package/src/lib.rs#L83) 与 [DictionaryEntryArtifact](../../../../crates/dictionary/package/src/lib.rs#L488) 只有 source/translation，没有匹配模式或捕获字段。仅在界面显示 `{value}` 不能让运行时生效。
- [TranslationWorkspace::rebuild_snapshot](../../../../crates/core/translation/src/workspace.rs#L736) 先解析来源层优先级，再生成精确快照。规则种类需要进入词条身份和冲突处理，否则同文精确项和动态项会相互覆盖。
- [SourceTextPolicy](../../../../crates/core/domain/src/source_text.rs#L5) 只有 Exact 与由文本生产者声明的 SpacePaddedSoftWrap；此能力归一化软换行，不是数值模板。不能在这里全局删除数字来实现动态匹配，否则会破坏版本号、型号及正文含义。
- [Runtime source_aliases](../../../../crates/runtime/targets/runtime/src/lib.rs#L732) 构建软换行别名，遇到不同译文歧义会放弃别名；这条现有兼容路径也不支持属性模板。

## 建议的最小实现

1. 字典新增明确的模式，例如 `exact` 与 `propertyNumber`；旧词条默认为 exact。普通文本里的 `{value}` 仍按普通文本处理，只有显式规则才激活。模式与字段贯穿导入导出、持久化、后端视图、工作区与 Runtime 快照；需要明确旧版本导入语义，不能静默丢失规则字段。
2. 规则保存固定属性原文及译名，数值视为运行时捕获片段。示例的捕获只接受一段 `[0-9]+`，原样保留 `0033` 之类格式；冒号与紧邻空白也保留原样。全串匹配，禁止隐式匹配 `NotHidden:0`、`Hidden:abc` 或正文中的 `Hidden:0`。
3. 在现有位置和上下文路由语义内先尝试完整精确项，再尝试显式动态项。沿用字典来源优先级、适配器范围与冲突诊断；不要在各 Adapter 内各写一个数字正则。若以后希望精确项跨路由优先，必须另行明确该语义，不能顺手改变现有路由顺序。
4. 将规则在快照发布时编译为有界结构，调用时只分解一次尾部数值并按固定属性键查找；避免逐帧扫描所有正则。返回重新组合的完整译文，仍走现有 Replace 通道。规则影响快照 digest，热更新能替换或撤销规则。
5. 采集列表保留真实原文及命中预览；可将 `Hidden:0` 与 `Hidden:1` 显示为同规则覆盖，但不能丢失原始观测。AI 只处理固定属性，数值不送去反复翻译。更一般的负数、小数、百分比、时间及多个占位符留到明确需求后扩展。

此方案解决「只汉化属性、数值持续变化」。如果目标还要求数字列在汉化前后保持同一横坐标，需要目标绘制适配器的测量/对齐语义支持；不能靠模板本身保证，也不建议靠补空格凑齐。

## 实现时的验收建议

- 合成核心用例：三个示例；数值从 0 到多位数变化；前导零原样保留；冒号与空白保留；负数、单位、空值、非数字和额外正文不匹配。
- 回归：精确词条仍精确；同位置精确项覆盖动态项；普通字面 `{value}` 不触发规则；来源优先级、上下文和适配器范围隔离；规则歧义明确处理。
- 包与发布：旧包默认 exact；规则保存、重载、导出导入不丢失；规则变化使 digest 变化；运行中发布下一代词条和撤销正确生效。
- 前端使用仓库 Playwright 验证从采集行创建规则、译名编辑、多个数值预览及重新加载。只运行显式 active-package 验证入口，不使用 workspace-wide Cargo 命令。

本轮仅完成源码调查，以上测试尚未实现或执行；没有声称目标软件已经支持动态匹配。
