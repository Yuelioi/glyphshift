# MonoGame 原生 Adapter

此组件按现有 Native ABI 加载，公开 ID 为 `windows.monogame.sprite-batch-draw-string`。同一个受校验 DLL 同时承担 Adapter 和 CoreCLR Profiler；没有独立安装的插件或托管启动助手。

Adapter 通过可选 `glyphshift_adapter_source_policy_v1` export 声明 `SpacePaddedSoftWrap`；没有该 export 的 Adapter 保持 Exact。Runtime 和 Capture 共享 Domain 规则：只去除拉丁词之间单个 ASCII 空格后的单个 LF/CRLF，保留该空格；普通换行、空行、缩进、双空格与冒号标题不变。相同形态的手工折行无法仅凭字符串区分。

查找优先使用规范化原文，并兼容旧原文精确键和无译文冲突的旧别名。历史探针只读合并，冲突显示已有译文供选择，不删除旧词典；混合 Adapter 探针按实际观测来源应用策略。译文没有明确换行时，根据原文行宽和替换字体度量重新折行。

若未命中且包含首尾 ASCII 空格、Tab 或换行，再用去边缘空白的键查找，补回原调用的首尾空白后绘制。该行为对应 Capture 目录已去除边缘空白的事实；不覆盖词典里的精确带空白条目。

缺字回退辅助程序集编译为字节并嵌入本 DLL，由目标内存桥接类型加载；没有外置程序集搜索路径或独立插件安装。辅助程序集只引用系统运行库，通过反射使用目标已经加载的 MonoGame 类型。

`scripts/build-monogame-native.ps1` 使用固定版本和哈希的官方 CoreCLR 头文件构建组件；`scripts/build-runtime-bundle.ps1` 将其按内容哈希命名并收入 Bundle。常规构建不含合成宿主门槛；`-Fixture` 显式编译隔离实验 profile。原始构建依赖与产物只存在 local-test/。

激活先检查当前进程的 CoreCLR 和 MonoGame 模块，再连接当前进程的 Diagnostics IPC，请求 CLR 加载本 DLL 的 Profiler。Profiler 按完整 MonoGame 签名选择四个 DrawString 叶子与两个 MeasureString 入口，在目标内存中增加桥接方法并发起 ReJIT。接口直接使用 Runtime 提供的 `decide_utf16`，由真实 Kernel 作词典决策，Capture 作批次采集；没有游戏类名规则。

当前还在 GraphicsDevice.Present 帧边界发布字体回退。缺字时复制原字体纹理、字形与度量，用 Windows TrueType 字体补充缺失的 BMP 字形；Windows 灰度字形转换为 MonoGame 预乘 alpha。原字体与 StringBuilder 保留，只有本次方法参数替换为副本。新字形在 Present 构建并发布，下一帧测量/绘制使用同一份字体；旧图集在所属设备线程的帧边界释放。

回退有每字体 512 个补充字形、32 个字体缓存以及总纹理预算 32 MiB 的上限。超出上限、非绘制线程首次请求、系统字体也缺字或生成失败时保留原文。已发布回退的测量不创建 GPU 资源。当前字体缓存仍可驻留到目标退出，复杂 shaping 与补充平面文字不在此实现范围。

原图集读取区分 Color 和 DXT1/3/5：GetData 返回纹理格式的存储数据，不自动转换成 RGBA。压缩格式先按块大小读取字节并在 CPU 解码，再复制进回退图集；不能对 DXT3 直接申请完整 Color 数组。解码保留颜色与 alpha，并处理不足一个块的边缘区域。当前有 7 个调色/透明度合同，以及包含 24 帧 DXT3 图像的 147 帧真实 Runtime 回归；混排验证同时检查补充中文字形与原有拉丁字形。

激活返回前检查元数据准备和 ReJIT 请求是否成功；方法实际编译可能由后续调用触发，不能把安装回执当作文字命中证明。独立状态查询用于合成验证全部六个入口完成编译。没有适用运行时、框架、签名或可用诊断通道时激活失败。

停用关闭决策入口并排空在途回调，恢复原调用参数；Profiler、已生成元数据与桥接组件留到目标退出。库在 CLR 接受附加时固定到进程寿命，避免失败回滚释放仍被 CLR 引用的代码。常规停用后可再次激活；调试 Revert 后拒绝重新激活。目标重启后才可更换驻留版本。产品目录用 `processResidentAfterDeactivate` 标识此行为，它不参与 Feature 授权协商。

正式 Runtime 停用 Adapter 时必须释放决策状态锁，否则可能与需要该锁的在途回调互相等待。对应调用顺序已在目标 Runtime 中修正。

验证入口：`test-support/monogame-text/test.ps1` 默认使用真实 Runtime/Kernel，覆盖像素、测量、观察批次、激活回执、两代更新和停用恢复；`-NativeOnly` 单独验证 Adapter；`-Fixture` 保留旧的显式附加实验。

已在授权真实 MonoGame 目标上获得生产 Controller 激活回执与非零标准文字采集，测试后停用。缺字回退已通过真实 Runtime/Kernel 的 117 帧合成回归，包括“斧头”新字形、测量、更新和停用恢复；此版本仍需目标重启后进行实机回归。自绘逐字纹理、复杂语言排版和任意框架版本均不在当前能力声明内。

协议依据：[官方 Diagnostics Client](https://github.com/dotnet/diagnostics/blob/main/src/Microsoft.Diagnostics.NETCore.Client/DiagnosticsClient/DiagnosticsClient.cs)、[IPC Header](https://github.com/dotnet/diagnostics/blob/main/src/Microsoft.Diagnostics.NETCore.Client/DiagnosticsIpc/IpcHeader.cs)、[IPC Commands](https://github.com/dotnet/diagnostics/blob/main/src/Microsoft.Diagnostics.NETCore.Client/DiagnosticsIpc/IpcCommands.cs)。
