# Windows Console / CMD Adapter observe / replace Seam 调研

## 结论

Windows Console 的可见文字不是一个进程内绘图函数，而是至少三层不同边界：

```text
Console client（cmd.exe 或其子进程）
  ├─ WriteConsoleW / WriteFile / WriteConsoleOutput* / VT bytes
  ▼
Console API server 与兼容层（conhost.exe）
  ├─ 经典模式：维护 screen buffer 并负责窗口呈现
  └─ ConPTY 模式：把 Console API 的效果投影成 UTF-8 + VT stream
  ▼
Terminal（例如 Windows Terminal）
  └─ 解析 VT、布局并绘制
```

Microsoft 明确定义 `conhost.exe` 是 Windows Console API server；在现代 Terminal 场景里，
`conhost.exe` 仍负责 API 服务、翻译和兼容，Terminal 通过 Pseudoconsole 接管交互与呈现。
ConPTY 的公开输出边界是一个 UTF-8 stream，其中普通文本与 VT 控制序列交错，而不是
`WriteConsoleW` 调用记录。来源：[Console / Terminal definitions](https://learn.microsoft.com/en-us/windows/console/definitions)、
[CreatePseudoConsole](https://learn.microsoft.com/en-us/windows/console/createpseudoconsole)。

因此首版建议是：

- 把进程内 `WriteConsoleW` 原型声明为**窄 `TextObserve`**，不声明 `TextReplace`；它只证明被注入
  进程中实际经过该函数的 UTF-16 调用，不代表 CMD、Console session 或 Windows Terminal 的完整覆盖。
- 不向系统 `conhost.exe` 注入生产 Adapter。它虽然更接近整场 Console API 服务，但内部 C++ seam
  不是 Windows 公共 ABI，且会把目标授权边界扩大到系统宿主。
- 如果产品允许 GlyphShift 启动并拥有一个新终端会话，最稳定的公开 session seam 是独立 Controller
  创建的 ConPTY。它可以观察同一 session 中多个 client 的合并输出，但首版仍应只声明
  `TextObserve`；通用 replace 必须先解决 VT 解析、流分块、列宽/光标状态和失败开放。
- `WriteFile`、`WriteConsoleOutputW`、`WriteConsoleOutputCharacterW` 和 Process Family 是独立覆盖项；
  不能用一次 `WriteConsoleW` 命中把它们推定为已覆盖。

这里特意区分“官方事实”与“项目判断”：Microsoft 没有公开承诺 `cmd.exe` 的 prompt、内置命令和
各版本输出都固定调用 `WriteConsoleW`。所以从公开一手资料只能得出条件结论：**如果 CMD 自身的一次
输出经过 `WriteConsoleW`，注入 CMD 的 hook 可以观察并在该次调用前改写参数；但是否经过、覆盖多少，
必须由合成 fixture 与授权版本实测证明，不能写成平台合同。**

## 1. 各 seam 覆盖什么

| seam | 能直接拿到什么 | 覆盖范围 | 明确漏项 | 首版判断 |
|---|---|---|---|---|
| client 内 `WriteConsoleW` | counted `wchar_t` / UTF-16 buffer、字符数、console output handle | 仅当前被注入进程、仅实际调用此 API 的输出 | 其他进程、`WriteFile`、`WriteConsoleOutput*`、重定向后的 stdout、未经过此导出的实现 | `TextObserve` 原型 |
| client 内 `WriteFile` | 任意 I/O device 的 bytes、byte count，且可能 overlapped | 当前进程中经过该函数的 file/device/pipe/console 写入 | 没有天然 Unicode/Console 语义；不覆盖其他输出 API 或其他进程 | 需严格 handle 分类和有状态解码后再做 observe |
| Console API server / `conhost.exe` | 每条 Console API request；`WriteConsoleW` 服务入口有 UTF-16 | 同一 Console session 中所有连接到该 host 的 client；也能看到低层 screen-buffer API | stdout/stderr 重定向到普通 file/pipe 后绕过 Console；不是稳定公共 hook ABI | 不做生产注入 |
| ConPTY output pipe | 整个 session 的 UTF-8 plain text + VT sequence stream | 同一 Pseudoconsole 中多个 client 的最终投影效果，包括 legacy Console API 的兼容翻译 | 无逐记录 PID；不保留所有原始 API 调用边界/源字符串；不能接管任意既有 session | 独立 Controller、observe-only 候选 |
| Windows Terminal 内部连接/renderer | Terminal 已解码后的 buffer/render state | Terminal 自己拥有的 pane/session | 私有产品实现，不是公共 Windows Adapter ABI；版本与产品耦合 | 不作为通用首版 seam |

### `WriteConsoleW`

`WriteConsoleW` 接收 output handle、字符 buffer、`nNumberOfCharsToWrite`，并通过
`lpNumberOfCharsWritten` 返回实际写入字符数；它要求 console handle，标准句柄若已重定向到文件则
调用失败，官方建议用 `GetConsoleMode` 区分 console handle 与重定向 handle。它写入当前光标位置，
并随字符写入推进光标。[WriteConsole](https://learn.microsoft.com/en-us/windows/console/writeconsole)

这使它成为干净的 UTF-16 observe seam，但不是“所有 stdout” seam：

- `WriteConsoleW` 只接受 Console screen-buffer handle；重定向输出应走 `WriteFile`。
- `WriteConsoleA` 和面向 console handle 的 `WriteFile` 是 byte / code-page 路径。
- 低层 `WriteConsoleOutputW` 直接写二维 `CHAR_INFO` cell block，
  `WriteConsoleOutputCharacterW` 从指定坐标写连续 cell；两者均不需要调用 `WriteConsoleW`。

直接来源：[High-level Console I/O](https://learn.microsoft.com/en-us/windows/console/high-level-console-input-and-output-functions)、
[WriteConsoleOutput](https://learn.microsoft.com/en-us/windows/console/writeconsoleoutput)、
[WriteConsoleOutputCharacter](https://learn.microsoft.com/en-us/windows/console/writeconsoleoutputcharacter)。

OpenConsole 的实际 server source 也把这些 verb 分开分派：dispatch table 分别注册
`ServerWriteConsole`、`ServerWriteConsoleOutput` 与 `ServerWriteConsoleOutputString`，不能假设 hook
其中一个等于覆盖另两个。[ApiSorter.cpp](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/server/ApiSorter.cpp#L40-L76)

### `WriteFile`

`WriteFile` 是通用 file/device API：输入是 bytes 而非 Unicode 字符，可用于 console buffer、file、
pipe、socket 等；它同时支持同步与异步写入。异步操作期间 caller buffer 必须保持有效，且
`ERROR_IO_PENDING` 不是失败。[WriteFile](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-writefile)

所以一个通用 `WriteFile` hook 不能直接发布“观察到 Console 文字”：

1. 先确认 handle 是本进程的 Console output handle；`FILE_TYPE_CHAR` 仍可能是其他字符设备，
   `WriteConsole` 文档给出的实用判定是 `GetConsoleMode` 成功。
2. 再按当前 Console output code page 解码。CP 65001 需要跨调用保存不完整 UTF-8 状态，DBCS code page
   需要保存 lead byte；不能逐次把任意 byte buffer 当完整字符串。
3. 对 file/pipe 重定向默认 fail-open，不观察或替换；它可能是二进制协议、任意编码，或 shell pipeline
   的中间数据，并不等于可见终端文字。
4. 若未来扩到 overlapped pipe/file，替换 buffer 的生命周期必须延续到 I/O 完成；栈上临时译文在
   `WriteFile` 返回时释放会违反 API 合同。

OpenConsole 自身的 A 路径印证了这种有状态解码要求：CP 65001 使用持续的 `u8state`，其他 code page
会跨调用缓存 DBCS lead byte，再统一转换到 UTF-16 后进入写入逻辑。
[OpenConsole `_stream.cpp`](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/host/_stream.cpp#L486-L618)

### `conhost.exe`

`conhost.exe` 的公共职责是全部 Windows Console API 的 server 以及经典 Console UI；在 Terminal
场景中它仍保留 API servicing 与 compatibility translation。
[Console / Terminal definitions](https://learn.microsoft.com/en-us/windows/console/definitions)

官方源码显示 `ServerWriteConsole` 在 Unicode 分支把 request buffer 构造成 `std::wstring_view`，再调
`WriteConsoleWImpl`；host 内部的 `DoWriteConsole` 已经收到 Unicode，并按 output mode 选择 legacy
写入或 VT state machine。[ApiDispatchers.cpp](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/server/ApiDispatchers.cpp#L339-L396)、
[`_stream.cpp`](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/host/_stream.cpp#L448-L470)

这证明 host 侧存在强 observe seam，但不自动使它成为合格产品 seam：

- 公开稳定合同是 Console API 与 ConPTY，不是 `ApiDispatchers` / `ApiRoutines` 的内部 C++ 地址。
- host 作用域是 Console session，不是 GlyphShift 当前授权的某一个 executable；同一 session 可连接多个
  client，替换可能影响用户没有授权的进程。
- 不同 API 的 payload 与返回合同不同；中央 hook 仍需分别处理 stream write、cell block、fill、scroll
  等 verb，而不是“拿到一条字符串就完成全部替换”。
- 经典 host 与 headless ConPTY host 的最终输出路径不同，Windows 更新还可改变内部实现。

因此本文只把 conhost internal seam 当作架构证据与合成 host 参考，不建议注入系统 conhost。

### ConPTY 与 Windows Terminal

Pseudoconsole 由 host application 在启动 child 之前创建，通过
`PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE` 交给新进程。host 从 output channel 读取呈现信息并解析 text / VT；
同一 session 仍允许多个 character-mode client 连接，channel 一律 UTF-8。
[Pseudoconsoles](https://learn.microsoft.com/en-us/windows/console/pseudoconsoles)、
[Creating a Pseudoconsole session](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session)

Windows Terminal 官方源码正是这条链：connection 创建 ConPTY、启动 attached client、持续排空 output
pipe，再把 UTF-8 增量解码成 UTF-16 交给 Terminal output。
[ConPTY 创建与 output thread](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/cascadia/TerminalConnection/ConptyConnection.cpp#L400-L477)、
[pipe 读取、增量 UTF-8 解码与 TerminalOutput](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/cascadia/TerminalConnection/ConptyConnection.cpp#L737-L839)

反向看，ConPTY host 会把内部 UTF-16 输出转换成 UTF-8，再用 `WriteFile` 写入 output pipe；
`WriteCharsVT` 先让 state machine 处理字符串，然后把文本与必要的 VT 注入写给 terminal。
[VtIo UTF-16 → UTF-8 与 pipe write](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/host/VtIo.cpp#L500-L603)、
[`WriteCharsVT`](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/host/_stream.cpp#L378-L434)

这带来两个关键差异：

- 在 Windows Terminal 下，client 的 `WriteConsoleW` 仍先由 conhost 兼容层服务；Terminal 进程本身通常
  不会收到该 Win32 调用，只收到 ConPTY 的 UTF-8/VT 结果。因此给 CMD 注入 `WriteConsoleW` hook 与给
  Terminal 注入绘图 hook 是两件不同的事。
- 现代应用更可能启用 `ENABLE_VIRTUAL_TERMINAL_PROCESSING` 并把控制序列与普通文字一起通过
  `WriteFile` / `WriteConsole` 写出。VT sequence 可以跨多个调用、甚至在任意 byte/character 位置拆分，
  所以 raw-buffer substring replacement 不安全。

直接来源：[Console Virtual Terminal Sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)、
[GetConsoleMode output flags](https://learn.microsoft.com/en-us/windows/console/getconsolemode)。

Microsoft 还说明 ConPTY 对 legacy Console API 维护一个模拟环境，并只把 API 的**效果**投影到最终
host；这意味着 ConPTY output stream 是可靠的 session presentation seam，但不保证保留原 API
buffer、调用次数或 process attribution。
[Classic Console APIs versus VT](https://learn.microsoft.com/en-us/windows/console/classic-vs-vt)

## 2. 在 CMD 内 hook `WriteConsoleW` 的准确边界

### CMD 自身输出

条件性答案是“可以，但只限实际经过该函数的调用”。进程内 binary detour 会重写目标函数并把对该
目标函数的调用转到 detour；它不会把其他函数自动折叠到同一入口。
[Microsoft Detours：Using Detours](https://github.com/microsoft/Detours/wiki/Using-Detours)

公开资料没有把 `cmd.exe` 内部使用哪个输出 API 固定为 Windows 合同。因此首版不能声明：

- “CMD prompt、`echo`、所有内置命令都经过 `WriteConsoleW`”；
- “命中一次 `WriteConsoleW` 就覆盖整次命令执行”；
- “classic conhost 与 Windows Terminal 下，各 Windows / CMD 版本调用形态完全相同”。

正确验收应分别触发 prompt、内置命令、批处理、错误输出、重定向和 VT mode，记录每个 seam 的命中；
没有命中只能说明该 seam 不覆盖当前路径，不能把输出推测为别的 API。

### 子进程输出

不覆盖。默认情况下，新 Console process 可以继承父进程的 Console，因此 CMD 与 child 可以连接到
同一个 Console session；但 child 仍是独立进程，并在自己的执行上下文中发出输出调用。
[Creation of a Console](https://learn.microsoft.com/en-us/windows/console/creation-of-a-console)

Microsoft 文档说明每个 user-mode process 有私有 virtual address space；Detours 也要求把 detour DLL
插入需要拦截的 process。由此可以直接推出：CMD 内的 hook 不会因为 child 继承同一 Console 就自动
出现在 child 中。[User mode and kernel mode](https://learn.microsoft.com/en-us/windows-hardware/drivers/gettingstarted/user-mode-and-kernel-mode)、
[Microsoft Detours：Using Detours](https://github.com/microsoft/Detours/wiki/Using-Detours)

所以若选择 TargetProcess seam，Process Family 必须对每个明确授权的 child 单独部署 Adapter；即使
如此也只是扩大 client-call 覆盖，并不等于 session 全覆盖。反之，ConPTY output channel 可以看到
同一 Pseudoconsole 中多个 client 的合并 presentation stream，但公开 stream 本身没有逐段 PID 字段。
[Pseudoconsoles](https://learn.microsoft.com/en-us/windows/console/pseudoconsoles)、
[CreatePseudoConsole](https://learn.microsoft.com/en-us/windows/console/createpseudoconsole)

### 重定向与 pipeline

标准句柄可由 parent 显式设置或继承，可能指向 console、file、pipe、socket 或其他 device；一旦 stdout
被重定向，`WriteConsoleW` 会失败，应用应使用 `WriteFile`。
[Console handles](https://learn.microsoft.com/en-us/windows/console/console-handles)、
[Console / Terminal definitions](https://learn.microsoft.com/en-us/windows/console/definitions)

因此 shell pipeline 中 child 写入 pipe 的中间 bytes 不应被当成最终可见文字，CMD 的
`WriteConsoleW` hook 也没有理由自动看到它。只有某个被注入进程后来把结果重新写回
`WriteConsoleW`，那一次新的调用才进入该 seam。

## 3. Adapter 放在哪里

### 方案 A：TargetProcess（首个最小原型）

建议首个合成/授权原型放在 TargetProcess，并把名字和 Feature 写窄：

```text
Windows Console Client WriteConsoleW Observe Adapter
scope: TargetProcess
features: [TextObserve]
seams: [WriteConsoleW]
not-claimed: [TextReplace, FontSubstitute, LayoutAdjust, SessionCoverage]
```

优点是沿用当前授权、注入、activate/deactivate 与 fail-open 模型；输入已是 counted UTF-16，且
`WriteConsoleW` 是同步 API，没有 `WriteFile` overlapped buffer 生命周期问题。缺点是覆盖面窄，需要
Process Family 逐进程部署，且不能宣称 Windows Terminal 或整个 Console session 已支持。

如果加入 `WriteFile`，应作为显式扩展 seam，而不是悄悄扩大同一实现：只接收经
`GetConsoleMode` 验证的 output handle；按 output code page 做跨调用 decoder；redirected file/pipe
保持 bypass。通过这套合同前不提升 Feature。

### 方案 B：Console Host（不建议）

host 侧有中央 Unicode 与 screen-buffer seam，理论上能覆盖同 session 的多个 client；但这同时破坏
当前最重要的 ownership 边界：目标从用户授权 executable 变成 Microsoft 系统宿主，并可能混入同
session 的其他 client。内部函数没有稳定 ABI，错误会影响整场 Console session。因此不建议作为
首版或生产部署位置。

### 方案 C：独立 Controller 拥有 ConPTY（长期最稳的 session seam）

如果 GlyphShift 的产品形态允许“从 GlyphShift 启动一个受管终端会话”，则由 Controller 调
`CreatePseudoConsole`、启动 shell、排空 output pipe 是最稳定的公开 session boundary。它天然覆盖
同一 session 的 attached child，并避免向 `conhost.exe` 注入。

但它不是任意既有 CMD / Windows Terminal pane 的透明接管：公开流程要求先创建 ConPTY，再通过
`PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE` 创建 attached child；没有公开 API 把第三方 reader 插到一个
已经由 Windows Terminal 拥有的 output pipe 中。
[Creating a Pseudoconsole session](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session)

ConPTY channel 必须由独立线程持续排空；Microsoft 明确警告双向 channel 在同一线程可能死锁，关闭
时也可能还有最终 frame 和 attached clients 输出，必须继续 drain。
[Creating a Pseudoconsole session：channel / teardown](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session)、
[ClosePseudoConsole](https://learn.microsoft.com/en-us/windows/console/closepseudoconsole)

建议这个长期候选的首版声明仍是：

```text
Windows ConPTY Session Observe Adapter
scope: ControllerOwnedSession
features: [TextObserve]
input: UTF-8 + VT stream
evidence: session stream（无 process attribution）
not-claimed: [TextReplace, ExistingSessionAttach, PerProcessRouting]
```

## 4. 为什么首版不声明 `TextReplace`

### 4.1 `WriteConsoleW` 变长替换不是只换两个参数

同步调用中用临时 `std::wstring` 替换 buffer 与 count 在 ABI 上可行，但透明语义仍未成立：

- caller 的 `lpNumberOfCharsWritten` 表示它提交的 source buffer 消耗量；原函数若接收译文，返回的是
  译文写入量。译文长度改变后必须映射回 source count。
- OpenConsole server 明确保留“写入一部分字符串后返回错误”的合同。若译文已部分输出，hook 无法再
  撤销并改写原文；没有 replacement-prefix → source-prefix 映射就不能正确返回 consumed count。
  [ServerWriteConsole partial-write comment](https://github.com/microsoft/terminal/blob/e74649d5f18b1c123556461b30f763407aea558f/src/server/ApiDispatchers.cpp#L357-L377)
- Console 的光标、wrap 和 processed-output 行为由实际写入字符与模式决定；后续调用还可能显式移动
  光标。译文改变 cell width 或换行会改变后续状态。
  [High-level Console I/O](https://learn.microsoft.com/en-us/windows/console/high-level-console-input-and-output-functions)、
  [GetConsoleMode](https://learn.microsoft.com/en-us/windows/console/getconsolemode)
- UTF-16 的一个 supplementary character 由 surrogate pair 表示；调用分块或错误截断可能把 pair
  拆开。invalid / standalone surrogate 的行为未定义，Adapter 不能按单个 `wchar_t` 做字符匹配。
  [Surrogates and supplementary characters](https://learn.microsoft.com/en-us/windows/win32/intl/surrogates-and-supplementary-characters)
- 当 VT processing 开启，普通文字与控制序列可能在同一 buffer 中，sequence 还能跨调用拆分；直接
  substring replacement 可能修改控制 payload 或破坏 parser state。
  [Console Virtual Terminal Sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)

仅限制“UTF-16 code-unit 数相同”也不等于安全：相同 code-unit 数不保证相同 terminal cell 宽度，
也不能处理 cursor control、组合字符或多次分块。因此 length-preserving 可以作为测试 case，不应直接
成为通用 `TextReplace` 声明。

### 4.2 ConPTY raw stream replacement 需要终端状态机

ConPTY stream 是 UTF-8 text 与 VT commands 的增量流。安全写回至少要做到：

1. 增量 UTF-8 decoder 不丢跨 chunk code point；
2. VT parser 只把 printable text token 交给 Dictionary，控制 token 原样保留；
3. transform 前后保持 byte order，不能在已经输出半个 transformed frame 后再“fail-open”回原 bytes；
4. 跟踪 cursor、wrap、alternate screen、erase/insert 与 resize 对 state 的影响；
5. 明确译文 cell width 改变后，是允许自然 reflow，还是要重写后续 cursor operations；
6. deactivation 时继续解析 bypass stream，或只在 parser 回到 safe ground state 后切换，否则重新激活
   会从半条 escape sequence 开始。

Windows 文档明确说 VT command 与 printable text 交错，并控制 cursor、颜色和其他操作；ConPTY host
负责 decode 与 layout，而不是提供独立 text record。
[CreatePseudoConsole](https://learn.microsoft.com/en-us/windows/console/createpseudoconsole)、
[Console Virtual Terminal Sequences](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)

在这些合同与测试存在前，ConPTY 只能声明 `TextObserve`。如果未来只支持“非交互、顺序追加、无
cursor rewrite”的 plain-flow profile，也必须作为显式受限能力，不能冒充通用终端替换。

## 5. deactivate 与 fail-open 合同

### TargetProcess hook

- hook 安装一次，`activate/deactivate` 只切换原子 feature gate；inactive 时以原 buffer、原 count、原
  output pointer 调一次 original，不在热切换时 detach/重写指令。
- 每次进入 hook 时 snapshot activation generation。若要求 deactivate acknowledgement 后绝无旧策略
  写回，就用 in-flight counter 等待已进入调用退出；否则要明确允许最多一个已在途调用完成。
- observe queue 必须有界且不阻塞 Console writer；队列满、capture sink 失败或 controller 不可用时
  丢诊断而不是阻塞或更改应用输出。
- lookup、UTF-16 validation、分配、reentry guard 或任何内部异常失败时，立刻以原参数调用 original；
  original 只能调用一次。
- 若 future replace 已调用 original 且发生 partial write，就不能再次输出原文补救；这也是首版只
  observe 的核心原因。

Microsoft Detours 的官方文档同样要求 detour 与 target 签名/调用约定完全一致，并说明二进制改写不受
Microsoft 支持或保证；这强化了“功能 gate + fail-open”而不是频繁 attach/detach 的选择。
[Using Detours](https://github.com/microsoft/Detours/wiki/Using-Detours)

### ConPTY Controller

- inactive 必须 byte-for-byte、顺序不变地 relay raw stream，同时继续 drain channel。
- 若 parser 在 inactive 时停止维护状态，reactivate 只能等到明确的 parser reset/ground state；更稳妥
  的做法是始终解析、仅关闭 observation/publication/transform。
- decoder/parser/lookup 失败时，对尚未发出的完整 input chunk 原样 relay；任何已经发出的 prefix 都
  不能重复。
- input/output pump 分线程；shutdown 继续 drain 到 pipe broken/closed，并为旧 Windows 的
  `ClosePseudoConsole` 阻塞差异留出状态机，而不是同步等待一个永不排空的 pipe。
- deactivation 只影响后续 stream；已经写入 terminal scrollback 的译文不会自动恢复，这一点必须进入
  产品语义，不能沿用 GUI 重绘后恢复原文的假设。

## 6. 最小验证矩阵

仓库内只需使用 deterministic synthetic fixtures；授权 CMD / Terminal 的真实证据继续放在
`target/local-test/evidence/`。

### 合成 client

至少提供以下互不混淆的 writer：

1. `WriteConsoleW`：完整 BMP、surrogate pair、pair 跨调用、embedded control、长短不同文本；
2. console-handle `WriteFile`：CP 65001 多字节跨调用、一个 DBCS code page、VT on/off；
3. `WriteConsoleOutputW` 与 `WriteConsoleOutputCharacterW`：证明 `WriteConsoleW` hook 不命中；
4. parent + child：共享 Console / ConPTY，但只给 parent 注入，证明 child 调用不被 parent hook 捕获；
5. redirected stdout/stderr：file 与 pipe，证明 Console observe Adapter bypass；
6. failure writer：模拟 partial write、error、broken pipe 与 capture sink failure。

### 状态验收

| 状态 | TargetProcess Adapter | ConPTY Controller |
|---|---|---|
| baseline | 原文字节/字符、返回值与 cursor snapshot | raw stream 原样到 synthetic terminal |
| active observe | observation 有来源 seam；输出与 baseline 一致 | printable token 可观察；VT 效果与 baseline 一致 |
| fail-open | lookup/capture/validation 失败仍只输出一次原文 | parser/lookup 失败后未发送 chunk 原样 relay，无重复/乱序 |
| deactivate | 后续调用完全 bypass；明确处理 in-flight | 后续 bytes 原样 relay且继续维护 parser state |
| reactivate | 新 generation 恢复 observe，无重复 hook | 不从半条 UTF-8 / escape sequence 中间恢复 |
| child | 未注入 child 不得误报已覆盖 | 同 session child 出现在合并 stream，但无 PID 归属 |

授权实测至少横跨 classic Console Host 与 Windows Terminal/ConPTY，并分别记录 prompt、内置输出、
external child、stderr、重定向、批处理和 VT application。真实命中只能提升对应 seam 的证据，不能从
某个 CMD/Windows 版本外推成全部 Console client 的保证。

## 7. Roadmap 建议

1. **先做 `WriteConsoleW` observe-only 合成 Adapter**，只声明 `[TextObserve]`；通过授权 CMD 版本后也
   只显示“该进程、该 seam 已观察”，不显示“Console 已支持”。
2. **随后研究 console-handle `WriteFile` observe seam**：handle 分类、code-page state、VT parser 与
   split-call fixture 完成后，再决定是否和 `WriteConsoleW` 组成一个 Console Client Adapter。
3. **Process Family 只解决逐进程部署，不冒充 session ownership**；child 输出覆盖必须按实际注入与
   seam 命中报告。
4. **不注入系统 conhost**。如果产品确实需要完整 session，另立 Controller-owned ConPTY experience，
   先交付 `[TextObserve]`。
5. **`TextReplace` 暂不声明**。只有 source-consumption mapping、VT state、cell-width/cursor 行为、
   fail-open 与 activate/deactivate 全部有 deterministic fixture 后，才考虑一个明确受限的 replace
   profile；不把限制藏在 Dictionary Entry 或软件品牌分支中。

这一路由保持 Dictionary 仍是纯 `source + translation`：Console API、process/session scope、code
page、VT parser state、PID 和 terminal ownership 都属于 Adapter / Runtime evidence，不进入字典。
