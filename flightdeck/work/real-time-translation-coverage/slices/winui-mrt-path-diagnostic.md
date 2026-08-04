# WinUI/MRT 文字路径诊断

Status: Planned

## Outcome

判断 WinUI 资源文字能否落到一个可复用、可安全返回译文的 MRT 原生入口。先证明真实调用路径，再决定
是否建设 Adapter；不因为框架名称、资源可枚举或显式 API 可调用就宣称支持 WinUI。

## Delivery

- [ ] 建立三层确定性宿主：显式 MRT C API、`ResourceLoader.GetString`、声明式 XAML `x:Uid`。
- [ ] 对每层记录入口命中、原文取得、返回值内存所有权、Dictionary 替换和停用恢复；无法证明内存
  所有权时只观察并否决写回。
- [ ] 只有声明式 `x:Uid` 代表性路径也命中同一安全入口，才选择一个授权、小型真实目标做可见 smoke。
- [ ] 真实目标仍须同时满足入口命中、Dictionary 匹配、可见译文和停用恢复，之后才允许进入 Bundle。

## Boundaries

- 不把 WPF 与 WinUI 合并；不在本切片建设 CLR Profiler、ReJIT 或 BAML 修改。
- 不修改目标 PRI 包、资源文件或控件状态；未命中时记录边界，不增加软件名或版本专用扫描。
- 机器路径、进程信息、截图、PRI 样本和原始日志只进入 `target/local-test/`。

## References

- [下一 Adapter 第一方资料评审](../references/next-adapter-primary-source-review.md)
