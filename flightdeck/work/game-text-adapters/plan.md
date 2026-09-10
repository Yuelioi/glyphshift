# 实施顺序

## 用户实测反馈与后续候选

- [x] [统一工作流运行生命周期](slices/workflow-lifecycle.md)：执行已接受的状态模型，统一三处展示与操作、收集设置、AI 取消及回归。

- [x] [采集合并与筛选性能](references/collection-performance-review.md)：首轮完成摘要/追加分离、版本快照与合并排序缓存、AI 单快照、请求合并和工作流隐藏筛选；30 项采集、99 项桌面、1 项分类缓存及 13 项前端流程通过。完整 JSON 原子保存保留，进一步存储迁移以实机耗时决定。

- [x] [字体与字号归工作流](slices/workflow-typography.md)：默认字体/字号与每字典覆盖，字典包不再参与显示策略；常用字体、创建字典语言下拉与关于链接同步完成。

- [x] 设置页内分通用、软件、字体、语言：共享最近记录的删除/清空、常用字体搜索收藏、翻译语言增删及恢复内置列表；字典新建/编辑/导入复用语言列表，已有配置不因候选删除而改变。

- [x] 设置页增加按译文语言保存的缺字备用字体：20 个常用语言选项、自定义语言代码、本机字体搜索、修改与移除；配置持久化完成，运行时消费仍属于下项字体回退接入。
- [ ] [Unity Mono 中文缺字修复（优先）](slices/unity-font-fallback.md)：已完成 TMP 原字体方框复现、两代中文实际字图与原文/字体/材质/像素恢复；尚待正式 Adapter 接入、资源寿命和 uGUI 独立验证。
- [ ] WOLF RPG Editor 通用适配器可行性验证：以 Mad Father 的 2.21 运行器为首个授权样本，确认完整文字入口、编码、字体、两代替换与停用恢复；再找独立目标验证复用。2.x 与 3.x 分开验收，尚未承诺支持。
- [ ] RPG Maker VX / RGSS2：以 The Witch's House 1.09a 验证完整文字入口、中文字形与恢复，不沿用 MV 的 JavaScript 入口。
- [ ] RPG Maker 2000/2003：以 END ROLL、OFF、Yume Nikki 验证运行器版本、编码与跨游戏文字替换；区分原作制作版本和实际运行器。
- [ ] KiriKiri Z：以 ATRI 验证正文、选择项、控制符、字体与停用恢复；只确认版本资源不算支持。
- [ ] CatSystem2：以 NEKOPARA Vol. 3 验证运行时文字入口、编码、字体与恢复。

候选按上述顺序进入可行性验证；MV 已完成的原型继续按下节推进，不因新增引擎重复开发。实测依据见 [游戏目录筛选](references/aooni-mad-father-check.md#扩展游戏目录静态筛选)。

## MV 当前实现

- [x] 审计已有文字、决策、刷新和生命周期接口，写清职责与能力边界。
- [x] 建立 MV 完整对白合成合同原型，验证同步查找、失败透传与停用后的下次显示。
- [x] [开发运行器真实引擎验证](references/mv-runtime-entry-validation.md)：完整中文对白、另一份译文、停用后新对白恢复；普通运行器无调试端口，正式接入仍未完成。
- [x] 普通发布运行器的窄版本原生入口：x86 / V8 6.5.254.31，受控项目、无调试端口、无文件修改，同一画面合同通过。
- [x] 原生会话与无 Frida 验证：DLL 管理入口 Hook、注册与队列；独立 x86 helper 承担测试装载，实际画面、12 轮 Runtime 启停、4 轮 Hook 会话及过期请求合同通过。
- [x] Runtime 激活拥有固定引擎脚本安装：脚本嵌入 DLL，启动自动接入、停止撤销自己的入口；4 轮实际字图与原函数恢复通过，接入失败回滚与重试纳入合同。
- [ ] 生产化版本与会话协商、目标线程调度；Frida 原型不随产品发布。
- [x] 将原型连接现有 Runtime 采集和 publication：测试桥通过正式 Native Loader、Text Host、Kernel 与 Capture，完整对白、两代更新、停用后新对白恢复通过；不是产品适配器。
- [ ] 实现当前对白更新与恢复，验证选择、控制符、变量、分页与字体。
- [ ] 用两个独立授权目标通过可见替换、下一代更新和停用恢复验收，再进入产品目录。

架构依据见 [引擎接入方案](references/engine-adapter-architecture.md)。当前执行位置由 Work 的 Next 指定。

## 已有 MonoGame 路线


- [x] [CoreCLR 运行中接入与恢复实验](slices/coreclr-late-attach.md)：已执行方法 → Native ABI 决策 → 两代替换 → 停用排空 → Revert → 新进程无扩展。
- [x] [MonoGame 标准文字实验](slices/monogame-standard-text.md)：固定版本真实 DrawString/MeasureString 的六个入口、已有字形、像素、两代更新与恢复；包装转发通过，任意内联和 ReadyToRun 不在本次结论内。
- [x] [通用 MonoGame Adapter 策略](slices/monogame-adapter-policy.md)：将 UTF-16 边界、字体字形覆盖、fail-open 与词典决策收进框架级 crate，不依赖游戏名称。
- [x] Runtime Bundle 直接接入：原生 Adapter 自行附加当前 CoreCLR，生产 Kernel/Capture 与 ACK 验证通过；目录标识与探针提示完成，停用后驻留到目标退出。
- [x] 生产完整性：Bundle 校验 10 个 Adapter，实际包含 MonoGame DLL；用户数据模式直接启动配套 exe/runtime。
- [x] [缺字字体回退](slices/monogame-font-fallback.md)：复制原图集补字形、帧边界发布、117 帧真实 Runtime 合成回归通过；待目标重启后实机复核。
- [x] [压缩字体图集修复](slices/monogame-compressed-atlas.md)：实机 DXT3 读取异常定位，7 个解码合同和 147 帧回归通过；待加载新 DLL 后实机可见复核。
- [x] [排版空白匹配](slices/monogame-edge-padding.md)：实机说明末尾空格 no_match 定位，精确优先与保留空白的回退匹配回归通过；待目标重启复核。
- [ ] 自绘文字入口发现实验：两个独立合成渲染器与非文字负例，证明发现边界后才允许写回。
- [x] [软折行去重](slices/monogame-soft-wrap.md)：来源声明、统一键、历史只读合并、冲突选择、译文重排与混合 Adapter 边界通过定向回归；待新版目标复核。
- [ ] 扩大真实覆盖：首个授权目标已取得生产 Active 与非零标准文字采集；仍需独立 MonoGame 游戏与更多场景，不将标准文字支持冒充完整对话支持。

## 尚需明确的能力

复杂布局与跨帧测量缓存、并发词典代次、ReadyToRun/任意内联和帧时间需要在扩大能力前单独验证，不随标准文字实验自动晋级。当前补字形只声明有预算上限的 Windows BMP 字形回退。
