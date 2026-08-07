# 资产创建与清理纠正

Status: In progress

## Trigger

第二次实机审阅暴露三类断点：临时 Probe 已结束后清理被不存在的 Probe Run 阻断；Software 页头把
“新建”与“快速捕获”拆成两个入口；Dictionary 新建只收名称和语言，未复用编辑器已有 metadata。

## Result

- Quick Probe ownership cleanup 先判断 Probe Run 是否仍存在。运行态引用悬空时尽力停止对应 Software
  capture、清除 active 状态，再继续按 ledger 处理 Dictionary 与 Software，不让 `capture.not_found`
  提前中断资产回收。
- Software 页头只保留“新建软件”。Modal 同时提供运行中可见软件下拉与刷新、设备快捷键捕获、浏览
  或手动输入程序路径；三种方式仍进入同一 preflight 与人工确认。
- `DictionaryMetadataForm` 同时用于新建和设置。名称、说明、标签、源语言、目标语言常显；发布版本、
  作者、许可证和主页进入“更多发布信息”。标签和作者使用 `UInputTags`，`SemanticVersionInput`
  用主/次/修订三个非负数字生成版本字符串。

## Verification

用户明确要求先停止检测。本 Slice 尚未运行自动化或 GUI 验证；后续只在用户许可后执行受影响路径
的定向门禁，不运行全仓测试。
