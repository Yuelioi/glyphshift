# CoreCLR 可选扩展的接入实验

这是独立的合成验证入口，不是已发布的游戏 Adapter。它验证未来随 Runtime Bundle 直接发布的生命周期行为，不会进入当前 Runtime Bundle 或生产 Catalog，也不会查找、启动或附加真实游戏。

原生实现另有受合成目标门槛保护的 MonoGame profile，其绘制验证由 [MonoGame 测试入口](../monogame-text/README.md) 独立运行。下述 string→string 边界描述的是本脚本的默认 profile。

## 运行

在 Windows x64 上准备 Visual Studio C++ 工具、支持 `--artifacts-path` 的 .NET SDK、.NET 6 运行时、.NET 8 运行时及 Rust，然后运行：

```powershell
./test-support/coreclr-late-attach/test.ps1
```

脚本下载并核验固定版本的官方 CoreCLR 头文件，构建原生组件和两个托管程序。全部新增构建产物、依赖副本和机器证据位于忽略的 `local-test/` 下；该目录不是仓库必需资源。随后定向运行 Native Host 的 ABI 合同，不触及归档包。

Runner 只附加自己创建的合成宿主。宿主先调用原方法完成预热，再加载实验 Adapter 并接受外部 AttachProfiler。没有启动 Hook、SMAPI、Harmony 或托管助手程序集。

## 已实现的接缝

- 在目标模块的内存元数据中新建桥接类型和 P/Invoke 方法，然后通过 ReJIT 给一个已执行过的方法增加参数转换前缀。原方法剩余逻辑保留，磁盘程序集不改写。
- 桥接调用 `NativeRuntimeHostV1.decide_utf16`，可切换词典代次；描述与激活使用真实 `glyphshift_adapter_entry_v1` 布局。Rust 测试用仓库现有 Loader 与 ABI 类型独立验证兼容性。
- 停用获取排他锁，等在途决策结束后撤销回调；后续调用保留原文。只观察不替换，非法决策和不支持的输入保留原文。
- `RequestRevert` 检查逐方法结果，恢复后即使重新注册词典回调也不再拦截该方法。
- 当前进程中的组件继续驻留；新进程没有启动配置或自动加载，检查原文、模块列表和正常退出。该检查证明此实验无后续自动加载，不代表产品扩展安装/卸载流程已经实现。

## 验证边界

目标明确限定为合成宿主的静态 `string Render(string)`，使用 NoInlining 隔离第一阶段的接入问题。虽然使用 RequestReJITWithInliners，**未验证真实既有内联、ReadyToRun、分层升级、任意方法签名或异常处理表的改写**；具有附加 IL section 的方法拒绝处理。只完成字符串层面的可恢复性，不包含画面验证。

新增的内存元数据和 Profiler 都可能留到目标退出。不能称为即时完整卸载；不可对仍被调用的组件使用 FreeLibrary。产品化还需要受信包校验、授权目标控制、激活 ACK、驻留生命周期和扩展移除流程。

词典决策由确定性的宿主回调模拟，生产 Runtime/Kernel 尚未接入。桥接的分配、锁和 UTF-16 marshaling 尚未完成游戏帧时间评估。无 MonoGame DrawString、MeasureString、字形图集、自绘入口发现或真实游戏覆盖声明。

上游接口来自 [.NET 6.0.36 CoreCLR](https://github.com/dotnet/runtime/tree/v6.0.36/src/coreclr)，下载头文件的原始许可证注释保留。实验方案与下一步见 [游戏文字适配器 Work](../../flightdeck/work/game-text-adapters/index.md)。
