# Web Desktop Session 最小状态机

Status: Complete

## Outcome

最小 DOM ownership 状态机作 **Go**，但只证明页面内逻辑成立，不等于 Web Desktop Session 已可生产。
一次性隐藏 Chromium 原型已从工作区删除，主分支只保留本结论。

## Question

在不使用目标私有符号、不修改用户 profile、也不接触真实软件的前提下，私有 CDP 会话中的隔离 world
能否同时完成：可见 DOM 原文观察、Dictionary generation 热更新、应用自身重渲染后的继续匹配，以及
只恢复仍由 GlyphShift 持有的译文节点？

## Delivery

- [x] 用一次性隐藏 Chromium harness 建立隔离 execution world 与有界 observation binding。
- [x] 只观察普通可见文本，排除 input、textarea、contenteditable、密码、隐藏节点和脚本/样式。
- [x] 验证第一代 Dictionary、第二代热更新与同一进程应用重渲染后的重新匹配。
- [x] 停用时只恢复仍等于本次译文的节点，不覆盖应用已经自行修改的内容。
- [x] 原型代码从主工作区删除，只把状态合同与 Go/No-Go 结论保留在本 Slice。

## Result

- 主 document 与一个 child frame 分别创建独立 world，共观察 `Open`、`Save`、`Owned` 与 `File`
  四个确定性 source；没有读取 input 值、contenteditable 或 hidden 文本。
- generation 1 同时把四个 source 替换为中文；generation 2 在不重建页面的情况下更新主 frame 与
  child frame 的译文。
- 应用把已翻译节点改为新的 `Close` 与 `Application override` 后，Observer 将它们视为新 source，
  旧 `Save`/`Owned` ownership 被释放；第二代 Dictionary 只接管其中已配置的 `Close`。
- 停用后 `Open`、`Close` 与 `File` 恢复各自当前 source，应用自行写入且未被第二代接管的
  `Application override` 保持不变；两个 context 的 owned node 数均归零。
- 一次性原型最终 verdict 6/6 通过；没有产生截图、页面源码、用户数据或持久浏览器 profile。

## State contract

每个已接管文本节点只保存会话内状态：

```text
source -> applied translation -> dictionary generation
```

- DOM 当前值仍等于 `applied translation` 时，Dictionary 新 generation 可以安全重算。
- DOM 当前值不再等于 `applied translation` 时，将其视为应用产生的新 source，而不是强行恢复旧值。
- 停用只恢复当前仍等于 `applied translation` 的节点；否则放弃所有权并保留应用当前值。
- 导航、frame 销毁或 execution context 消失会丢弃该 context 的全部节点状态，不跨文档复用 Node 身份。

## Boundaries

- 原型使用无窗口、无用户数据的确定性浏览器页面，不启动第三方软件。
- 不读取 URL、cookie、storage、网络请求、页面源码、用户输入或不可见文本。
- 不建立正式 Controller、Extension、Catalog、UI 或持久配置；原型不能作为产品支持证明。
- 真实目标、私有 transport、单实例语义和 profile 安全仍由后续 Slice 单独验证。

## Go gate

全部 Delivery 已在同一隐藏会话通过，下一步可单独设计进程外 Web Session transport。该 Go 不授权
连接真实软件，也不允许跳过私有通道、进程生命周期和 profile 安全验证。

## References

- [Web 桌面软件授权会话评审](../references/web-desktop-authorized-session-review.md)
