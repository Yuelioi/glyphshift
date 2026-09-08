# 本地化查询层生命周期模型

独立 C++ 验证模型，按公开的 VGUI「本地化查询 → SetText 深拷贝 → Paint 读取副本」调用方式，固定新路线必须满足的生命周期要求。它不加载真实 VGUI，不使用游戏路径、不安装 Hook，也不声称验证了任何目标 ABI。

运行 `scripts/test-localize-lifecycle.ps1`，默认分别编译并运行 x86/x64。所有输出位于本地测试目录，不接入 Runtime Bundle 或活动适配器目录。

六项检查覆盖：晚启用不刷新旧副本、仅拦截 Find 会漏索引路径、换代需要再次 SetText、查询恢复与控件恢复不同、缺失 token/原始表保持，以及在首次查询前启用能够覆盖新控件。另检查更新/停用后旧查询指针仍存活。

QueryBridge 使用无限增长的保留列表，仅用于短命合成进程。它不是可发布的缓存实现，没有验证容量预算、跨线程调用、返回指针被调用者修改、真实控件发现或刷新机制。后续原生接入必须另有真实 ABI 合同和像素验收，不能把这六项模型检查当成实现完成。

依据与未解条件见 [生命周期调查](../../flightdeck/work/adapter-coverage-and-x86/references/localize-lifecycle.md)。
