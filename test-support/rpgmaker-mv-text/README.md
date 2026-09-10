# MV 完整对白实验

仅用于明确授权、自行编写的 MV 测试项目，不是随产品发布的适配器。
当前验证范围：x86、NW.js 0.29.0 normal、Node 9.7.1、V8 6.5.254.31。
项目标题必须为 `Glyphshift MV Contract`，有固定两行自动对白和选项事件；安装文件不改动。
真实引擎、项目副本和所有输出只保存在 `local-test/`，不作为仓库必需资源。

## 两种合同

- `node --test test-support/rpgmaker-mv-text/adapter.test.cjs`：纯合成策略测试。
- 仓库 Playwright 运行器 + `runtime.spec.cjs`：实际引擎画面。SDK 可用显式 loopback CDP，普通运行器使用实验原生驱动；二者只能选一个。

普通运行器的当前路线只需要本机 Python 标准库和构建出的 x86 helper，不需要 Frida。调用者明确提供 `GLYPHSHIFT_MV_PID`、
`GLYPHSHIFT_MV_EXE` 和 `GLYPHSHIFT_MV_NATIVE_DRIVER`。驱动检查可执行文件、根进程和唯一 renderer。
它只在具名 V8 回调中执行有界请求，错误执行环境不会消费请求，等待超时为 5 秒。
`native-transport.cjs` 用标准输入输出连接，没有另开 HTTP 服务。

## 使用真正的 Runtime

1. 执行 `build-runtime-bridge.ps1 -OutputRoot <local-test-root>`，生成 x86 测试 DLL 和 `glyphshift_mv_remote_driver.exe`。脚本下载并校验固定版本的官方 Node 头文件及 MinHook 源码；后者的 BSD 许可证保留在解压目录。
2. 按仓库 `scripts/cargo-target.ps1` 初始化 Cargo 输出位置，构建 `cargo build -p glyphshift-target-runtime --target i686-pc-windows-msvc`。
3. 执行 `cargo run -p glyphshift-target-runtime --example mv_deployment -- <bridge-dll> <local-test-root>`，生成两代正式 publication 和带桥工件哈希的 deployment。
4. 除上述进程变量外，设置 `GLYPHSHIFT_MV_RUNTIME_DLL` 为刚构建的 Runtime、`GLYPHSHIFT_MV_RUNTIME_FIXTURE` 为生成目录。
5. 设置 `GLYPHSHIFT_MV_NATIVE_SESSION_DLL` 为本次生成的桥 DLL，`GLYPHSHIFT_MV_REMOTE_HELPER` 为同目录的 helper；此路线不导入 Frida，也无需设置其 Python 模块路径。
6. 使用 `node apps/glyphshift-desktop/node_modules/@playwright/test/cli.js test -c test-support/rpgmaker-mv-text/playwright.config.cjs --output <local-test-output>` 运行。

桥不在 DLL 构造函数中注册 Node 模块。测试先由正式 Runtime 验证并加载，未接入 Node 时激活明确失败；
再保留一个测试加载引用，在游戏正常事件循环中执行 `process.dlopen`，原生入口只在匹配该 DLL 路径的
`uv_dlopen` 成功返回时调用注册函数。已经加载的 DLL 也可完成注册，之后正式 Runtime 激活成功。
注册触发器和 V8 队列均在 `native-session.cpp`，使用固定版本 MinHook，测试工具不拥有游戏 Hook。完整对白通过 Text Host 的一次请求
完成采集和查词。Runtime 控制命令从测试驱动线程执行，不在游戏绘制回调中部署组件。
测试检查完整原文、准确译文、两代字图、采集数量、非法输入、仅观测请求与 Runtime 停用。
画面证据是引擎 framebuffer，不是操作系统窗口截图。

原生会话采用单个待执行请求、5 秒等待与有界输出。停用取消排队请求并禁用自己的 Hook，
已完成的会话可以再次启动；跳板和 DLL 驻留至进程退出。错误页面不消费请求，超时请求不得在页面恢复后执行。
helper 在实际使用的进程句柄上验证路径、创建时间和 x86 位数；远程入口按对应模块 RVA 定位。
超时的远程线程不被强行终止，其可能使用的分配也不提前释放。

旧 Frida 路线保留用于入口对照：不设置 helper 时，驱动按 `GLYPHSHIFT_MV_NATIVE_SESSION_DLL`
选择仅控制 Frida 脚本或旧全 Hook 脚本。不能把旧实验路线当作正式产品依赖。

测试会重新开始自行编写的测试对白，以验证下一次显示；这不是适配器刷新方案。
未解决当前对白恢复、选择和控制符、多适配器绘制去重，也未验证商业游戏。
原生模块驻留至进程退出；重建桥后必须重新启动测试项目，不能强行卸载 Node 引用的 DLL。
Frida、实验模块和部署生成器不进入 Runtime Bundle 或产品目录。
本目录的任意脚本求值和通用远程控制仅用于自行编写的 fixture；进入产品前须替换为固定引擎脚本、会话授权和既有 Controller 的装载流程。

## 由 Runtime 激活负责安装

构建时追加 `-ManagedBootstrap`，运行时设置 `GLYPHSHIFT_MV_MANAGED_BOOTSTRAP=1`，即可选择
`managed-runtime.spec.cjs`。该合同需要新启动的受控项目；不能与上一种桥合同共用已经安装过入口的进程。
生成器、Runtime 路径与 helper 设置保持相同。

`engine-bootstrap.cpp` 从本 DLL 的路径加载 Node 模块，执行构建时嵌入的固定 `adapter.cjs`。
Runtime 调用适配器激活时自动启动原生会话并安装对白入口；停用先关闭决策，再撤销自己的 JS 入口和原生 Hook。
测试不发送 `process.dlopen` 或 `installMessageAdapter`，只用诊断求值读取画面与运行自行编写的对白事件。
合同验证接入超时回滚后可重试、两代实际字图、单次采集、4 轮自动安装/撤销与原函数引用恢复。
当前仍仅支持下一次对白恢复，不是已显示对白的即时恢复。
