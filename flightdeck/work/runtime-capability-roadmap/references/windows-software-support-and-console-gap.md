# Windows 软件支持分级与 Console 缺口

## 结论

Glyphshift 不能仅凭软件名称、PE 可执行性或成功注入声明“支持”。Windows 软件的真实覆盖至少要按
渲染/文本通道、目标进程归属和运行时证据三项判断。

当前 Runtime Bundle 只提供四个目标进程内 Adapter：`ExtTextOutW`、`TextOutW`、
`DrawTextW/DrawTextExW` 与 `GdipDrawString`。它们能覆盖经目标进程内 GDI、USER32/GDI 或 GDI+
绘制的文字，但不代表传统桌面软件的所有 surface，也不能覆盖由其他进程或终端宿主绘制的文字。

## CMD 诊断

隔离的 x64 CMD 客户端持续输出已知文本时，现有四个 Adapter 能随 Runtime 注入，但连续两轮均产生
零条适用文字信号。因此该现象不是 Dictionary 未命中，而是当前 Adapter 没有经过 CMD 可见文字的
执行路径。

Microsoft 对 Windows Console 的定义解释了这条边界：命令行客户端通过 Console API、屏幕缓冲区
或文本流与宿主交互；`conhost.exe` 是 Console API server 和经典界面宿主，现代 Terminal 则可通过
ConPTY 接管界面并负责布局与图形呈现。`cmd.exe` 本身不是其可见字符的通用绘制所有者。

因此当前结论应表述为：

- CMD 通过基础接入检查并不等于可翻译；
- 当前把 GDI/GDI+ Adapter 注入 CMD 客户端后，没有观察到可处理文字；
- 继续增加另一个普通目标进程绘图 Hook 不能解决宿主进程边界；
- 不按 `cmd.exe` 名称硬编码例外，Console 应作为独立技术通道评估。

## 支持状态

建议把一个软件目标的状态拆成以下证据链：

1. **可接入**：文件、架构、运行状态、授权与 Runtime Bundle 满足基础条件。
2. **已注入**：目标实例加载并激活了所选 Adapter。
3. **已观察**：至少一个 Adapter 在用户触发目标界面后收到文字信号。
4. **已匹配**：观察文本命中字典 Publication。
5. **已写回**：Adapter 报告替换执行；真实软件仍需视觉验收确认布局和字符正确。

“没有观察”还要区分两种结果：测试窗口内确有足够界面活动但信号为零时，才可判断当前 Adapter
不覆盖；没有触发足够文本活动时只能报告证据不足。

## 软件类别的当前判断

| 类别 | 当前判断 | 说明 |
|---|---|---|
| 经典 Win32、MFC、WinForms 的 GDI/GDI+ surface | 候选支持 | 命中四个现有入口时可观察/写回；自绘或混合 surface 仍需 Probe |
| 已验证的 After Effects 2020 surface | 部分实证支持 | 菜单/控件存在 GDI 与 GDI+ 实证，但不能推广到所有 Adobe 版本和全部 surface |
| CMD、PowerShell 等 Console client | 当前不覆盖可见终端文字 | 可见文字由 Console/Terminal host 管理；客户端内 GDI/GDI+ 注入没有信号 |
| Windows Terminal 承载的 shell | 当前不支持原位写回 | Terminal/ConPTY 是独立宿主和文本流边界，需要专门能力 |
| WPF、WinUI、UWP、DirectWrite/Direct2D surface | 未支持 | DirectWrite Adapter 尚未验证；UIA 只能作为条件性 observe-only 候选 |
| Chromium、Electron、CEF、现代浏览器 surface | 未证明支持 | 常见多进程与 Skia/GPU 路径不能由现有四个入口推导，需 Process Family 与真实 Probe |
| Qt、自研画布、GPU 合成桌面 UI | 未证明支持 | 后端和 surface 可混合，不能按框架品牌预判 |
| Direct3D、OpenGL、Vulkan 游戏/工具 UI | 未支持 | 只有真实目标证据出现后才分别评估，不建立万能 Renderer |

## Console 能力候选

Console 不应混入 DirectWrite Adapter，而应作为独立候选方向：

- **Console client observe Adapter**：研究 `WriteConsoleW`、面向 Console handle 的 `WriteFile` 和 VT
  输出；只能证明输出流可观察，不能预设安全原位替换。
- **Process Family**：命令往往由 CMD 启动的子进程实际输出，若只接管根客户端会持续漏字。
- **ConPTY-owned session**：当 Glyphshift 自己创建并拥有终端会话时，可观察 UTF-8/VT 流；它不能
  无缝接管任意已运行 CMD 或 Windows Terminal 会话。
- **UIA、Console buffer 或 OCR**：可作为 observe-only 或显式兜底，不能显示为目标内
  `TextReplace`。

Console Adapter 是否进入实现阶段，应由真实终端翻译用例、输出流可重组性、子进程覆盖和写回安全
共同决定。DirectWrite 仍是更广泛桌面 UI 覆盖的优先候选。

## 产品落点

- 新增软件的前置检查继续只阻断确定失败，不把 Console 子系统程序一律拒绝；Console 程序也可能
  创建自己的 GUI surface。
- 可在检查结果中把“基础接入可用”和“文字覆盖已验证”分开，并对 Console 子系统给出技术风险提示。
- 添加后的短时兼容性 Probe 应展示 Adapter 观察计数和证据状态，而不是笼统显示“支持”。
- 所有分类按 Capability 与证据得出，Core、GUI 和 Runtime 不增加软件品牌或可执行文件名分支。

## 官方依据

- [Microsoft：Windows Console and Terminal Definitions](https://learn.microsoft.com/en-us/windows/console/definitions)
- [Microsoft：Pseudoconsoles](https://learn.microsoft.com/en-us/windows/console/pseudoconsoles)
- [Microsoft：Console Input and Output Methods](https://learn.microsoft.com/en-us/windows/console/input-and-output-methods)
- [Microsoft：Windows Terminal 官方仓库](https://github.com/microsoft/terminal)
