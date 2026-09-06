# MonoGame 标准文字与画面恢复

## Deliverable

把已验证的 CoreCLR 接缝用于真实 MonoGame 标准绘制/测量入口，以像素与原对象保护断言证明两代替换和恢复。

## Current

已完成窄范围合成验证。入口为 [MonoGame 测试脚本](../../../../test-support/monogame-text/test.ps1)，完整边界见 [实验说明](../../../../test-support/monogame-text/README.md)。

Playwright CLI 测试通过：75 张图像，渲染表面与窗口后台缓冲区差异 0，Builder 修改 0。六种调用方式覆盖 string、StringBuilder、基本/标量/向量缩放；字形、宽度、裁剪、阴影、两代更新、停用、再激活和 Revert 均验证。上一阶段 34 项集成断言和 1 项 Rust ABI 合同已回归通过。

## Decisions

- 选择四个 DrawString 叶子重载和两个 MeasureString 重载，跳过标量缩放包装层；按完整框架签名筛选，无游戏专属绑定。
- MonoGame 方案在内存桥接类型中增加字体字形校验与 Builder 复制逻辑，避免在缺字时产生异常译文或改写原 Builder。
- 只在全部目标方法完成 ReJIT 编译后报告就绪；单个入口完成不能代表整组能力就绪。
- 无额外托管助手程序集，无磁盘框架改写。原生组件仍走既有 Native ABI，生产 Loader/ABI 无改动。

## Limits

固定框架版本的合成结果，不是星露谷验收。当前字体必须已有译文字形，缺字时保留原文；自动图集补字、复杂语言排版、ReadyToRun/任意内联、跨帧缓存、并发代次与帧时间仍未证明。目标内存元数据与 Profiler 仍驻留到退出。

新增的窗口协议与字体矩阵都是确定性 fixture，所有本机图像和日志仅在 local-test/。不运行归档包，不构建或启动桌面产品。

## Next

本 Slice 已完成。下一步按 Work Plan 设计 Runtime Bundle 直接接入、目标授权、驻留标识与退出后清理；保持缺字透传和自绘对话未覆盖的能力声明。
