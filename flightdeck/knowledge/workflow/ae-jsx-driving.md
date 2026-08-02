# 用 JSX 驱动 After Effects 测试场景

创建 composition、solid、effect 和参数变化时优先使用 ExtendScript，而不是 SendKeys 或坐标点击。脚本调用不依赖前台焦点，跨分辨率更稳定。

仓库中的测试脚本是 `scripts/testbed.jsx`。对已运行的测试 AE 实例，可以用同一 AE 可执行文件的 `-r` 参数转发脚本：

```powershell
$aeExe = '<authorized-test-AfterFX.exe>'
$jsx = (Resolve-Path 'scripts/testbed.jsx').Path
Start-Process -FilePath $aeExe -ArgumentList @('-r', $jsx)
```

不要把示例中的 AE 路径写回仓库；实际 executable 来自 GUI 检测、测试配置或用户明确指定。

## 场景脚本原则

- 每轮先关闭未保存测试工程并新建 project，避免跨 `-r` 调用累积同名 comp。
- 添加效果使用语言无关 `matchName`，例如 `ADBE Cartoonify`。
- 失败操作用 JSX `try/catch` 收敛，脚本尽量不写文件。
- 需要 JSX 写文件时，AE 的 “Allow Scripts to Write Files” 权限只能由用户在偏好设置中开启，脚本不能自行提升。

脚本负责制造确定性 UI 状态；注入流程见 [注入与验证流程](inject-verify.md)。日志、截图和
本机效果清单统一写入 `target/local-test/`，不进入 Git。
