# Probe 与 Dictionary 联动

Status: In progress

## Trigger

用户创建 Probe 时选择已有 Dictionary，保留临时会话后发现设置中没有词典配置，详情也没有把采集
结果加入、同步或移出词典的明确操作。底层虽在行内译文失焦时写入 `dictionaryId` 对应资产，但产品
表面没有表达这一事实，也不能修改绑定。

## Result

- Probe 详情在联合表上方常显绑定 Dictionary 的名称、语言与词条数，说明 Observation 与 Dictionary
  的边界，并可直接打开词典编辑器。
- Probe 设置纳入唯一 `dictionaryId`；运行中保持锁定，释放连接后可切换。切换只改变引用，不复制或
  删除新旧 Dictionary 内容。
- 联合表的状态改为“未进词典 / 已在词典 / 词典已有”，翻译列明确命名为“词典译文”。单条保存和
  多选“保存所选到词典”只接受非空译文；“从词典移除”删除词条但保留 Observation。
- Desktop API `/26` 增加批量词条同步 command；Desktop Backend 在一次 revision 更新中原子 upsert
  多条翻译，之后统一 reconcile Workflow 并刷新 Probe preview。

## Review

- 没有允许纯原文生成空译文或原文自拷贝的伪词条。
- 批量保存与既有 500ms 行内保存共享串行保存队列，避免失焦与批量点击并发造成 revision conflict。
- 切换绑定受运行状态保护；旧词典内容、Observation 与临时资产 ownership 不被隐式改写。

## Verification

用户明确要求先停止检测。本 Slice 尚未运行自动化或 GUI 验证；后续只在用户许可后执行 Core、Desktop
Shell 与 Probe 页面定向门禁，不运行全仓测试。
