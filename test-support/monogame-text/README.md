# MonoGame 标准文字绘制实验

当前默认合同已扩展为 117 帧：在仅有拉丁字形的原字体上，用真实 Runtime 发布“中文”“中”和新增的“斧头”，比较独立参考字体的像素/度量，并验证停用后原字体和原文字恢复。下面 75 帧/74 帧数据是初次实验记录，当前实现以 [字体回退 Slice](../../flightdeck/work/game-text-adapters/slices/monogame-font-fallback.md) 为准。

使用真实 MonoGame DesktopGL 3.8.1.303 和 .NET 6 创建确定性窗口，验证 CoreCLR 可选扩展路线的文字与像素恢复。不是已发布的 Adapter，不向真实游戏附加，不进入生产 Runtime Bundle。

后续更新：原生组件现已随 Runtime Bundle 发布。本脚本默认验证真实 Runtime/Kernel 驱动的组件（74 帧、捕获批次与 ACK）；`-NativeOnly` 验证原生 Adapter，`-Fixture` 才运行旧的独立附加实验。下文“未接入生产”的描述是初次实验范围，当前实现与生命周期以 [原生组件说明](../../crates/adapters/implementations/framework/monogame-native/README.md) 为准。

## 运行

安装上一阶段要求的 Windows x64 C++/.NET/Rust 工具，并准备仓库桌面项目的 Playwright 依赖，然后执行：

```powershell
./test-support/monogame-text/test.ps1
```

脚本构建同一 checkout 的原生实验组件与合成窗口，使用仓库的 Playwright CLI 驱动窗口进程的测试协议；不启动浏览器。截图、GPU 图像数据、日志、依赖和构建产物只写入 `local-test/`。本机路径由脚本解析或环境变量提供，没有游戏路径配置。

Runner 只创建和附加本次合成窗口；Profiler 还要求进程中存在该合成宿主模块，避免把实验入口当作真实目标连接器。

## 已验证行为

- 接入前通过真实绘制 API 生成 `AAA`、`中文`、`中`、`B` 的参考图像与测量值。
- 定向 ReJIT MonoGame 的四个 DrawString 叶子重载（string/StringBuilder，各有基本与 Vector2 缩放版本）和两个公开 MeasureString 重载。
- 标量缩放重载通过原框架转发到叶子；一帧只发生一次测量与两次阴影/前景绘制的决策，不重复翻译包装调用。
- 词典把 `AAA` 依次翻译为 `中文`、`中`。六种调用方式的完整 GPU 像素、测量宽高都与接入前参考一致；裁剪、阴影、GC、只观察与词典未命中行为有断言。
- 字体缺失译文字形时保留原文。原字体与 StringBuilder 不修改；译文 Builder 使用新对象。图集由确定性字形矩阵生成，不依赖机器字体。
- 停用后原文图像恢复且不再采集；再次激活恢复当代译文；RequestRevert 后即使回调重新激活也保持原文。
- 渲染表面与窗口后台缓冲区逐帧哈希一致，命令回执在后续 Update 发出。该证据不包括桌面遮挡、显示器合成或真实游戏窗口。
- 宿主和 MonoGame 磁盘程序集的哈希未变；目标正常退出。

本次完整测试通过，采样 75 张图像，后台缓冲区差异 0，Builder 修改 0。前一阶段 34 项断言和 1 项 Rust Native ABI 互操作测试也已回归通过。

## 实现边界

原生桥接仍沿用现有 Native Adapter ABI。MonoGame 方法按框架类型与完整签名筛选，不使用游戏类名；合成宿主名字仅作为禁止误附加真实目标的实验门槛。

当前字体已有中文字形时可替换；**不包含动态补字形或字体替换**。没有证明复杂布局、跨帧排版缓存、并发代次切换的一致性、Unicode shaping、RTL、ReadyToRun、任意既有内联及游戏帧时间。测试包含正常框架包装路径，但不能从此推导所有内联代码都已覆盖。

ReJIT 字体桥接类型直接在目标模块内存中生成；支持的六个方法必须全部完成编译才报告就绪。原 IL 的附加 section/EH 仍拒绝改写。新增内存元数据和 Profiler 驻留到目标退出，不声称即时完整卸载。

生产 Kernel、扩展安装/移除、可信包与目标授权控制面尚未接线。星露谷绕过 DrawString 的自绘 SpriteText 对话仍是独立问题。

接口依据：[SpriteFont API](https://docs.monogame.net/api/Microsoft.Xna.Framework.Graphics.SpriteFont.html)、[固定版本 SpriteBatch 源码](https://github.com/MonoGame/MonoGame/blob/v3.8.1/MonoGame.Framework/Graphics/SpriteBatch.cs)。工作进度见 [Flightdeck](../../flightdeck/work/game-text-adapters/index.md)。
