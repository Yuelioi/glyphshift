# CoreCLR 运行中接入与恢复实验

## Deliverable

提供不依赖 SMAPI、启动 Hook 或游戏文件改写的确定性接入实验，兼容现有 Native Adapter ABI，区分行为恢复与组件驻留。

## Current

已完成。入口为 [合成验证脚本](../../../../test-support/coreclr-late-attach/test.ps1)，详细边界见 [实验说明](../../../../test-support/coreclr-late-attach/README.md)。

本次通过 34 项托管/原生集成断言，以及 1 项使用仓库权威 Rust Native ABI 和 Loader 的互操作合同。构建无警告。没有附加真实游戏、构建桌面应用或运行归档包。

## Findings

- 运行中 AttachProfiler + ReJIT 可在已预热的合成方法前插入原生决策。无需加载额外托管助手程序集：新增内存桥接类型，P/Invoke 进入 NativeRuntimeHostV1 回调。
- 在已加载的原类型中新增方法曾导致 MissingMethodException；新建桥接类型后通过。此限制是实验得到的本路线实现边界，不能按元数据 API 返回成功推断 JIT 可解析。
- 首代、下一代、只观察、词典未命中、空值、超长及嵌入空字符输入、非法输出、GC 和正在执行的回调排空均有断言。
- 停用先排空回调并恢复原文；Revert 检查逐方法结果。恢复后重新注册词典回调仍保持原文，证明已移除方法拦截而不只是暂时透传。
- 目标磁盘程序集哈希未变；当前进程仍有扩展 DLL，新进程无该 DLL。没有实现产品安装/卸载流程，不能将“新进程无加载”冒充完整安装移除验收。

## Limits

只验证静态 string→string、NoInlining 的合成方法，原 IL 保留；附加 section/EH 不支持。未证明现有内联、ReadyToRun、任意签名、MonoGame 绘制、中文图集、上下文、帧时间或真实游戏覆盖。词典回调是确定性宿主实现，生产 Kernel 尚未接线。

实验代码位于 test-support，只有专用脚本才运行，不进入生产 Catalog。原始日志、下载头文件和构建产物只保存在 local-test/。共享 ABI 和生产卸载流程没有修改。

## Next

本 Slice 已完成；按 Work Plan 进入 MonoGame 标准文字合成实验，验证实例方法/string/StringBuilder 等真实签名及画面恢复，先不向真实游戏附加。
