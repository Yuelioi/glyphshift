# Web Desktop Session 进程外 transport

Status: Complete

## Outcome

临时 loopback CDP 的功能合同作 **Go**，生产私有性作 **No-Go**。endpoint 发现、page attach、隔离
Runtime 与目标退出断线全部确定，但同机第二个无凭据客户端可同时连接并执行 browser 级命令。因此
`remote-debugging-port=0` 不能成为 GlyphShift 的生产 Web Session transport。

## Question

GlyphShift 作为父进程启动一个明确授权、无用户 profile 的 Chromium 类目标时，能否使用官方
`remote-debugging-port=0` 生命周期可靠地发现临时 endpoint、建立 CDP、锁定一个 page target，并在目标
退出时及时关闭会话？即使功能成立，loopback WebSocket 是否满足生产所需的私有性？

## Delivery

- [x] 用一次性无窗口 Chromium 子进程和独立本地 profile 验证临时 endpoint 发现，不使用固定端口。
- [x] 通过最小 CDP client 完成 browser handshake、创建/附加 page target 与隔离 Runtime 调用。
- [x] 目标退出后 WebSocket 必须关闭，会话不能继续报告 Healthy 或保留 page ownership。
- [x] 明确 loopback port 的授权边界，并决定生产 transport 是 Go、受限诊断，还是 No-Go。
- [x] 删除一次性原型，只保留可移植的 transport 结论。

## Result

- 隐藏目标以独立空 profile 和端口 `0` 启动，`DevToolsActivePort` 能在有界时间内提供临时 endpoint；
  仓库没有记录端口、profile 路径或浏览器输出。
- 首个客户端完成 Browser handshake、page target attach、隔离 execution context 与确定性 `Open` 读取。
- 第二个客户端不提供 token、secret 或用户确认即可连接同一 endpoint，并成功执行 browser 级版本命令；
  随机端口和 loopback 地址都不是认证。
- Browser 退出后两个客户端均收到 disconnect，子进程有界退出，临时 profile 被验证位于本地测试根后
  删除。
- 最终 verdict 为七项功能/生命周期检查通过，`productionPrivate = false`；一次性原型已删除。

## Boundaries

- 不启动第三方软件，不读取或复用浏览器/应用真实 profile。
- profile、endpoint、进程输出和原始协议记录只允许位于 `local-test/`，且不进入 Flightdeck。
- 不建设正式 Controller、Extension、Catalog、UI，也不把 CDP 权限缩写成普通 `TextReplace` Adapter。
- 不把随机端口误称为认证或私有通道；同机其他进程的访问能力必须计入判断。

## Go gate

功能门槛全部通过，但安全门槛未通过。端口模式只保留为本地确定性诊断手段，不进入产品；下一步只能
验证由 GlyphShift 父进程独占并继承给目标的 pipe，或由目标宿主主动提供受限插件接口。

## References

- [Web 桌面软件授权会话评审](../references/web-desktop-authorized-session-review.md)
- [Web Desktop Session 最小状态机](web-desktop-session-prototype.md)
