# Rust/Cargo crate 命名与 workspace 边界调研

> 调研日期：2026-08-06
> 范围：Rust/Cargo 官方资料与当前仓库的只读检查；本文不构成已批准的迁移方案。
> 标记：**官方规则**表示文档明确规定；**机制事实**表示工具或语言明确提供的语义；**工程推论**表示根据这些机制对 Glyphshift 作出的保守判断。

## 结论摘要

1. **Rust/Cargo 没有规定 workspace 内部 package 必须保留项目名前缀。** Rust API Guidelines 甚至将 crate 的大小写风格列为“unclear”，只明确建议不要使用 `-rs` / `-rust` 这类无信息量前后缀。因此，`glyphshift-` 是项目命名策略，不是 Rust 最佳实践的硬性要求。[Rust API Guidelines: Naming](https://rust-lang.github.io/api-guidelines/naming.html#casing-conforms-to-rfc-430-c-case)
2. **package 名、library crate 名、依赖在当前 consumer 中的名字、物理目录名是四个不同概念。** 可以保留发布/工具侧的 `glyphshift-runtime-contract`，把源码中的依赖名缩短成 `runtime_contract`，同时把目录整理为 `crates/runtime/contract/`；Cargo 官方机制允许三者不同。[Cargo manifest `name`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field)、[Cargo targets `name`](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-name-field)、[Renaming dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml)、[Workspaces members](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-members-and-exclude-fields)
3. **若 package 可能发布到 registry，保留 `glyphshift-` 更稳妥。** crates.io 使用全局、先到先得的名称；registry 还会做大小写及 `-`/`_` 归一冲突检查。通用名称 `protocol`、`session`、`runtime` 的身份表达和可用性都更弱。[Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html#before-your-first-publish)、[Registry index name restrictions](https://doc.rust-lang.org/cargo/reference/registry-index.html#name-restrictions)
4. **若 package 确定永不发布，先显式设置 `publish = false`，再讨论去前缀。** Cargo 官方把它定义为防止私有 package 被意外发布的明确手段。[Cargo manifest `publish`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-publish-field)
5. **不要因为名称长就合并 crate。** crate 是编译、链接、版本、分发和运行时加载的边界；文件/模块才是纯导航边界。ABI/cdylib、独立进程、跨进程协议、需要独立 feature/依赖集合或发布策略的组件，应优先保留独立边界。[Rust Reference: Crates and source files](https://doc.rust-lang.org/reference/crates-and-source-files.html)、[The Rust Book: Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)
6. **对当前四个候选，保守结论是“先分组和简化本地引用，不合并”。** `runtime-contract`、`runtime-kernel`、`protocol`、`session` 当前都有多个直接 consumer，且依赖方向不同。尤其不应把实现依赖并入 contract/protocol 的低层边界。

## 四层命名分别由什么控制

| 层次 | Cargo/Rust 中的含义 | 官方规则或默认值 | 对 Glyphshift 的意义 |
| --- | --- | --- | --- |
| package 名 | `[package].name`；用于依赖、`cargo -p` 等 package 选择，也是默认 target 名的来源 | 只允许字母数字、`-`、`_` 且非空；registry 可增加限制。[Manifest](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field) | `glyphshift-runtime-contract` 是 package 身份，不等同于源码必须写出的名字 |
| library crate 名 | `[lib].name`；Rust 代码引用的 crate 名 | 默认取 package 名，并把 `-` 转成 `_`；Rust crate 名不能含连字符。[Cargo targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library)、[Rust Reference: extern crate names](https://doc.rust-lang.org/reference/items/extern-crates.html#name-restrictions) | 当前默认得到 `glyphshift_runtime_contract`；可显式设置 `[lib].name`，但这是公开 target 名变更 |
| consumer 本地依赖名 | `[dependencies]` 中的 key；配合 `package = "..."` 指向真实 package | Cargo 明确支持依赖重命名，原因包括避免源码中的 `use foo as bar`、同时依赖多个版本或同名 package。[Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml) | 可让源码使用 `runtime_contract`，而 package 继续叫 `glyphshift-runtime-contract` |
| 目录名 | 包含 `Cargo.toml` 的 package root；workspace member 和 path dependency 使用目录路径 | workspace `members` 是包含 manifest 的目录路径，并支持任意相对路径和 glob；官方没有要求目录 basename 等于 package 名。[Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-members-and-exclude-fields)、[Package layout](https://doc.rust-lang.org/cargo/guide/project-layout.html) | `crates/runtime/contract/` 完全可以容纳 package `glyphshift-runtime-contract` |

补充的**机制事实**：Cargo package 可以有至多一个 library target 和任意多个 binary target；每个 target 都是一个 crate。[Cargo targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html)、[The Rust Book](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)

因此，“去掉前缀”至少有三种不同操作，不能混为一谈：

- 只缩短目录：改变物理导航和 path，不改变 package 或 Rust API 身份。
- 只给依赖取本地 alias：改变 consumer 源码中的 crate 路径，不改变真实 package 身份。
- 真正重命名 `[package].name`：改变 Cargo package 身份、默认 lib/bin target 名、`cargo -p` 选择名以及 metadata/lockfile 中的身份；若已经发布，registry 中等同于另一个 package。Package ID 由名称、版本和来源共同标识。[Package ID Specifications](https://doc.rust-lang.org/cargo/reference/pkgid-spec.html)

## 项目前缀：官方要求与工程判断

### 官方明确说了什么

- Rust API Guidelines 是建议而非强制规范；其 crate 命名表明确写的是 “unclear”，只明确反对无意义的 `-rs`/`-rust`。[Guidelines status](https://rust-lang.github.io/api-guidelines/)、[Naming](https://rust-lang.github.io/api-guidelines/naming.html#casing-conforms-to-rfc-430-c-case)
- package 名是依赖及默认 target 的标识；Cargo 没有命名空间语法来表达 `glyphshift::runtime-contract`。[Manifest](https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field)
- crates.io 名称按先到先得分配，发布后版本不可覆盖；registry 名称存在全局冲突约束。[Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)、[Registry index](https://doc.rust-lang.org/cargo/reference/registry-index.html#name-restrictions)
- 私有 package 可以用 `publish = false` 明确禁止发布。[Manifest `publish`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-publish-field)

### 从机制推导出的建议

以下不是 Rust 官方强制规则：

- **可能公开发布或作为独立 SDK/ABI 交付的 package：保留 `glyphshift-`。** Cargo 没有 registry 命名空间，前缀承担所有权、搜索和冲突规避作用。
- **严格仓库内部的 package：可以不带前缀，但应先 `publish = false`。** 即使如此，`cargo metadata`、构建日志、`cargo -p` 和依赖图中的 `protocol`/`session` 仍比 `glyphshift-protocol`/`glyphshift-session` 缺少上下文。
- workspace 可在 `[workspace.package]` 中集中提供 `publish`，再由成员继承；这适合统一声明一组内部 package 的发布策略。[Workspace package inheritance](https://doc.rust-lang.org/cargo/reference/workspaces.html#the-package-table)
- **推荐折中：package 身份保留前缀，物理目录按领域分组，必要时使用本地依赖 alias。** 这样能改善仓库导航与源码可读性，而不把“整理结构”升级成 package 身份迁移。
- **不要额外设置 `[lib].name` 只为去掉前缀，除非确实希望所有 consumer 都把它视为新的公开 crate 名。** 本地 dependency alias 更局部、迁移风险更小。
- **不建议把 library crate 直接命名为 `core`。** 标准库的 `core` 已由 extern prelude 自动加入，重复引入同名 crate 会产生名称冲突；物理目录叫 `core/` 则没有这一问题。[Rust Reference: extern prelude](https://doc.rust-lang.org/reference/names/preludes.html#extern-prelude)、[`E0259`](https://doc.rust-lang.org/error_codes/E0259.html)

示意（不是本次已执行变更）：

```toml
[dependencies.runtime_contract]
package = "glyphshift-runtime-contract"
path = "../runtime/contract"
```

这会让 consumer 使用 `runtime_contract::...`，真实 package 仍保持 `glyphshift-runtime-contract`。该能力来自 Cargo 的依赖重命名机制。[Cargo dependency renaming](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml)

## 何时合并，何时保留独立 crate

### 官方机制提供的边界

1. **编译、链接及产物边界。** Rust Reference 明确把 crate 定义为编译、链接、版本、分发和运行时加载的单位；一次编译处理一个 crate 并产生一个可执行文件或库。[Rust Reference](https://doc.rust-lang.org/reference/crates-and-source-files.html)
2. **模块是 crate 内部的组织与可见性边界。** 一个 crate 可以跨多个源文件；模块可控制公开/私有范围，不会独立编译。[The Rust Book: Modules](https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html)、[Rust By Example: Crates](https://doc.rust-lang.org/rust-by-example/crates.html)
3. **feature 属于 package。** 同一个 package 被多个 consumer 使用时，Cargo 通常取被启用 feature 的并集；feature 应当是可叠加的。官方在互斥 feature 无法避免时明确建议考虑拆成不同 package。[Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification)、[Mutually exclusive features](https://doc.rust-lang.org/cargo/reference/features.html#mutually-exclusive-features)
4. **workspace member 可以独立选择。** `cargo build/check/test -p <package>` 可针对某个 package，但 workspace 仍共享 lockfile 和默认 target 目录。[Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
5. **ABI/动态库是实际产物边界。** `cdylib` 产生供其他语言/系统加载的动态系统库；`rlib` 是供 rustc 使用的 Rust 库中间产物。[Rust Reference: Linkage](https://doc.rust-lang.org/reference/linkage.html)
6. **Rust ABI 本身没有稳定保证。** 对动态/跨语言边界应显式使用合适的外部 ABI，并约束数据布局；默认 Rust representation 除少数安全性要求外没有其他布局保证。[Rust Reference: ABI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi)、[Type layout](https://doc.rust-lang.org/reference/type-layout.html#representations)
7. **发布和 SemVer 以 package 为单位。** Cargo manifest 的 version、publish 和 registry metadata 都属于 package；workspace 可以共享值，但不取消 package 边界。[Manifest](https://doc.rust-lang.org/cargo/reference/manifest.html)、[SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)

### 保留独立 crate/package 的保守条件（工程推论）

- 需要产生独立 `bin`、`cdylib`、`staticlib` 或 proc-macro 产物。
- 是跨进程、跨动态库或跨语言的协议/ABI 合同，需要尽量小而稳定的依赖面。
- 不同 consumer 需要互斥 feature、不同平台依赖、不同 MSRV 或明显不同的编译依赖集合。
- 需要单独执行 `cargo test -p`、发布、版本化、审计、授权或安全审查。
- 有两个以上真实 consumer，且它的依赖方向比 consumer 更低、更稳定；合并会迫使低层合同依赖高层实现。
- 需要把 `unsafe`/FFI/系统 API 隔离在更小的审计范围内。

独立进程需要特别区分：**进程边界要求独立 binary crate，但不必然要求独立 package**，因为一个 package 可以有多个 binary target。若该进程还需要独立依赖集、feature、构建选择、部署/版本策略，才更有理由成为独立 package。[Cargo binaries](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)

另一个限制是：拆成不同 package 只让它们拥有各自的 feature 定义，并不保证共同第三方依赖永不发生 feature union；resolver 2 只避免部分 target/build/dev 场景的合并，需要彻底隔离时可能仍要分别执行 Cargo 构建。[Cargo resolver features](https://doc.rust-lang.org/cargo/reference/resolver.html#features)

### 更适合合并成一个 crate 内模块的条件（工程推论）

- 拆分只为了文件变短或目录好看，没有独立产物、依赖、feature、consumer 或发布边界。
- 两个 crate 总是一起修改、一起实例化，只有单一 consumer，公开 API 主要用于跨越人为制造的 crate 边界。
- 拆分造成大量双向转换、facade/re-export 或循环概念，而模块私有性已经足够表达边界。
- 独立 crate 没有可验证的隔离收益，只增加 manifest、依赖边和公开 API 维护成本。

是否合并不能由行数单独决定。Rust 官方首先建议用模块和多个文件组织增长中的代码，项目继续增长时才提取为单独 crate/workspace package。[The Rust Book: Managing Growing Projects](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)

## Glyphshift 候选的只读核对

以下为当前仓库事实，不是官方通则：

| package | 当前目标与规模 | 直接 consumer | 依赖/职责观察 | 保守建议 |
| --- | --- | ---: | --- | --- |
| `glyphshift-runtime-contract` | 单一 lib，约 351 行 | 11 | 定义 `RuntimePublication`、identity、JSON wire 编解码；被 desktop、service、session、controller、worker、target runtime 等共同依赖 | **保留独立。** 这是高扇出的序列化合同；不应并入 kernel 或任一 host。可移到 `crates/runtime/contract/`，package 名暂留 |
| `glyphshift-runtime-kernel` | 单一 lib，约 259 行 | 2 | 依赖 runtime contract、adapter registry、decision、domain、translation；被 service 与 target runtime 使用 | **保留独立。** 合并进 contract 会让合同层引入实现依赖；合并进 target runtime 会丢失 service 的共享实现。可移到 `crates/runtime/kernel/` |
| `glyphshift-protocol` | 单一 lib，约 1115 行 | 5 | controller/连接模型及 transport trait；被 controller host、desktop runtime、isolated worker host、service、target process host 使用 | **保留独立。** 多 host 共享的协议面比某个进程实现更稳定；可按领域移到 `crates/controller/protocol/` 或 `crates/protocol/` |
| `glyphshift-session` | 单一 lib，约 1134 行，内部已有 `contract`/`manager` 模块 | 4 | 依赖 adapter registry、domain、runtime contract；被 desktop runtime、isolated worker host、service、target process host 使用 | **目前保留独立，不与 protocol 合并。** 两者依赖方向和职责不同。只有在 consumer 使用面证明 contract 与 manager 必须独立编译时，才考虑进一步拆分 |

当前四个 package 都继承 workspace `0.1.0` 版本，且 manifest 未设置 `publish = false`。因此，不能在架构决策中直接假设它们是“永不发布的内部包”。如果项目意图确实如此，先统一声明禁止发布，价值高于立即改成通用 package 名。

## 推荐的低风险次序

1. **先确定发布策略。** 对纯内部 packages 显式使用 `publish = false`；对 SDK、ABI、可复用协议决定是否保留潜在发布能力。
2. **先改物理信息架构。** 例如 `crates/runtime/{contract,kernel}`、`crates/controller/{protocol,...}`；目录改名不要求 package 改名。
3. **按实际可读性引入 dependency alias。** 只在 consumer 端缩短 `glyphshift_runtime_contract` 等路径，并保持 alias 具备领域信息；避免把所有东西都泛化成 `runtime`、`core`、`common`。
4. **最后才评估 package 身份重命名。** 此时要单独审计 `cargo -p`、workspace dependency keys、scripts、metadata consumers、默认 lib/bin artifact 名及任何外部分发约定。
5. **合并必须用边界证据驱动。** 至少核对直接 consumer、feature/平台矩阵、产物类型、依赖方向、独立测试与发布需求；不要把“前缀长”当成合并理由。

## 最终建议

对 Glyphshift，当前最稳妥且最符合 Cargo 机制的方案是：

- 继续用 `glyphshift-*` 作为 package 身份，至少在发布策略尚未明确前如此；
- 采用领域化短目录，不要求目录 basename 重复完整 package 名；
- 对源码可读性确有帮助时使用明确的本地 alias，例如 `runtime_contract`、`runtime_kernel`、`controller_protocol`，而不是过度通用的 `runtime`、`protocol`；
- 保留 `runtime-contract`、`runtime-kernel`、`protocol`、`session` 的独立 crate 边界；当前证据不支持互相合并；
- 后续若要减少 crate 数量，优先审计“单一 consumer、无独立产物、无独立依赖/feature/发布边界”的 implementation crates，而不是高扇出的合同与协议 crates。

这不是“Rust 官方要求项目名前缀”，而是利用 Cargo 把**全局/工具身份、源码局部名字和物理导航解耦**后的保守设计。
