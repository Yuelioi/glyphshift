# 游戏文字适配推进顺序

- [x] [CoreCLR 运行中接入与恢复实验](slices/coreclr-late-attach.md)：已执行方法 → Native ABI 决策 → 两代替换 → 停用排空 → Revert → 新进程无扩展。
- [x] [MonoGame 标准文字实验](slices/monogame-standard-text.md)：固定版本真实 DrawString/MeasureString 的六个入口、已有字形、像素、两代更新与恢复；包装转发通过，任意内联和 ReadyToRun 不在本次结论内。
- [x] [通用 MonoGame Adapter 策略](slices/monogame-adapter-policy.md)：将 UTF-16 边界、字体字形覆盖、fail-open 与词典决策收进框架级 crate，不依赖游戏名称。
- [x] Runtime Bundle 直接接入：原生 Adapter 自行附加当前 CoreCLR，生产 Kernel/Capture 与 ACK 验证通过；目录标识与探针提示完成，停用后驻留到目标退出。
- [x] 生产完整性：Bundle 校验 10 个 Adapter，实际包含 MonoGame DLL；用户数据模式直接启动配套 exe/runtime。
- [x] [缺字字体回退](slices/monogame-font-fallback.md)：复制原图集补字形、帧边界发布、117 帧真实 Runtime 合成回归通过；待目标重启后实机复核。
- [x] [压缩字体图集修复](slices/monogame-compressed-atlas.md)：实机 DXT3 读取异常定位，7 个解码合同和 147 帧回归通过；待加载新 DLL 后实机可见复核。
- [x] [排版空白匹配](slices/monogame-edge-padding.md)：实机说明末尾空格 no_match 定位，精确优先与保留空白的回退匹配回归通过；待目标重启复核。
- [ ] 自绘文字入口发现实验：两个独立合成渲染器与非文字负例，证明发现边界后才允许写回。
- [x] [软折行去重](slices/monogame-soft-wrap.md)：来源声明、统一键、历史只读合并、冲突选择、译文重排与混合 Adapter 边界通过定向回归；待新版目标复核。
- [ ] 扩大真实覆盖：首个授权目标已取得生产 Active 与非零标准文字采集；仍需独立 MonoGame 游戏与更多场景，不将标准文字支持冒充完整对话支持。

## 尚需明确的能力

复杂布局与跨帧测量缓存、并发词典代次、ReadyToRun/任意内联和帧时间需要在扩大能力前单独验证，不随标准文字实验自动晋级。当前补字形只声明有预算上限的 Windows BMP 字形回退。
