# 适配器边界与 GDI 双架构

Status: Open

## Goal

在保持 x64 App、一个逻辑适配器和共享源码的前提下，交付 x86/x64 Runtime 工件与首批 GDI 双架构链路；保留此前字体/适配器审计作为后续独立修复项。

## Current

首批 GDI 双架构链路已实现：x64 App 通过现有 Controller 的双编译连接 x86/x64 目标，一个逻辑适配器按目标选择工件。Bundle schema 4、Registry 架构变体、静态元数据与目标复验、混合进程族路由均已接入。没有新增逐次绘制 IPC。

活动包回归 492 passed、0 failed、21 ignored；最后一次路由顺序调整后 protocol 4 项及双架构原生合同 3 项通过。合成合同覆盖 x64 父进程/x86 子进程同会话的替换、更新、采集、恢复与重新连接，以及 x86 TextOutW、DrawTextW 和字体替换。尚未验收真实 x86 软件。

MonoGame 字形按实际译文动态生成，但默认字体候选、字符及纹理上限写死，尚未接入工作流字体策略。另记录 DirectWrite 的既有 layout/再次启用关联缺口、Qt Quick AutoText 译文门控风险、MonoGame 长会话配额回收和架构识别失败回退问题；区分确定路径与待复现风险。

已通过规定的 review-app 入口同步构建并启动桌面 App，生产加载器验证 11 个逻辑适配器。三项 GDI 增加 x86 工件，其他适配器仍保持原架构边界。构建继续遵循全局 Cargo target 配置。

按用户要求将桌面包、Tauri 配置及锁文件统一升级为 0.3.0，再次通过 review-app 同步构建并启动；运行中的可执行文件 ProductVersion 为 0.3.0，进程正常响应。

用户实测发现快捷探针仍提示 x86 架构不支持。根因是桌面壳 AdapterTargetSupport 对 TargetProcess 保留了仅 x64 的硬编码，底层双架构合同未覆盖该入口。已修复为允许 x86/x64 且继续核对 Bundle 的实际架构清单；预检查失败测试已先复现，新增合成 x86 PE 的预检查、适配器筛选及探针创建回归，桌面壳 85 项测试通过。

门控修复后已再次通过 review-app 同步构建并启动 0.3.0，两次生产加载器验证通过；用户确认原 x86 目标可用并授权提交。该人工结果不扩展为所有 x86 软件兼容。

启动后进程正常响应；Playwright CLI 的界面 smoke 因 WebView 调试连接被拒绝而未完成，不能将进程存活表述为界面已验收。

既有 IDA/R-Saver 的 Qt 修复已由用户验收并提交；本轮双架构实现、0.3.0 版本升级和研究文档获用户授权提交。retour 的最小 i686 ABI 兼容补丁保留在仓库 vendor 下，未修改全局依赖缓存。

## Next

用户另报告求生之路无法注入，下一步从具体产品错误和目标技术入口区分连接失败与文字入口未覆盖；尚未对该游戏作新的注入验证。按 [实施与验收边界](references/dual-architecture-implementation.md)保留 GDI 范围，后续适配器修复参考既有审计。

## References

- [双架构实现与验收边界](references/dual-architecture-implementation.md)
- [研究范围](context.md)
- [字体与适配器审计](references/adapter-audit.md)
- [x86 可行性与阶段](references/x86-feasibility.md)
- [成熟项目对比与方案权衡](references/architecture-comparison.md)
- [Windhawk 与 OBS 源码证据](references/comparison-windhawk-obs.md)
- [Frida 与 Detours 源码证据](references/comparison-frida-detours.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
