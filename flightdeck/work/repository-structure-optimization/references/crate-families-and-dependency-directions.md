# Crate 家族与依赖方向

Status: Complete

## 结论

仓库结构使用两套互补分类：

1. **物理家族**用于目录导航，回答“这个 package 属于哪类能力”。
2. **依赖层级**用于架构约束，回答“这个 package 可以知道谁”。

二者不能合并成一棵树。具体 Adapter、Adapter SDK、Native ABI 与动态 Host 虽然属于同一业务家族，
却处于不同部署 seam 和依赖层；`core` 也只能作为物理目录名，不能成为一个新 crate 或允许任意反向
依赖的理由。

## 物理家族

下列分类记录 Stage 8 完成后的实际目录。47 个 `crates/` package 均使用家族入口和短叶目录；crate 名、
ABI 与 Cargo package 名保持不变，所有 package 当前继承内部发布策略 `publish = false`。

### `crates/core/`（已落实）

领域事实与可复用策略：

- `glyphshift-domain`
- `glyphshift-translation`
- `glyphshift-capture`
- `glyphshift-decision`
- `glyphshift-workflow`
- `glyphshift-extension`

`core` 不是依赖层。这里的 policy crate 可以依赖 Adapter Registry 等低层合同，但不得依赖 Host、
Desktop 或具体 Adapter 实现。

### `crates/dictionary/`（已落实）

独立语言资产及其分发：

- `glyphshift-dictionary-package`
- `glyphshift-dictionary-distribution`

### `crates/adapters/platform/`（已落实）

所有 Adapter 共用的描述、解析与 Native 加载 seam：

- `glyphshift-adapter-sdk`
- `glyphshift-adapter-registry`
- `glyphshift-adapter-native-abi`
- `glyphshift-adapter-native-host`

### `crates/adapters/implementations/`（已落实）

23 个具体 capability package 与部署 companion 使用第二级导航分组：

- `native/`：Console、Direct2D、DirectWrite、Win32 DrawText/GDI 与 GDI+，共 13 个 crate。
- `framework/`：GTK3/Pango、Qt Painter 与 raylib，共 6 个 crate。
- `accessibility/`：UI Automation descriptor 与 Worker，共 2 个 crate。
- `fallback/`：OCR descriptor 与 Worker，共 2 个 crate。

每个叶目录仍使用技术短名，例如 `native/gdi`、`native/gdi-native`、
`accessibility/uia-worker`。二级分组仅用于物理导航，不是依赖层或新的运行时 seam。

一个技术可拥有 descriptor、Native DLL 或 Worker companion，但不按软件品牌创建 Adapter。

### `crates/runtime/`（已落实）

跨进程合同、Controller、Worker、Session 与 Target Runtime 编排：

- 公共编排：`acquisition/`、`protocol/`、`contract/`、`kernel/`、`session/`、`desktop/`
- Controller：`controller/{sdk,host,windows}`
- Worker：`worker/{acquisition-sdk,acquisition-host,process-grant,sdk,host}`
- Target：`targets/{contract,process-host,runtime}`；复数目录避免命中 Cargo/仓库的 `target/` 构建输出忽略规则

### 产品与测试（保持原位）

- `crates/product/desktop-backend` 是产品内部 crate。
- `apps/glyphshift-desktop` 与 `apps/glyphshift-service` 保持应用组合根，不移入 `crates/`。
- 五个 `test-support/` package 保持独立；生产 package 禁止依赖它们。

分类当前覆盖 54 个 `crates/` package、2 个 App package 与 6 个 test-support package，共 62 个 workspace
package；所有 `crates/` package 均有真实物理家族入口。

## 依赖层级

箭头方向统一写作“调用者 → 被依赖者”。同层依赖只在现有精确依赖合同允许时成立。

| 层级 | package | 允许依赖 |
|---|---|---|
| L0 基础模型/SDK | Domain、Translation、Capture、Dictionary Package、Adapter SDK、Controller SDK、Worker SDK | 外部库及更基础的同层模型 |
| L1 策略/目录 | Adapter Registry、Decision、Workflow、Extension、Dictionary Distribution、Runtime Contract | L0 与经审计的 L1 |
| L2 部署/线协议合同 | Native ABI、Protocol、Target Runtime Contract | L0-L1 与合同同层 |
| Adapter 实现侧车 | 23 个具体 Adapter/Native/Worker package | L0-L2 中对应 SDK/ABI/合同及本技术 companion |
| L3 Host/Kernel | Native Host、Controller Host/Windows、Worker Host、Runtime Kernel、Session、Target Process Host、Target Runtime | L0-L2、Adapter Platform；运行时发现具体 Adapter，不静态依赖产品 |
| L4 产品编排 | Desktop Backend、Desktop Runtime、Service | L0-L3；不得依赖 Shell 或测试支持 |
| L5 产品 Shell | Desktop Shell | L0-L4；只负责用户/平台入口和组合 |
| Test | `test-support/` 与合同测试 | 可依赖生产 interface；生产代码不得反向依赖 Test |

```text
L5 Shell
   ↓
L4 Product orchestration
   ↓
L3 Hosts / Kernel
   ↓
L2 Deployment contracts
   ↓
L1 Policy / Catalog
   ↓
L0 Foundation / SDK

Concrete Adapters ──→ L0-L2
Tests             ──→ production interfaces
```

## 禁止方向

- Domain、Translation、Capture、Dictionary 与 policy crate 不依赖 Host、Desktop 或具体 Adapter。
- 具体 Adapter 不依赖 Desktop、Service、Session、Controller Host 或 Target Runtime 编排。
- Runtime contract/SDK 不依赖对应 Host 实现。
- Host/Kernel 不依赖 Desktop Shell、Desktop Backend 或 test-support。
- Desktop Backend 和 Desktop Runtime 不依赖 Tauri/Vue Shell。
- 任意生产 package 不依赖 `test-support/`。
- 不创建 `glyphshift-core`、`common`、`utils` 等无单一变化原因的聚合 crate。

## 落地顺序

1. 先完成超长实现体的 crate 内 module 拆分，验证真实 seam。
2. Stage 7 先移动 Adapter 物理目录，Stage 8 经独立审计后完成全部家族短路径；均未改 package 名、Rust
   crate 名、ABI 或公开 interface。
3. 移动同步 Cargo member/path、脚本、架构扫描路径与文档链接，并按切片验证。
4. 在 `architecture-tests/check.ps1` 保留逐 package 精确依赖、上述层级的禁止方向与内部发布策略。
5. package 合并与 identity 改名仍须独立证据，不能从物理目录归档机械外推。

## 自审

- Standards：54 个 workspace package 均恰好归入一个能力家族；短目录没有把部署 seam 合并，也没有新增
  production/test 反向依赖。
- Spec：分类保持 Adapter 按文字技术划分、Native/Worker 真实进程边界、现有 package 名与 ABI 不变。
- 删除测试：移除任一家族规则后，目录移动与新增依赖都失去可审计依据；该文档不是对现有 Cargo 精确
  依赖检查的重复，而是它缺少的跨 package 方向语义。
