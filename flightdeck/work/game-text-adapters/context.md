# 工作上下文

用户目标是通用游戏翻译，星露谷只是首个真实目标。沿用 Adapter 描述、授权、Runtime Bundle 与词典决策模式；不使用 SMAPI、游戏 Mod、游戏专属资源包作为交付路线，不把游戏专属方法伪装成通用框架能力。

研究与原型结论不等于产品支持。需要区分静态入口、动态命中、可见替换、下一代更新与停用恢复。跨游戏复用需要独立目标验证。

原始本机证据只保存在 local-test/。不恢复 UIA 或依赖它的 OCR，不运行归档包。用户已授权持续修复并启动可测试的桌面应用；统一使用 scripts/review-app.ps1，同步重建 Shell 与 Runtime，默认读取用户配置和数据。

现有 Native Adapter ABI 可提供 UTF-16 文本决策与代次，但 CoreCLR 接入、托管对象寿命、JIT 代码变化和字体布局需另行证明。

首个合成实验已证明可通过新增内存类型及 P/Invoke 桥接现有 Native ABI，无需另行装载托管助手程序集。该路线仍会保留 Profiler 与新增元数据到进程结束；NoInlining 的静态字符串函数结果不能推广为 MonoGame 绘制覆盖。

用户进一步强调无损与卸载恢复，并确认这类游戏适配直接随 Runtime Bundle 发布。行为恢复、进程内组件卸载、下次启动不再加载分别验收；通过 `ProcessResidentAfterDeactivate` 目录/会话标识提示组件可能驻留到目标退出。标识不改变 Native ABI 的 capability feature 映射；无法可靠恢复的实现不交付。

实际接入使用原生 Adapter 自己的激活入口向当前进程的 .NET Diagnostics IPC 请求附加。无需向 TargetRuntimeDeployment 添加 CoreCLR 专属参数。已有 Runtime 提供唯一词典与采集回调；MonoGame 能力按真实框架签名声明，是否采到文字仍须观察批次验证。用户启动必须默认读取本机已有配置，可打开目录必须自带配套 Runtime。
