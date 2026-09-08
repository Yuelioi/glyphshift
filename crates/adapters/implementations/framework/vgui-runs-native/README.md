# VGUI 文字绘制段（待实机验收）

此适配器提供 `windows.vgui.text-run`，目前限定 Windows MSVC x86、可验证的 `VGUI_Surface031` 绘制接口及标准 `IImage::Paint` / 公开 Surface 上下文边界。尚未加入默认 Runtime Bundle 或产品 Catalog。

实现读取标准 MSVC RTTI 与接口方法契约，不读取或修改 TextImage 的文字、私有字段、颜色流或换行缓存。对象名称用于识别通用框架类型，不包含游戏名、安装路径、样本地址或词句规则。

在 Paint 边界内，将尚未执行的逐字绘制交给 Domain 的 `DeferredTextDraw`。只有字体、颜色、基线、已知起点和原生 advance 都连续一致时，才在边界结束处交给共用 Text Host 做整句决策；译文还须落在已经证实的原绘制宽度内。未命中则按原始顺序重放。

继承同一已验证 Paint 方法的派生对象也可使用该边界，无须对象 vtable 与基类完全相同。自定义控件可通过 `PushMakeCurrent / PopMakeCurrent` 的公开配对获得边界，提交发生在原生 Pop 之前，保留原上下文的坐标转换及裁剪。配对错误、嵌套超限和未知操作均保留原始绘制；没有保存或修改控件私有字段。

发生未知接口调用、位置查询、样式变化、异常间距或资源上限时，先重放尚未执行的原始调用，再执行当前操作。不把提前结束的前缀当作完整词条。完整 PrintText 自带绘制边界：最近明确设置且未被未知操作失效的字体、颜色和位置可用于宽度受限的替换，无须强制嵌套在 TextImage 中；缺少这些状态时只观察。RGBA 与打包颜色参数均记录状态并核验各自调用约定。

接口位置有明确的 ABI profile；除坐标 getter/setter 的符号化数据流对应外，还用指令解码核对所有直接调用的方法的可达返回路径及 x86 栈清理。未知方法通过保留寄存器、FP 状态和原始参数的跳板转交原实现，不猜测其函数签名。Shadow vtable 保留原 RTTI 头及全部原接口槽。

原生钩子、接口表和模块引用保留到目标退出；停用等待在途绘制完成并恢复原始绘制决策。该版本不承诺 ARM64、x64 VGUI、任意自定义 Surface、任意混合排版或未验证的接口布局。

验证入口：`scripts/test-vgui-runs.ps1`。独立 C++ 宿主包含两个私有对象布局不同的文字模块，覆盖整句替换、更新、停用、宽度拒绝、样式/位置回退、未知操作顺序、浮点参数/返回值与 LastError，以及错误函数 ABI 在接口表变更前的拒绝。打包颜色和独立 PrintText 均有先失败再修复的回归案例，覆盖换代与恢复。

授权实机已经取得完整字幕、独立 PrintText 标题/说明及标准 TextImage 列表文本。独立 PrintText 两代决策和原生尺寸检查非零，不代表可见菜单成功；定时前台截图仍为原文。公开上下文已让菜单逐字调用不再缺少归属，但仍有状态回退需要定位。因此实际替换、更新和恢复尚未验收，不能把合成合同当作游戏已修复。

本地诊断导出分别记录激活阶段、独立/延迟绘制计数、尺寸接受/拒绝、Paint 路径计数与缺失状态位，不保存文字、对象指针或进程标识。计数是独立采样，不构成原子快照，更不能代替像素验收。

公开依据：[VGUI ISurface](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui/ISurface.h)、[IImage](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/vgui/IImage.h)、[TextImage Paint](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/TextImage.cpp)。


当前暂停点：单次设置坐标后连续画字的模式已在独立合同中先失败再修复。缓存期间按原生 advance 维护暂存光标，显式位置变化仍参与连续性检查。该修正已载入实机，但末次采集仅覆盖启动提示页，尚未复验主菜单；用户已暂停实机推进。完整证据边界见 [工作区整理](../../../../../flightdeck/work/adapter-coverage-and-x86/references/implementation-review.md)。
