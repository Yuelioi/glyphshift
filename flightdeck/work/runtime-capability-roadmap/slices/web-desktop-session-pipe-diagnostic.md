# Web Desktop Session 继承管道诊断

Status: Complete

## Outcome

Chromium 父进程继承管道 transport 作 **Go**。隐藏目标的命令/事件管道、Browser handshake、page target
attach、隔离 Runtime 读取、无调试 endpoint、目标退出断线共十项检查全部通过。该结果只证明
GlyphShift 启动并持有 Chromium 子进程时的 transport 可行，不证明任意 Electron、CEF 或 WebView
软件可接入。

## Question

在 GlyphShift 明确作为父进程启动授权 Chromium 目标的前提下，`remote-debugging-pipe` 能否在 Windows
使用父子进程继承句柄完成 browser handshake、page target attach、隔离 Runtime 调用和目标退出回收，且
不创建本地监听端口或可供无关进程复用的 endpoint？

## Delivery

- [x] 用一次性无窗口 Chromium 子进程和独立空 profile 验证继承管道，不启动真实第三方软件。
- [x] 经管道完成 Browser handshake、page target attach 与隔离 Runtime 的确定性文字读取。
- [x] 确认没有 `DevToolsActivePort`、没有调试地址，并明确句柄只存在于父子进程关系中。
- [x] 目标退出后协议管道必须关闭，父进程不能继续把会话报告为 Healthy。
- [x] 删除一次性原型，只保留可移植结论；不把 Chromium 结果外推为 Electron、CEF 或任意 Web 软件支持。

## Result

- Chromium 在父进程提供的 fd 3/4 管道上完成 browser handshake；随后创建 page target、attach flat
  session，并在独立 execution world 读取确定性可见文本。
- 运行期间 profile 没有 `DevToolsActivePort`；按浏览器进程检查 TCP Listen 结果为零，不存在第二客户端
  可发现和复用的 loopback endpoint。
- `Browser.close` 后事件管道关闭且子进程有界退出，父进程可据此把 Session 明确转为 disconnected。
- 结果为十项检查全部通过；一次性原型已删除，浏览器位置、profile 和进程数据只留在本地测试目录。

## Boundaries

- 浏览器位置、profile、进程输出和原始协议记录只位于 `target/local-test/`，不进入 Flightdeck。
- 本切片只回答 transport 可行性，不建设正式 Controller、Extension、Catalog 或 UI。
- 继承管道仅适用于 GlyphShift 启动并持有生命周期的目标；附加到任意已运行软件仍需宿主接口或明确授权。

## Go gate

五项 Delivery 已全部通过，因此 Chromium 父进程私有 transport 为 Go。下一门槛不再研究通用 transport，
而是选择一个授权真实 Web 桌面目标，先确认该目标的正式启动开关、单实例/profile 语义与版本边界；
Electron、CEF、WebView2 和其他宿主仍分别进入 Support Matrix。

## References

- [Web 桌面软件授权会话评审](../references/web-desktop-authorized-session-review.md)
- [Web Desktop Session 进程外 transport](web-desktop-session-transport-prototype.md)
