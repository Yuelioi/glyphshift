# WinUI/MRT 文字路径诊断

Status: Complete

## Outcome

判断 WinUI 资源文字能否落到一个可复用、可安全返回译文的 MRT 原生入口。先证明真实调用路径，再决定
是否建设 Adapter；不因为框架名称、资源可枚举或显式 API 可调用就宣称支持 WinUI。

## Delivery

- [x] 建立三层确定性宿主：显式 MRT C API、`ResourceLoader.GetString`、声明式 XAML `x:Uid`。
- [x] 对每层记录入口命中、原文取得、返回值内存所有权、Dictionary 替换和停用恢复；无法证明内存
  所有权时只观察并否决写回。
- [x] 已判断声明式 `x:Uid` 不命中显式字符串的同一入口，因此按既定闸门停止在合成原型，不进入真实
  目标 smoke。
- [x] 未进入 Runtime Bundle，也未把加载时替换计入实时翻译覆盖。

## Result

- 微软第一方源码与合成宿主实测闭合了两条不同调用链：`ResourceLoader.GetString` 和显式字符串读取
  命中 `MrmLoadStringResource`，默认 WinUI 3 `x:Uid` 属性包命中
  `MrmLoadStringOrEmbeddedResourceByIndex`。
- 两个入口均可在精确字典命中时使用 `MrmAllocateBuffer` / `MrmFreeResource` 对称替换；声明式入口还须
  同时满足字符串类型和 `Text` / `Content` 等文本属性白名单。合成窗口中的三类原文均得到可见译文。
- 停用 Hook 后重新创建窗口，三类资源均恢复原文；已经加载进控件的值不会被通用 MRT API 主动刷新或
  回滚。因此这条能力只能定义为“WinUI 3 / MRT Core 加载时字典替换”，不符合当前 Work 对立即更新与
  停用恢复的完整实时翻译承诺。
- 原始命中日志、宿主二进制与本机运行证据只保留在本地测试目录；仓库仅记录可移植结论。

## Decision

本切片对“生产级实时翻译 Adapter”作 **No-Go**，不继续真实目标与 Bundle 集成。若后续产品明确接受
“新页面或重新加载后生效、旧界面不自动恢复”的独立能力，可在 Roadmap 中以加载时资源 Adapter 重新立项，
并补齐属性包来源追踪、版本闸门和真实目标验收。

## Boundaries

- 不把 WPF 与 WinUI 合并；不在本切片建设 CLR Profiler、ReJIT 或 BAML 修改。
- 不修改目标 PRI 包、资源文件或控件状态；未命中时记录边界，不增加软件名或版本专用扫描。
- 机器路径、进程信息、截图、PRI 样本和原始日志只进入 `target/local-test/`。

## References

- [下一 Adapter 第一方资料评审](../references/next-adapter-primary-source-review.md)
- [WinUI 3 / MRT Core 原生入口诊断](../references/winui-mrt-primary-source-diagnostic.md)
