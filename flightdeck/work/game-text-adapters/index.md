# 游戏文字适配器

Status: Open

## Goal

沿用 Glyphshift Adapter 模式建立可跨游戏复用的文字翻译能力，以星露谷物语验证首条技术路线。

## Current

用户已确认说明翻译生效，随后发现同一句原文因折行位置不同出现多条记录。已落实 Adapter 声明的软折行归一化：采集与查找使用统一键，历史观测只读合并；已有译文冲突由用户在行内选择，保留原始词典。混选 Adapter 时依据真实观测来源，不改其他 Adapter 的正常换行。平铺译文按原文行宽重新折行。

辅助代码嵌入同一个受校验 DLL，只使用目标现有 MonoGame 类型与 Windows 字体；没有独立插件。缺字图集在 Present 帧边界发布，下一帧测量/绘制使用一致字体，旧纹理延迟释放。旧 fixture 34 项断言及 Rust Loader 合同回归通过；仍保留驻留到目标退出的生命周期约定。

最新桌面 App 已同步构建并启动，默认读取用户数据；可直接打开的程序目录包含配套 runtime，启动明确传递数据与 Runtime 路径。驻留标识已在探针选择器显示。当前进程若加载过前一版驻留组件，需要退出目标后再测试新构建；不得强制卸载 CLR 仍引用的代码。

只读 IL 检查确认普通文字调用 DrawString，对话经 SpriteText 直接提交纹理字形，标准 MonoGame Hook 无法完整覆盖。不按游戏名称的结构扫描找到 6 个候选但有误报，自动自绘文字替换仍未证实。已排除 SMAPI/游戏 Mod 作为当前交付路线。

## Next

已按 [软折行去重验收](slices/monogame-soft-wrap.md) 同步打包并启动新版，生产 Loader 两次校验 10 个 Adapter 通过；等待用户重启目标后复核历史重复合并、冲突选择、不同提示框宽度复用同一译文。

## References

- [工作约束](context.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
- [初次入口研究及已排除的 Mod 路线](references/stardew-initial-feasibility.md)
- [适配方案](references/monogame-adapter-design.md)
