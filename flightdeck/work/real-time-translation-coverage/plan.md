# 通用实时翻译覆盖计划

## Stage 1：覆盖基线与下一 Adapter

- [x] 建立当前 Adapter 支持矩阵，分别记录观察、字典命中、写回、真实目标和停用恢复证据。
- [x] [选择下一种实时写回 Adapter](slices/next-writeback-adapter.md)，必须绑定一个明确可验证的通用
  文字路径和授权目标。

## Stage 2：端到端原型

- [x] 用确定性合成目标证明 DirectWrite Unicode 观察、Dictionary 替换、失败开放、停用恢复、重复
  激活和复杂布局安全放行。
- [x] 在授权真实目标中证明路径命中和可见译文；零命中时否决该目标，不用更多抽象掩盖结果。
  - 早期 WPF 目标零命中后未进入生产；后续 Scintilla DirectWrite 编辑区取得完整 source、两代可见
    译文和停止恢复，满足生产接入门槛。

## Stage 3：生产接入

- [x] 仅在真实写回验收通过后加入正式 Runtime Bundle、Adapter Catalog 与 Workflow 候选。
- [x] Probe 编辑绑定 Dictionary 后保存并发布下一代预览；GDI+ 确定性目标无需重启即可使用新代次。
- [x] [收敛探针与词典创建流程](slices/creation-flow-distillation.md)：创建时只询问必要信息，取消与
  关闭完整丢弃未提交草稿，发布类元数据留在词典设置中维护。
- [x] 收敛探针管理列表：创建任务已保存时即关闭 Modal；路径与 Adapter 明细按需展示；重启后的
  旧连接统一恢复为“未连接”，列表只显示“未连接 / 已连接”。
- [ ] 在干净重启的授权真实目标上完成
  [Probe 到 GDI+ 实时更新](slices/probe-gdiplus-live-update.md)的用户可见验收。
- [x] 根据实际会话 ACK 区分“可直接替换 / 仅采集 / 无信号”，且仅采集明确说明目标界面不会被修改；
  该状态只属于当前连接，不写入持久任务。
- [x] 同一目标进程内并行候选 Adapter 独立激活；不适用技术只报告自身失败，不撤销健康写回技术，
  Controller 回传真实激活集合供 Session 生成 Active / Failed 状态。
- [x] 将没有目标译文的观察条目明确显示为“词典未命中”。
- [x] 预览发布失败时明确区分“Dictionary 已保存”与“Runtime 未收到更新”，不误报成停止失败。
- [x] 复核 Adapter 最终应用回执并作 No-Go：Qt/GTK 绘制入口没有返回值，GDI API 成功也不证明像素
  可见；运行诊断只显示“替换决策已生成”，不新增第二套持久诊断模型。

## Stage 4：重复扩展

- [x] 根据支持矩阵与第一方资料选择下一文字技术，不按软件品牌累计特例。
- [x] 完成 [Qt Painter 实时写回 Adapter](slices/qt-painter-writeback-adapter.md) 的有界原型：先证明
  每次绘制都能取得原文、Dictionary 命中可替换本次绘制、热更新生效且停用恢复。
- [x] 仅在显式配置的合成宿主合同通过后，选择一个授权 Qt Widgets 目标完成可见 smoke；未命中时
  记录覆盖边界，不退化为仅注入成功。
- [x] 完成 [WinUI/MRT 文字路径诊断](slices/winui-mrt-path-diagnostic.md)：只有声明式 `x:Uid` 也命中
  可安全替换的入口，才进入 Adapter 实现。
- [x] 把共享 Hook、区域 Binding、字体布局和 OCR 等阻塞项送回各自 Roadmap，不抢占覆盖主线。
- [x] 完成 [Tk 文字绘制链对照诊断](slices/tk-text-draw-path-diagnostic.md)：只有公开 Tk 层相对现有
  GDI 在完整词条或中文 fallback 上产生稳定增量，才新增生产 Adapter；结果等价则 No-Go。
- [x] 完成 [Tk 之后的实时文字入口复核](references/post-tk-runtime-seam-review.md)：SDL2_ttf 因缓存
  语义不进入实时 Bundle；SDL3_ttf 与需要目标主动授权的 Web/托管入口转入 Roadmap。

## Stage 5：真实软件缺口驱动

- [x] 完成 [WPF 真实目标缺口验收](slices/wpf-real-target-gap.md)：实际模块确认 WPF；UIA 取得 72 条
  唯一公开文本，但六条 Native 写回入口持续重绘仍全部零命中。
- [x] 将该 WPF 目标明确降级为“仅结构化观察”，不因 DirectWrite/Direct2D 模块存在或成功注入计入
  通用实时写回覆盖。
- [x] WPF 后续选择 UIA 驱动的外部翻译呈现并路由到交互式取词 Roadmap；当前不实现 Managed Agent、
  私有符号 Hook、属性全局改写或万能 Overlay Renderer。
- [x] 按[下一真实目标候选](references/next-real-target-candidates.md)验收 Notepad++ Scintilla DirectWrite
  编辑区：只统计完整编辑区文字，完成首版替换、第二代热更新和释放恢复，零命中则 No-Go。
- [x] 将 DirectWrite TextLayout 加入正式 Runtime Bundle 与 Catalog，并修复目录链接导致的软件路径
  身份不一致，确保通过包管理器稳定入口启动的实例仍可发现。
- [x] 在重新构建的桌面 App 中把 DirectWrite 加入既有 Probe Adapter Plan，确认 UIA 仍可采集原文的
  同时，编辑区由 DirectWrite 产生肉眼可见译文；只有两项证据同时成立才结束端到端验收。
- [x] 复核真实级联菜单：一级与二级菜单均由既有 GDI/USER32 Adapter 捕获，并确认二级词条可生成
  `Matched + Replaced`；不为级联菜单另建 Adapter。
