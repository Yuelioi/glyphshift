# Crate 拓扑与删除测试

Status: Complete; physical path decision superseded by Stages 7-8

## Goal

逐项判断 54 个 workspace package 是否拥有独立 interface、部署制品或进程 owner，并据此定稿物理目录；
不能因为 crate 小就合并，也不能只为目录整齐制造大规模路径迁移。

## Evidence

- workspace 共 54 个 package：47 个位于 `crates/`、2 个 App、5 个 test-support。
- 当前 package 名已经按 `glyphshift-<capability>` 排序；物理家族与依赖层级并不重合，例如 Native ABI
  属于 Adapter 平台家族但处于 L2，Desktop Runtime 属于 Runtime 家族但处于 L4。
- 前序评估把大量引用迁移视为无 interface 收益的噪音；用户复核后明确指出 25 个 Adapter 的统一物理入口
  本身就是重要导航收益。Stage 7 的实际机械变更为 25 个 workspace member 与 79 个 manifest path 属性，
  外加脚本、架构扫描和文档链接。
- 最小 production crate 仍对应真实 `cdylib`/进程/SDK seam；高复用中心包括 Domain（32 个直接下游）、
  Adapter SDK（15）、Native ABI（13）、Registry（12）、Capture/Runtime Contract（各 11）。

## Revised physical topology decision

- [x] 将全部 25 个正式 Adapter crate 归入 `crates/adapters/`，并以 `platform/` 和
  `implementations/` 区分共享平台合同与具体文字技术。
- [x] Stage 7 先保持其他 package 原位；Stage 8 经独立命名/删除审计后，再落实 `core/`、`dictionary/`、
  `runtime/` 与 `product/` 短目录。App 与 test-support 组合根仍保持原位。
- [x] 保持 crate/package 名、ABI、crate type、制品与发布身份。
- [x] 以 `crates/README.md`、Adapter 子目录 README、家族 partition 和跨层门禁共同表达导航与约束。

前序“全部保留扁平”的结论低估了 Adapter 家族的物理可发现性，现由 Stage 7 明确推翻。目录表达能力
家族，依赖层表达允许方向；descriptor/native/worker 即使处于不同依赖层，仍可在同一 Adapter 家族下保留
各自部署 seam。

## Per-package deletion audit

结论均为“保留”。下表的删除测试描述把该 package 合并或删除后会失去的独立 seam。

### Core 与 Dictionary

| Package | 删除测试 |
|---|---|
| `glyphshift-domain` | 32 个下游共享的 Feature/Generation/Decision 基础语言会被复制或反向依赖。 |
| `glyphshift-translation` | 不可变 Snapshot 与可编辑 Workspace 策略会泄漏进 Runtime/产品 owner。 |
| `glyphshift-capture` | 非阻塞 observation wire、batch producer 与 checkpoint catalog 的独立 schema/恢复合同消失。 |
| `glyphshift-decision` | 纯决策 policy 会被并入 Runtime kernel，失去不启动 Host 的确定性测试 seam。 |
| `glyphshift-workflow` | Workflow/target/font policy 配置语言会被 Desktop 产品模型拥有，阻断跨产品复用。 |
| `glyphshift-extension` | 软件扩展与部署描述的稳定序列化合同会被 Controller/产品实现重复。 |
| `glyphshift-dictionary-package` | 可移植字典制品 schema 会与 catalog 网络/签名策略耦合。 |
| `glyphshift-dictionary-distribution` | Catalog、签名验证、安装/回滚生命周期会污染纯 package model。 |

### Adapter 平台

| Package | 删除测试 |
|---|---|
| `glyphshift-adapter-sdk` | 15 个下游的 descriptor/capability interface 将依赖具体 Host 或实现。 |
| `glyphshift-adapter-registry` | 目录验证与选择策略会被每个 Host 重复，无法统一校验文档 URL/能力。 |
| `glyphshift-adapter-native-abi` | 13 个下游共享的稳定 C ABI 会与 Rust Host implementation 绑定。 |
| `glyphshift-adapter-native-host` | 动态库验证/协商/装载 owner 会泄漏到 Target/Desktop Runtime。 |

### 具体 Adapter 与部署 companion

| Package | 删除测试 |
|---|---|
| `glyphshift-adapter-console` | Console descriptor/观察语义无法独立于 Windows DLL 测试。 |
| `glyphshift-adapter-console-native` | `cdylib` 的 WriteConsole hook 与发布身份消失。 |
| `glyphshift-adapter-direct2d` | Direct2D capability model 会与注入实现耦合。 |
| `glyphshift-adapter-direct2d-native` | `cdylib` 的 Direct2D hook/资源 owner 消失。 |
| `glyphshift-adapter-directwrite` | DirectWrite layout capability model 无法供确定性 Host 验证。 |
| `glyphshift-adapter-directwrite-native` | `cdylib` 的 DirectWrite hook 与 COM 生命周期消失。 |
| `glyphshift-adapter-draw-text-native` | Win32 DrawText 独立 hook 制品会被错误合并进 ExtTextOut 实现。 |
| `glyphshift-adapter-gdi` | GDI call/font/glyph policy 被四个调用者共享，合并会复制安全逻辑。 |
| `glyphshift-adapter-gdi-native` | GDI ExtTextOut `cdylib` 部署制品消失。 |
| `glyphshift-adapter-gdi-native-support` | 三个 GDI DLL 共享的 hook/ABI 支持会被复制。 |
| `glyphshift-adapter-gdi-text-out-native` | TextOut 独立入口与发布制品会被 ExtTextOut 分支吞并。 |
| `glyphshift-adapter-gdiplus` | GDI+ call/font policy 无法脱离 native hook 做确定性测试。 |
| `glyphshift-adapter-gdiplus-native` | GDI+ DrawString `cdylib` 与资源 owner 消失。 |
| `glyphshift-adapter-gtk3-pango` | Pango descriptor/call model 会与跨平台 module 装载实现耦合。 |
| `glyphshift-adapter-gtk3-pango-native` | GTK/Pango 动态 hook 制品与独立失败语义消失。 |
| `glyphshift-adapter-qt-painter` | Qt Painter descriptor/call model 无法独立验证。 |
| `glyphshift-adapter-qt-painter-native` | Qt module/hook `cdylib` 与发布身份消失。 |
| `glyphshift-adapter-raylib` | raylib text call model 会与动态符号 hook 耦合。 |
| `glyphshift-adapter-raylib-native` | raylib `cdylib` 的符号解析/失败开放 owner 消失。 |
| `glyphshift-adapter-uia` | UIA observation descriptor/schema 会依赖 Worker process 实现。 |
| `glyphshift-adapter-uia-worker` | 独立 Worker executable/library 与 UIA timeout/privacy owner 消失。 |

### Runtime、产品与应用

| Package | 删除测试 |
|---|---|
| `glyphshift-controller-sdk` | Controller plugin JSON/stdio 合同将反向依赖 Host。 |
| `glyphshift-isolated-worker-sdk` | Worker transport 合同将反向依赖 Supervisor。 |
| `glyphshift-protocol` | 跨进程 wire/model 与 Connection 状态机会散入多个 Host。 |
| `glyphshift-runtime-contract` | 发布 generation、错误/trace/capture 配置合同会依赖 Kernel 实现。 |
| `glyphshift-target-runtime-contract` | Target ABI 上层消息合同会与注入 Runtime 混合。 |
| `glyphshift-controller-host` | 通用 Controller plugin owner 无法与 Windows 发现/提权分离。 |
| `glyphshift-controller-windows` | Windows executable/elevation/remote injection owner 会污染跨平台 Host。 |
| `glyphshift-isolated-worker-host` | Worker/Supervisor/placement/restart 生命周期会泄漏到 Session。 |
| `glyphshift-runtime-kernel` | 纯决策执行 kernel 会被 native ABI 或产品 pool 拥有。 |
| `glyphshift-session` | 部署 Contract 与目标有状态 Manager 会散入每种 Host。 |
| `glyphshift-target-process-host` | process-family discovery/activation owner 会与 Target Runtime DLL 混合。 |
| `glyphshift-target-runtime` | `cdylib` 的 Native ABI、RuntimeState 与 capture owner 发布身份消失。 |
| `glyphshift-desktop-backend` | 产品持久模型/Dictionary/Workflow/Software 编排会与进程 Runtime 生命周期耦合。 |
| `glyphshift-desktop-runtime` | Bundle/Pool/Target 生命周期会进入 Tauri 壳或 backend storage。 |
| `glyphshift-service` | 无 UI 产品组合根/服务入口会被桌面壳替代。 |
| `glyphshift-desktop-shell` | Tauri/Vue/Windows shell 组合根无法与可测试产品 crate 分离。 |

### Test support

| Package | 删除测试 |
|---|---|
| `glyphshift-reference-adapters` | 多 Adapter SDK 的确定性第三方实现 fixture 消失。 |
| `glyphshift-test-controller-plugin` | 独立 Controller plugin process/stdio fixture 消失。 |
| `glyphshift-test-isolated-worker` | 独立 Worker process/transport fixture 消失。 |
| `glyphshift-windows-host` | GDI/GDI+/Direct2D/DirectWrite/UIA 合成像素与窗口 fixture 消失。 |
| `glyphshift-windows-runtime-target` | 独立目标 executable/process-family fixture 消失。 |

## Review checklist

- [x] 54 个 workspace package 恰好出现一次，未遗漏新增 GTK/Qt 与 Windows Runtime target。
- [x] 所有保留理由对应稳定 interface、crate type、进程/权限/ABI owner 或多调用者策略，而非“已有文件”。
- [x] README 导航与本页家族一致，且没有把物理家族描述为统一依赖层。
- [x] 删除测试仍证明每个小 crate 的独立 interface/部署 seam；Stage 7 只改变物理路径。

## Review

- Standards：审计表 54 行/54 个唯一 package，与 Cargo metadata 集合精确一致；能力家族和依赖层级的
  职责清楚，没有创建聚合 crate 或把测试制品并入 production。原物理结论由 Stage 7 纠正。
- Spec：Adapter 仍按文字技术和部署 companion 划分；Native ABI、Worker/Controller SDK、Target DLL、
  App/Test executable 的发布身份均不变。删除物理移动后，导航与可执行约束仍能独立交付。
