# Web 桌面软件授权会话评审

Status: Complete

## 结论

Electron、CEF 与 WebView2 的页面文字具备很高覆盖价值，但不应伪装成另一个目标进程内 Native Adapter。
建议后续以 **用户显式授权的 Web Desktop Session** 建设独立 Extension：由 GlyphShift 启动或连接一个
明确开启调试/脚本接口的目标，使用受限 CDP 或宿主 API 注入固定的本地脚本，观察可见 DOM 文本并按
Dictionary 更新显示。

当前判断是 **Roadmap 条件 Go，生产 No-Go**：最小 DOM ownership 与 Chromium 父进程继承管道已由
确定性原型闭合，但目标软件的正式启动方式、单实例/profile 语义、宿主版本边界和真实可见收益尚未验证。
现阶段不新增用户选项、不修改 Native Bundle，也不通过 DLL 注入 V8/Blink 私有符号绕过目标授权。

## 三类宿主的接入边界

### Electron

Electron 官方支持 `--remote-debugging-port=<port>`，底层通过 Chrome DevTools Protocol 暴露 renderer。
这要求目标从启动时启用；对已经运行且未开放调试接口的进程，没有等价的通用外部 attach。单实例应用还
可能把第二次启动转发给旧进程，因此 Extension 必须先识别“目标已运行但不可连接”，不能静默重启或杀进程。

### CEF

CEF 的公开 `remote_debugging_port`/同名命令行开关同样只在初始化时启用。端口设为 `0` 可以选择临时端口
并输出 WebSocket endpoint，但这仍是拥有完整页面调试权限的通道，不是普通文本 API。

### WebView2

WebView2 的 `AddScriptToExecuteOnDocumentCreated`、`ExecuteScript` 和 CDP API 由宿主持有的
`ICoreWebView2` 对象调用。它们证明宿主可安全提供 Extension seam，却不意味着外部进程能枚举并接管
任意既有 WebView2 控件。因此通用现有进程 attach 当前 No-Go；只有软件 Extension、宿主插件或在创建
阶段显式交出会话的目标才进入支持范围。

## 最小安全模型

Web Desktop Session 拥有的权限远高于文字 Hook。Chrome 官方已把 remote debugging 用于窃取 cookies
列为真实风险，并从 Chrome 136 起要求默认 Chrome profile 之外的 `--user-data-dir`。Electron/CEF 的
具体行为不能直接套用 Chrome 版本规则，但相同权限风险成立。

原型必须满足：

- 每次连接由用户针对一个已登记软件显式开启，不做后台全局扫描；
- 优先使用父子进程继承的私有 pipe；只有目标明确不支持 pipe 时才评估 loopback 临时端口；
- 不使用固定端口，不监听非 loopback，不把 endpoint、cookie、storage、请求内容或页面快照写入日志；
- 注入脚本随 GlyphShift 本地 Bundle 发布并做完整性校验，不下载或执行远端代码；
- 使用独立 execution world，不开启 Node integration，不调用目标应用的 Electron/Node API；
- Session 结束即移除脚本、binding 和 DOM 修改；连接丢失明确报告“会话中断”，不声称已恢复原文。

若目标只接受新 user-data-dir，但这会丢失用户现有登录、设置或项目状态，则该目标首版不兼容，不能为了
获得调试端口偷偷切换 profile。

### Transport 验证结论

临时 loopback 端口已作生产 No-Go：即使使用随机端口和独立 profile，同机第二个无凭据客户端仍能连接并
执行 browser 级命令。随机性不是认证。

Chromium 继承管道已作 transport Go：父进程提供 fd 3/4，隐藏目标完成 handshake、target attach 与
隔离 Runtime；运行期间没有 `DevToolsActivePort`，浏览器进程没有 TCP Listen，关闭后管道和进程均
有界退出。Chromium 源码把该开关定义为 stdio/指定 pipe 的调试通道，ChromeDriver 的官方测试也明确
pipe 模式不返回 debugger address。

该结果不能自动套用到 Electron 或 CEF。只有目标宿主正式支持该开关，或主动提供等价的私有宿主 API，
并且 GlyphShift 可以从启动阶段拥有其生命周期时，才可复用这条 transport。

## DOM 翻译合同

CDP 的 `Target.setAutoAttach` 可跟踪新 page/frame，`Page.addScriptToEvaluateOnNewDocument` 可在新文档
创建时注入隔离 world，`Runtime.addBinding`/`Runtime.evaluate` 可建立有界双向控制。Chrome Extension
content script 的官方模型也证明隔离 world 可以读取和修改共享 DOM，但 JavaScript 变量与页面隔离。

首版只处理可见普通文本节点：

- 排除 `script`、`style`、`textarea`、`input`、`contenteditable`、密码及隐藏节点；
- 不翻译用户输入、代码编辑器、Canvas、图片、闭合 shadow root 或无文字语义的 GPU 内容；
- 用 `WeakMap` 记录节点原文和本次译文，Dictionary generation 更新时重新决策；
- MutationObserver 只处理新增或由应用重新写入的原文，避免自己的替换形成循环；
- 停用时仅当节点仍等于 GlyphShift 最后写入值才恢复，应用已自行更新的节点不得被旧原文覆盖；
- Observation 以去重、有界 batch 返回，不上传 DOM、URL、cookie、属性全集或页面源码。

隔离 world 只能隔离脚本变量，不能隔离 DOM 变化；目标应用仍可能读取被替换后的 `textContent`，虚拟 DOM
也可能覆盖译文。因此该能力必须显示为“Web 页面文字替换”，并允许目标 Extension 缩小 frame/origin，
不能宣传成与 GDI/Qt 绘制时替换完全相同的无状态 Hook。

## Go gate

只有一个授权且适中的 Web 桌面目标同时通过以下门槛，才开始生产设计：

1. GlyphShift 可在不破坏原 profile、登录和单实例语义的情况下启动并取得私有会话；
2. 可见文本观察、Dictionary 首版、第二代热更新和停用恢复在同一进程可见；
3. 主 frame、子 frame、导航和 renderer 重建具有确定状态，未连接 frame 不误报已翻译；
4. 用户输入、密码、编辑器和敏感 storage/network 数据不会进入 observation；
5. 目标应用重渲染与 GlyphShift 恢复不会互相覆盖，循环和批次均有硬上限；
6. 会话端点只在本地短时存在，未授权进程无法复用。

失败时保持 Roadmap，不退化到 Blink/V8 私有符号、签名扫描或固定应用版本地址表。

## 第一方资料

- [Electron 支持的命令行开关](https://www.electronjs.org/docs/latest/api/command-line-switches)
- [Electron 安全建议](https://www.electronjs.org/docs/latest/tutorial/security)
- [CEF `remote_debugging_port`](https://cef-builds.spotifycdn.com/docs/139.0/structcef__settings__t.html)
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [CDP Target domain](https://chromedevtools.github.io/devtools-protocol/tot/Target/)
- [CDP Page domain](https://chromedevtools.github.io/devtools-protocol/1-3/Page/)
- [CDP Runtime domain](https://chromedevtools.github.io/devtools-protocol/v8/Runtime/)
- [WebView2 脚本与 CDP API 概览](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/overview-features-apis)
- [Chrome remote debugging 安全变更](https://developer.chrome.com/blog/remote-debugging-port)
- [Chrome content script 隔离 world](https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts)
- [Chromium `remote-debugging-pipe` 定义](https://chromium.googlesource.com/chromium/src/+/main/content/public/common/content_switches.cc)
- [ChromeDriver pipe transport 合同测试](https://chromium.googlesource.com/chromium/src/+/main/chrome/test/chromedriver/test/run_py_tests.py)
