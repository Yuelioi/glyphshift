# 仓库代码结构基线

基线日期：2026-08-06

基线 checkpoint：`bbebea9`

## 统计口径

使用 `rg --files` 只统计 Rust、TypeScript、Vue、JavaScript、CSS 与 PowerShell 代码；排除 `target/`、
`node_modules/`、`dist/` 和生成的声明文件。文件长度使用 `Get-Content` 数组元素数统计物理行，包含空行；
不能使用会忽略空行的 `Measure-Object -Line`。

```powershell
$files = rg --files -g "*.rs" -g "*.ts" -g "*.tsx" -g "*.vue" -g "*.js" -g "*.css" -g "*.ps1" `
  -g "!target/**" -g "!**/node_modules/**" -g "!**/dist/**" -g "!**/*.d.ts"
$rows = foreach ($file in $files) {
  [pscustomobject]@{ File = $file; Lines = @(Get-Content -LiteralPath $file).Count }
}
```

结果：158 个代码文件；11 个超过 1200 行，18 个超过 800 行，39 个超过 500 行。

## 超过 1200 行的文件

| 物理行 | 类别 | 文件 | 初始判断 |
|---:|---|---|---|
| 4633 | 生产 + 内嵌测试 | `apps/glyphshift-desktop/src-tauri/src/lib.rs` | 约 3052 行实现与 1581 行测试；请求/视图、应用编排、命令、窗口与字体职责并存 |
| 2851 | 生产 | `crates/glyphshift-desktop-backend/src/lib.rs` | 视图模型、持久化制品、Dictionary、Workflow 与软件目录编排并存 |
| 2781 | 生产 + 内嵌测试 | `crates/glyphshift-desktop-runtime/src/lib.rs` | 约 1817 行实现与 964 行测试；Bundle、Pool、单目标生命周期与协议映射并存 |
| 1757 | Playwright | `apps/glyphshift-desktop/tests/desktop.spec.ts` | 多个桌面功能面集中在单一规格文件 |
| 1593 | 生产 | `crates/glyphshift-translation/src/lib.rs` | Translation Snapshot、Font Policy 与 Workspace 编辑模型并存 |
| 1479 | 生产 + 少量内嵌测试 | `crates/glyphshift-isolated-worker-host/src/lib.rs` | 进程监督、重启、Host、混合 Placement 与协议解码并存 |
| 1339 | 生产 + 内嵌测试 | `crates/glyphshift-capture/src/lib.rs` | 约 941 行实现与 398 行测试；先评估迁出测试再判断实现拆分 |
| 1284 | 生产 + 内嵌测试 | `crates/glyphshift-controller-windows/src/lib.rs` | 约 1040 行实现与 244 行测试；接近而非明显越过实现阈值 |
| 1274 | Rust 合同测试 | `crates/glyphshift-session/tests/session_contract.rs` | 应按 Session interface 场景分组，不改变生产 module |
| 1266 | 生产 + 内嵌测试 | `crates/glyphshift-capture/src/workspace.rs` | 约 904 行实现与 362 行测试；先迁出测试 |
| 1239 | 测试宿主 | `test-support/windows-host/src/lib.rs` | 确定性 Windows 测试基础设施，不属于产品 implementation |

行数是分流信号。拆出内嵌测试后，首批实现体明确超过 1200 行的是桌面 Tauri 壳、Desktop Backend、
Desktop Runtime、Translation 与 Isolated Worker Host。

## crate 拓扑

- Rust workspace：54 个 crate。
- Adapter 家族：25 个 crate；其他产品、Runtime、协议、Dictionary 与测试支持：29 个 crate。
- 入向依赖最高：`glyphshift-domain` 34、`glyphshift-adapter-sdk` 19、
  `glyphshift-adapter-registry` 14、`glyphshift-adapter-native-abi` 13、`glyphshift-translation` 13、
  `glyphshift-capture` 11。
- 内部直接依赖最多：`glyphshift-target-runtime` 14、`glyphshift-adapter-native-host` 12、
  `glyphshift-desktop-runtime` 12、`glyphshift-desktop-shell` 11、`glyphshift-isolated-worker-host` 10。
- `architecture-tests/check.ps1` 当前逐 crate 锁定直接依赖集合，并维护关键源码扫描路径；任何物理分组都
  必须同步这些合同，但不能借目录移动改变依赖方向。

## 初始结构判断

1. 顶层 `lib.rs` 过长首先是同 crate 内 locality 问题，不等同于需要更多 crate。桌面壳只有 `run()`
   这一小外部 interface，适合保留 facade 并建立私有内部 module。
2. 54 个 crate 的扁平目录确实增加导航成本，尤其 25 个 Adapter crate；但 SDK、Registry、Native ABI、
   Native Host、Worker Host 和具体 Adapter 对应不同真实 seam，不能为了目录整齐合并。
3. 可评估 `core/`、`adapters/`、`runtime/` 等物理家族，但应保持 crate 名称，并先定义允许的依赖方向。
   `core` 不应成为新的共享代码垃圾场。
4. 小 crate 不自动代表浅 module。Native/Worker 的进程与 ABI seam、生产与确定性测试 Adapter，可能用
   很少代码提供高 leverage；必须逐项使用删除测试。
5. 第一实施切片应保持序列化字段、Tauri 命令、错误码、持久格式、Native ABI 和 Runtime Bundle 完全
   不变，以纯结构提取验证方法和测试门槛。

## References

- [Workspace 清单](../../../../Cargo.toml)
- [架构依赖合同](../../../../architecture-tests/check.ps1)
- [产品契约](../../../../PRODUCT.md)
- [Adapter 架构既有结论](../../../knowledge/research/market-landscape.md)
