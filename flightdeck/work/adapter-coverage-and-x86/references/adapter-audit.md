# 字体机制与适配器边界审计

基线：已提交的 Qt Widgets 命名空间修复及当前默认 Runtime Bundle。本文是源码审计，没有继续注入实机、没有修改实现。把代码可直接确认的行为、风险推断和既定支持边界分别说明；不将未测试风险表述为用户已经遇到的故障。

## MonoGame 中文字体究竟是不是预设

实际译文来自 Runtime 的 `decide_utf16`，不是从适配器内置中文台词读取。生产选择依据是 MonoGame 的完整 DrawString / MeasureString 签名，没有按游戏类名选择方法。依据：`crates/adapters/implementations/framework/monogame-native/native/profiler.cpp:61`、`native/monogame_profile.h:32`。

`FontFallback.Resolve` 读取实际 SpriteFont.Characters，再从本次实际译文找出缺失字符；需要的字形进入 Requested 集合，由 Present 帧边界栅格化、合成新字体纹理并在下一帧使用。BuildFont 复制原图集、已有字形和度量，原字体对象和原字符串保持不变。辅助代码编译成字节嵌入同一个受校验 DLL；这不是嵌入一套固定中文图片字库。依据：`managed/FontFallback.cs:32`、`:119`、`:165`，路径均相对上述 monogame-native 目录。

**确实存在固定默认值：** 栅格化依次尝试 `Microsoft YaHei`、`Yu Gothic`、`Malgun Gothic`、`Segoe UI`，使用安装在目标机器上的系统字体；字体候选尚未连接工作流的 Font Policy。每字体最多补 512 个字形、字体缓存计数上限 32、纹理总预算 32 MiB；单图集最多约 4 百万像素，支持的行高为 4–128。代理字符被拒绝，因此补充平面字符和完整复杂 shaping 不在实现内。依据：`managed/FontFallback.cs:12`、`:43`、`:49`、`:64`、`:244`。

不能补齐字形时，托管桥还会检查返回字体是否包含译文全部字符；失败即走原调用，不把缺字方块冒充翻译成功。依据：`native/font_fallback_il.h:99`。因此这是动态缺字回退加固定候选/资源预算，既不是只支持预先准备的几句中文，也还不是任意文字与任意字体都能处理的完整排版系统。

## 建议优先处理的代码问题

### 1. DirectWrite 的再次启用与缓存 layout 关联（高优先级）

**已确认路径：** `CreateTextLayout` 仅在 Adapter 活跃时记录原文；deactivate 清空全部 layout 记录；DrawTextLayout 要求记录存在；request_refresh 为 noop。因而“启用后创建并翻译 layout → 停用 → 再启用 → 应用继续复用同一个 layout”会失去关联，直到宿主重新创建该 layout。晚附加前就已经创建的 layout 同样不自动具备原文关联。

另有 4096 条记录上限，按创建序列淘汰旧记录，不能据此保证仍活跃的长寿命 layout 全部保留。记录键是地址，还应补充 COM 对象销毁/地址复用的验证。

依据：`crates/adapters/implementations/native/directwrite-native/src/lib.rs:153`、`:177`、`:342`、`:379`、`:535`。下一步应建立复用同一 layout 的启停合同，再确定原文关联的寿命与回收方式，不能直接取消缓存上限。

### 2. Qt Quick AutoText 的原文/译文格式门控不一致（高优先级代码风险）

`Qt::labels` 根据控件当前 text 判定 eligibility；`eligible_text` 对 AutoText 中的 `<` 拒绝，但新译文在写入前没有相同格式门控。原文通过后，若译文包含 `<b>…</b>` 等内容，下一轮会把自己写入的译文判为不适用，恢复原文，再下一轮又可能翻译。Qt 也可能将 AutoText 译文解释为富文本。

依据：`crates/adapters/implementations/framework/qt-quick/src/lib.rs` 的 `eligible_text`、`qt-quick-native/src/qt.rs:203`、`qt-quick-native/src/lib.rs:176`。这是从调用链推导的风险，尚未做真实 Qt 像素复现。建议门控同时使用保存的原文、当前格式和候选译文，补 PlainText / AutoText / 小于号 / HTML 文本的合同，避免改变宿主 textFormat 来强行通过。

### 3. MonoGame 缺字资源配额与配置（中优先级）

`fontCount` 在新缓存分配时增长，除超限回退外没有随旧字体回收而递减；ConditionalWeakTable 本身不会替这个静态计数归还配额。allocatedBytes 仅在构建失败和旧图集退役时扣减，没有看到弱键失效时对当前图集的配额回收路径。因此多次加载/卸载字体的长会话可能耗尽的是历史配额，而不只是当前活跃资源。

依据：`managed/FontFallback.cs:13`、`:49`、`:184`、`:228`、`:234`。这是代码中的配额行为，实际发生频率还需要字体反复重建/设备退出的压力合同；不据此断言已发生原生内存泄漏。

另外应把字体候选变成受控配置或接入 Font Policy，补明确的“等待字形/缺字体/预算耗尽/不支持字符”诊断。目前多个失败分支只保留原文，用户不容易区分原因。

### 4. 架构查询失败不能猜成宿主位数（x86 前置）

`process_architecture` 在 OpenProcess 或 IsWow64Process2 失败后返回 Controller 自身架构。应改成 unknown 并拒绝不确定部署，避免 x86 链路把无法识别的目标猜成 x64。依据：`crates/runtime/controller/windows/src/platform.rs:233`。详细改造见 [x86 报告](x86-feasibility.md)。

## 默认 11 个 Adapter 的覆盖边界

能力清单以 `scripts/runtime-bundle-adapters.zh-CN.json`、`scripts/build-runtime-bundle.ps1` 和实际 descriptor 为准，而不是只看 crate 名或早期文件头。

| 组别 | 当前能力 | 尚未覆盖或需单独处理 |
| --- | --- | --- |
| GDI 三项：TextOutW、ExtTextOutW、DrawTextW/ExW | Unicode Win32 绘制及部分 glyph-index，支持相应字体替换 | 没有独立 A/ANSI 钩子；有些 A 调用会转入 W，但不能保证所有程序都会如此；字形反解、布局宽度和权限仍有限制 |
| GDI+ | GdipDrawString | 不自动覆盖文字已转路径、纹理或缓存后的绘制 |
| DirectWrite | CreateTextLayout + Direct2D DrawTextLayout | 复杂格式、inline object、自定义 renderer、无法关联的既有 layout；见上述缓存问题 |
| Qt Painter | 动态 Qt 5/6 的四类 drawText，Qt 6 的 QT 命名空间 | 不自动覆盖静态 Qt、全部命名空间、QTextDocument/QStaticText 或直接 glyph 路径；x86 thiscall 尚未实现 |
| Qt Quick | Qt 6.8.3 Windows x64 普通 QQuickText 标签及对应菜单/提示 | 版本、ABI 和命名空间范围窄；Input/Edit、富文本、自绘、超出遍历预算等不覆盖；250ms 扫描还需性能与边界诊断 |
| GTK 3/Pango | 动态 GTK3 标准 Pango layout，属性为空或全局统一 | GTK4、局部文本属性、静态链接与其他渲染路径 |
| raylib | 动态 raylib 5.5 DrawTextEx，缺字图集 | 静态链接、业务纹理缓存、复杂 shaping；字体/纹理预算不是无限 |
| MonoGame | CoreCLR/.NET 6+ 的标准 SpriteBatch.DrawString / SpriteFont.MeasureString | 自绘 SpriteText/逐字纹理、旧 XNA/其他 runtime、复杂 shaping、非 BMP；字体候选和资源配额见上文 |
| Unity Mono | 已支持标准 uGUI/TMP 的对象与主线程路线 | IL2CPP、UI Toolkit、NGUI、自绘 mesh 和未满足元数据/主线程门控的游戏 |

以下代码存在并不等于默认产品支持：独立 Direct2D 包未进入当前 Bundle；Unity IL2CPP 为未发布观察原型；Console 暂停；SDL3_ttf 仍为实验。UIA/OCR 属归档边界，不作为本轮覆盖补丁。

## 文档债务

- `unity-mono-standard-ui-native/src/lib.rs:1` 仍称 observe-only/unpublished，实际 native_adapter 已支持替换且默认 Bundle 包含它。
- MonoGame README 仍有“全部构建产物只在 local-test”字样，已与全局 Cargo target 改造不符。
- 应把“声明 x86”“编译出 x86”“x86 实机通过”分开记录，不由 ARCH_X86 位推断现状。

## 推进建议

按用户下一目标，先开展 x86 的 GDI 最小链路，同时修正架构识别失败边界。不要让全部框架迁移阻塞首版，也不要为了覆盖数量放宽 Qt/Unity 等未验证门控。DirectWrite 复用 layout、Qt Quick 格式一致性、MonoGame 配额回收可作为独立小项逐个修复；它们不应被“多加几个 Adapter”掩盖。
