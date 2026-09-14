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
- [Probe 到 GDI+ 实时更新](slices/probe-gdiplus-live-update.md)的应用层与确定性 Runtime 合同已交付；
  授权 AE 可见验收作为不再阻塞当前覆盖主线的旧尾项停止，不计为已完成证据。
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

## Stage 6：SDL3_ttf 技术门槛复核

- [x] 完成 [SDL3_ttf 真实目标候选筛选](references/sdl3-ttf-real-target-candidates.md)：外部真实开源
  目标仍缺，生产 Adapter 暂缓；官方 `showfont` 只作为确定性技术宿主。
- [x] 完成 [SDL3_ttf `showfont` 技术 smoke](slices/sdl3-ttf-showfont-technology-smoke.md)：使用动态
  SDL3_ttf 构建，确认 Renderer Text 绘制链、固定 UTF-8 原文以及首代、第二代与停用恢复的可实施性。
- [x] 根据 smoke 结果收口：技术 seam 为 Go，但生产 Adapter 仍为 No-Go；在外部真实目标出现前不新增
  生产 crate、Catalog 项、Runtime Bundle 或软件支持声明。

## Stage 7：下一动态 C 绘制栈

- [x] 完成 [SDL3_ttf 之后的动态 C 文字入口复核](references/post-sdl3-next-dynamic-c-seam-review.md)：
  raylib 真实候选受 ASCII glyph atlas 阻断；Allegro/Open Surge 进入唯一下一实机 Go/No-Go。
- [x] 完成 [Allegro / Open Surge 真实目标 smoke](slices/allegro-opensurge-real-target-smoke.md)：正式包
  为静态/monolith；显式动态组合虽通过 PE、模块与导出检查，但无法形成健康运行目标，按门槛 No-Go。
- [x] 只有动态 ABI、完整 UTF-8 segment 和可见写回同时通过，才进入 Allegro Adapter 实现；本轮运行
  失败都以 No-Go 收口，不新增空置生产选项。

## Stage 8：raylib fallback Font/atlas 原型

- [x] 用 [raylib fallback Font/atlas 原型](slices/raylib-fallback-font-atlas-prototype.md) 明确资源状态机：
  fallback atlas 只在有效 GPU context 的渲染线程创建、替换与销毁，停用立即恢复原字符串和原 Font。
- [x] 在 Musializer 固定源码的 hot-reload Windows 构建中先确认 PE import、`raylib.dll` 实际加载与公开
  文字入口命中，再验证首代中文、同进程第二代新增字形和停用恢复的可见像素。
- [x] 状态机与真实目标四态均通过，判定技术与目标为 Go；默认 atlas 行式打包器被公开 skyline 路径
  替代，缺字、零尺寸或重叠 rectangle 继续 fail-open。

## Stage 9：raylib 生产 Adapter

- [x] 完成 [raylib `DrawTextEx` 生产 Adapter](slices/raylib-draw-text-adapter.md)：实现描述符、Native ABI、
  动态模块/导出门槛、Dictionary 决策与 fallback atlas 生命周期。
- [x] 接入 workspace、架构约束、Native Host 合同、正式 Runtime Bundle 与中文目录；没有 `raylib.dll`
  或兼容 fallback 字体时独立激活失败，不影响同进程其他健康 Adapter。
- [x] 用正式 Bundle 在 Musializer 动态目标复测首代、第二代、停用和关闭清理；通过后才声明支持动态
  raylib 5.5 兼容 `DrawTextEx`，不扩张到静态链接、复制绘制实现或纹理缓存文字。

## Stage 10：raylib 之后的下一 Adapter

- [x] 完成 [raylib 之后的下一写回 Adapter 选择](slices/post-raylib-next-writeback-adapter.md)：只比较有
  外部真实目标、动态公开入口、完整原文与可恢复写回语义的候选，不把 UIA 观察或框架名称计入写回覆盖。
- [x] 只选择一个候选进入有界 Go/No-Go；本轮没有候选越过真实目标与动态 ABI 门槛，已明确收口且未新增
  空置 crate、目录选项或 Bundle 项。

## Stage 11：Adapter 技术文档入口

- [x] 完成 [Adapter 技术文档链接](slices/adapter-technical-documentation-links.md)：在清单、Runtime、
  桌面快照和帮助页贯通可选 `documentationUrl`，旧 Bundle 缺字段时保持兼容。
- [x] 为正式目录 10 个技术条目补齐官方 HTTPS 文档；桌面端只通过受限 Tauri Opener 在系统默认浏览器
  打开，界面不显示裸 URL 或内部制品位置。
- [x] 完成 Bundle、Rust、Clippy、架构、Playwright 和双尺寸视觉回归；缺文档时显示明确空状态。

## Stage 12：Unity / Unreal Engine 引擎感知可实施性

- [x] 完成 [Unity / Unreal Engine 游戏文字 Adapter 可实施性复核](references/unity-unreal-engine-adapter-feasibility.md)：
  区分 Unity Mono / IL2CPP / Localization / 标准与自研 UI，以及 UE localized `FText`、非本地化文字、
  Slate/UMG、Canvas、shaping 与 Shipping 构建边界；不从引擎名推导覆盖率。
- 准入规则：只有每个准备实施的技术分层先取得至少两个无反作弊、未预装插件、可重复的外部 Shipping 真实
  目标，并另有可靠拒绝的负例，才建立 Shipping-like 合成合同与有界原型；逐游戏符号、偏移或机器码
  签名依赖直接触发 No-Go，不新增生产 crate、Catalog 或 Runtime Bundle 项。
  - [x] 匿名公开实现源码已确认 Unity Mono/IL2CPP 运行时元数据与标准 UI setter 路线可作为外部原型
    候选，同时确认初始枚举、对象所有权和停止恢复仍未被现成方案解决。
  - [x] 完成第一轮公开发布候选筛选：保留两份 Windows 候选，并降级两份证据不足的项目。
  - [x] 完成两份正式 Windows 包的只读二进制盘点：均为 x64 Unity Mono，跨 2021.3/2022.3，业务
    程序集覆盖 TMP setter，后一份同时覆盖 uGUI 与 Localization；盘点阶段未启动或注入候选。
  - [x] runtime-only smoke 已验证第一份样本的初始菜单、关卡选择和动态分数，以及第二份样本的无头显
    Viewer 与完整 Monoscopic 工作区；两者均正常退出且未加载 Glyphshift。
  - [x] 为第二份 Mono Standard UI 样本完成可见 Localization 切换；原候选虽有正式 `-language=` 入口，
    但平面模式没有可判读文字，已由 x64 Unity 6 Mono 成品替代。新候选的欢迎页在中文和英语偏好下分别
    显示对应 Localization 文案，连同既有动态 TMP 正例与 NGUI Mesh/atlas 负例通过 Mono 样本 gate。
  - [x] 完成 [Unity Mono Standard UI observe-only 原型](slices/unity-mono-standard-ui-observe-prototype.md)：
    先建立 Shipping-like 合同和只读 Runtime seam，证明初始枚举、setter 观察、去重、停用与 unsupported
    拒绝；本 Slice 不修改目标字符串，不新增生产 Catalog 或 Runtime Bundle。
  - [x] 完成 [Unity Mono Standard UI TextReplace 原型](slices/unity-mono-standard-ui-writeback-prototype.md)：
    复用已验证 runtime seam，证明初始写回、动态属性 setter、第二代更新和停用恢复；两个真实正例可见
    验收前不进入生产 Catalog 或 Runtime Bundle。
  - [x] 完成 [Unity Mono Standard UI 生产接入](slices/unity-mono-standard-ui-productionization.md)：把已通过
    双正例与同后端负例的 Adapter 接入 Catalog/Runtime Bundle、中文技术说明和官方文档入口，再用正式
    Bundle 复核一个正例与负例。
  - [x] 为 Unity IL2CPP standard UI 完成第一轮公开筛选：一份 Unity 2021.3 轻量游戏和一份 Unity 6
    桌面交互应用的发布脚本均明确选择 Windows x64 IL2CPP，工程也实际使用 TMP/uGUI；备用候选不占用
    首轮下载额度。
  - [x] 获得授权后下载并静态盘点两份 IL2CPP 正式包：均为 x64，metadata 29/39 有效，核心
    `il2cpp_*` 入口与 TMP/uGUI 标记存在，未发现符号、开发输出目录或常见反作弊文件名；没有启动候选。
  - [x] 取得单独授权后完成 IL2CPP runtime-only smoke：Unity 2021 LTS 候选两次启动，标准 UI 可见、
    运行模块与静态盘点一致且正常退出；Unity 6 候选虽可见完整 UI，但成品明确是 Development Build，
    首启模态框阻断关闭后需要强制清理，按真实样本 gate 否决。两者均未加载 Glyphshift 或注入模块。
  - [x] 取得单独下载授权后，以备用 Unity 6 release 替换被否决样本：x64 Unity 6000.3.4f1、metadata
    39、核心 IL2CPP 导出与 TMP/uGUI 标记通过静态门禁；连续两次显示完整编辑器 UI、运行模块一致、
    无 Development Build/Glyphshift 模块并正常退出。
  - [x] 完成 [Unity IL2CPP Standard UI observe-only 原型](slices/unity-il2cpp-standard-ui-observe-prototype.md)：
    直接复用两个已固定的跨代正例，证明初始/增量 live-object snapshot、停用与退出；不扩大到 UI
    Toolkit、NGUI、两次 snapshot 间的瞬时文字、写回或生产 Catalog/Bundle。Unity 2021 LTS 动态分数
    正例取得 14 条 Observation，Unity 6 搜索更新正例取得 72 条；均正常停用退出。
  - [ ] 完成 [Unity IL2CPP Standard UI TextReplace 原型](slices/unity-il2cpp-standard-ui-writeback-prototype.md)：
    只通过公开 `il2cpp_class_get_method_from_name`、`il2cpp_runtime_invoke`、UTF-16 string/GC handle 与通用
    Windows 主线程调度调用标准属性 setter，证明初始写回、动态原文、第二代更新和停用恢复；不直接写
    `m_text`/`m_Text` 字段，不读取私有 `MethodInfo` 或逐游戏地址。
  - [ ] 后续恢复 [Unity IL2CPP Standard UI 同后端负例宿主](slices/unity-il2cpp-standard-ui-negative-harness.md)：
    source-only fixture 与静态合同已完成；真实构建/smoke 不再阻塞 observe-only Hook，也不为此安装 Unity。
  - [x] 用户明确授权在外部负例仍缺时先接入生产 Catalog/Runtime Bundle；该决定只改变发布时机，不改变
    证据等级。跨 Unity 2021/Unity 6 的真实 TextReplace 首代、第二代、停用恢复、防回流和兜底字体复跑已完成；
    动态业务 source 的可见写回与同后端负例仍未补齐。
  - [ ] 另行取得一份外部、可重复的 Windows x64 IL2CPP Shipping-like 自绘负例；四次有界公开筛选均为
    No-Go，不用 Mono/混合栈样本或合成宿主冒充该证据。
