# 双架构 GDI 实施

1. [x] 架构识别失败拒绝部署、同位数远程地址解析；定向构建 x86 Controller / Runtime / 三项 GDI。
2. [x] 一个 Adapter ID/版本下的架构工件、双架构清单和同位数元数据生成/目标复验。
3. [x] 在现有 Controller 传输接口下路由不同架构目标，保留混合进程族及独立启停语义。
4. [x] 同一 x64 App 的双架构 GDI 合成验收、故障拒绝和 x64 回归。
5. [x] 通过规定入口同步打包并启动 App，交由用户验收指定 x86 软件；真实软件结果待用户测试。

以上为已验收并提交的 GDI 首批。后续用户授权的扩展见下方；仍不运行归档 UIA/OCR。

## 其他活动适配器双架构扩展

用户已验收首批并要求继续补齐其他适配器。范围为默认 Runtime Bundle 的活动适配器；新增游戏引擎文字入口不等同于现有 Adapter 的架构移植。

6. [x] GDI+ / DirectWrite：x86 原生替换、字体与恢复合同，接入 Bundle。
7. [x] GTK 3 / raylib：C ABI 与结构体传值检查，双架构原生合同和工件。
8. [x] Qt Painter：Qt 5 MSVC x86 thiscall、导出及容器布局；Qt Quick 根据受支持 Qt 版本保留明确边界。
9. [x] MonoGame / Unity Mono：Profiler / Mono 调用边界、架构门控、托管测试宿主。
10. [x] App 入口与活动包回归、同步打包启动，记录每项实际通过的范围及仍缺的条件；真实框架软件待用户验收。
