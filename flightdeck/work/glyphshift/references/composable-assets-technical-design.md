# 可组合字典、字体方案与 Adapter Catalog 技术方案

Status: Implemented
Date: 2026-08-02

## 1. 结论

Glyphshift 的持久产品模型改为六个正交事实：

```text
Platform Facts
Adapter Catalog
Software
Dictionary
Font Profile
Workflow Target Composition
```

Dictionary 只承载翻译内容和便携元数据；Font Profile 只承载字体候选；Adapter Catalog
只描述可执行机制、兼容性和展示信息；Workflow Target 是唯一组合根。运行时继续消费编译后的
`TranslationSnapshot`、`FontPolicy` 和 Adapter Requirements，不理解产品资产。

当前产品尚未发布。本次直接替换持久化 schema 和公开产品 View，不读取、迁移或双写
`glyphshift.dictionary/1`、`glyphshift.workflow/1`，也不保留旧字段的反序列化默认值。

## 2. 最后一轮审查

### 2.1 主流做法的一致部分

- gettext、Fluent、Android String Resources 与 Unity String Tables 都把翻译内容作为独立
  资源；字体资产和渲染策略由宿主、样式或独立 Asset 管理。
- XUnity.AutoTranslator 也分别保存翻译文件、Text Framework 开关、字体覆盖和 UI resizing；
  它们不属于逐条翻译的固有字段。
- WinGet、OCI、VS Code Extension 与 npm 均区分包身份/版本、机器兼容信息、展示信息和实际
  下载来源/内容完整性。下载 URL、digest、签名和安装状态不应写进被校验的内容正文。

因此“平台、技术、Adapter、字典、字体分别独立，再由工作流组合”成立。保留的限制是：
Dictionary 可以拥有上下文和未来受控语义 annotation，但不得通过 metadata 重新夹带字体、
Hook、Adapter、平台、脚本或运行权限。

### 2.2 当前实现的偏差

- `Dictionary`、`DictionaryArtifact` 和 Vue detail 同时保存 Hook scope、默认字体、逐词条字体
  与逐词条 Adapter IDs。
- `WorkflowTarget` 只有 Dictionary IDs；Adapter 来自 Software Extension/Runtime Bundle 的
  隐式全部选择，字体来自 Dictionary 编译。
- Runtime Bundle 只有手写 `label`，没有独立 platform、technology、technical target、说明
  和配置事实。
- Native Adapter Descriptor 有架构但没有 OS；Registry 只检查架构。
- Dictionary schema 已有 `revision`，但没有清晰区分格式版本、本地编辑 revision 与在线发布
  release version。

底层的 `TranslationSnapshot` 与 `FontPolicy` 已正交，现有 Runtime Publication seam 可以保留。

## 3. 领域模型

### 3.1 Platform

Platform 是目标和 Adapter 的机器兼容事实，例如 `windows`、`macos`。它不是显示名称前缀，
也不由用户在 Dictionary 中选择。Target Facts 与 Adapter Descriptor 都声明 Platform，Registry
在产出 Binding 前检查 Platform、Architecture、ABI 和 Feature。

### 3.2 Technology

Technology 是 Catalog 的分类维度，例如 `gdi`、`gdiplus`、`core-text`、`accessibility`。
一个 Technology 可以拥有多个 Adapter；Technology 本身不可激活。首版每个 Adapter 声明一个
主要 Technology，展示模型预留 `tags`，不提前实现任意多轴分类系统。

### 3.3 Capability Adapter

Adapter 是具体可执行实现，例如 `ExtTextOutW`、`GdipDrawString`。用户选择的是多个 Adapter，
不是一个模糊 Hook Type。首版 Adapter Plan 只支持 `parallel`；未来 fallback、互斥和替代关系
通过新 Workflow schema/plan strategy 增加，不在 v1 配置里预埋无语义字段。

### 3.4 Dictionary

Dictionary 是可独立编辑、安装、发布和复用的纯翻译资产。它只拥有非空、按 source 唯一的
`source + translation` 词条和便携元数据，不拥有 keep、Location、Context、字体、Platform、
Technology、Adapter 或 Hook。

现有 Adapter 无法从同一 Hook 稳定识别主界面、弹窗或面板，因此不能把猜测的 Location 写入
Dictionary。未来只有在 Runtime 提供真实区域信号时，Workflow Target 才通过独立 Region Binding
把区域选择器与 Dictionary 组合；Dictionary payload 保持不变。

### 3.5 Font Profile

Font Profile 是可独立编辑和复用的字体方案。首版保存有序 family candidates；Workflow 编译时
从当前机器 Font Catalog 选择第一个可用 family。这样同一 Profile 可以同时包含跨平台候选，
Runtime 仍只接收一个已解析的具体 family，Native ABI 不需要知道 fallback。

Font Profile 不保存 Location。`FontProfileBinding` 在 Workflow Target 中声明它适用于全部
Location 或明确 Location 集合；同一 Target 的 Font Bindings 不得覆盖同一 Location。

### 3.6 Workflow Target

Workflow Target 是唯一组合根：

```text
WorkflowTarget
  softwareId
  adapterPlan
    strategy: parallel
    adapterIds[]
  dictionaryIds[]             # 有序，高优先级在前
  fontBindings[]
    fontProfileId
    scope: all | locations[]
```

Target 只要能编译出 Text Replace 或 Font Substitute 之一即可；因此允许纯字体 Target，也允许
没有字体的纯翻译 Target。空 Adapter Plan、没有任何有效文字/字体行为、未知引用、未知 Location、
无可用字体或 Adapter Feature 覆盖不足都在启用前拒绝，不产生半成品 Intent。

## 4. 版本化持久化

### 4.1 Dictionary v2

```json
{
  "schema": "glyphshift.dictionary/2",
  "revision": 1,
  "metadata": {
    "id": "dictionary.menu.zh-cn",
    "releaseVersion": "0.1.0",
    "name": "菜单简体中文",
    "description": "菜单文字翻译",
    "sourceLocale": "en",
    "targetLocale": "zh-CN",
    "authors": [],
    "license": null,
    "homepage": null,
    "tags": ["menu"]
  },
  "entries": [
    {
      "location": "menu",
      "context": null,
      "source": "File",
      "text": { "kind": "replace", "text": "文件" }
    }
  ]
}
```

- `schema` 是格式版本；破坏性结构变化必须提升。
- `metadata.releaseVersion` 是可发布内容版本，使用 SemVer 字符串。
- `revision` 是本地 CAS revision，不等于发布版本。
- `authors/license/homepage/tags` 为在线目录预留的便携元数据。
- 下载 URL、registry ID、digest、size、签名、publisher trust、installedAt 和 update channel
  属于未来 Catalog/Installation Record，不进入 payload。
- metadata 首版采用明确白名单；未来扩展必须是受控字段或 namespaced extension，不能接受会
  影响 Runtime 的任意 JSON。

### 4.2 Font Profile v1

```json
{
  "schema": "glyphshift.font-profile/1",
  "revision": 1,
  "metadata": {
    "id": "font-profile.cjk-ui",
    "name": "简体中文界面",
    "description": "跨平台中文 UI 字体候选"
  },
  "families": ["Noto Sans CJK SC", "Microsoft YaHei UI", "PingFang SC"]
}
```

families 非空、去重且顺序有意义。首版不声称修改字号、字重或样式；现有 GDI/GDI+ Adapter
继续保留宿主的 size/style/unit，只替换 family。

### 4.3 Workflow v2

```json
{
  "schema": "glyphshift.workflow/2",
  "id": "workflow.main",
  "name": "主工作流",
  "description": "",
  "revision": 1,
  "targets": [
    {
      "softwareId": "software.editor",
      "adapterPlan": {
        "strategy": "parallel",
        "adapterIds": ["windows.gdi.ext-text-out", "windows.gdiplus.draw-string"]
      },
      "dictionaryIds": ["dictionary.menu.zh-cn"],
      "fontBindings": [
        { "fontProfileId": "font-profile.cjk-ui", "scope": { "kind": "all" } }
      ]
    }
  ]
}
```

首版不实现 schema 兼容或自动迁移。持久文件读到其他 schema 时明确拒绝。

### 4.4 Target Runtime Deployment v2

`glyphshift.target-runtime/2` 在进程间传输完整 Adapter Descriptor，包括 Platform、Architecture、
ABI、Feature、Apply Model 与 Placement。Platform 不能只留在桌面 Catalog；注入目标内的 Native
Host 也必须用同一份事实核对动态库描述符，防止 UI 可选但 Runtime 因证据丢失而拒绝加载。

## 5. Adapter Catalog

### 5.1 Machine Descriptor

Native Descriptor 增加 platform bits；Registry 增加 Platform mismatch 拒绝，并补充 arm64
architecture bit。ID、version、ABI、Apply Model、Placement、Feature、Platform、Architecture
是执行事实。

### 5.2 签名包 Presentation

Runtime Bundle 的每个 Adapter artifact 增加签名 presentation：

```text
name                 ExtTextOutW
summary              拦截 GDI ExtTextOutW 绘制并执行文字/字体决策
technology.id        gdi
technology.label     GDI
technicalTarget      gdi32.dll!ExtTextOutW
configuration.kind   none
```

Platform 来自 machine descriptor；其本地化 label 由宿主 Catalog 提供。Presentation 只能影响
展示，不能改变 machine descriptor 的 Feature、Platform、ABI 或入口。

UI 独立显示 Platform、Technology、Adapter name 和 Capability，不再拼接“Windows GDI”前缀，
也不再暴露“Hook 类型”。

### 5.3 Adapter Plan 解析

DesktopBackend 在启动时接收当前 Runtime Bundle 投影出的只读 Composition Environment：

```text
available Adapter requirements + features
available font families
```

Backend 负责验证持久组合；RuntimeBundle/Registry 仍负责最终签名、hash、Platform、Architecture、
ABI 和 Feature Binding。Workflow 中只保存稳定 Adapter IDs，不保存本机路径、版本解析结果或
artifact hash。

## 6. 编译与运行

公开组合 seam 深化为：

```text
resolve(workflow, software, dictionaries, fontProfiles, environment)
  -> CompiledWorkflow
```

内部顺序：

1. 验证 Software、Dictionary、Font Profile、Adapter 引用。
2. 按有序 Dictionary IDs 编译 Translation Snapshot；冲突由高优先级 Dictionary 获胜并诊断。
3. 解析 Font Bindings，验证 Location 不重叠，从 Font Catalog 选择每个 Profile 的首个可用 family，
   编译 Font Policy。
4. 从实际内容推导 Text Replace / Font Substitute。
5. 验证所选 Adapter 的 Feature 并为每个 Adapter 生成所需 Feature 的 Requirement。
6. 原子产出 Target Intent；任一步失败不修改 activation 或最后有效 Runtime Publication。

Adapter ID 不再进入 Translation Snapshot 或 Font Policy 的查找键。一次 Observation 仍携带
Adapter ID 供状态、诊断和仲裁使用，但文字/字体规则只按 Location、Context、Source 决策。

## 7. Desktop 产品表面

主导航调整为：

```text
工作流 / 软件 / 词典 / 字体
```

- Dictionary Library：名称、说明、源语言、目标语言、发布版本、规则数、本地 revision、标签。
- Dictionary Editor：Metadata 区域与纯文字规则表；删除 Hook、默认字体、字体处理和字体列。
- Font Library：名称、说明、family candidates、当前机器命中结果、引用状态。
- Workflow Editor：每个 Software Target 独立维护 Adapter 多选、可显式上下调整的 Dictionary
  有序栈和多条 Font Bindings；每条字体绑定独立选择 all/locations scope，重叠位置在保存前拒绝。
  Platform/Technology 只用于 Adapter 分组和说明。
- Software 页继续只管理身份与程序绑定。
- 普通界面不显示内部 Adapter ID、DLL 路径、hash 或签名细节。

## 8. 在线 Dictionary 预留

当前只实现本地 Dictionary payload 和 metadata。未来在线功能在独立 seam 接入：

```text
DictionaryCatalogPort
  list/query metadata
  resolve release
  fetch artifact descriptor

DictionaryArtifactDescriptor
  mediaType
  size
  digest
  downloadUrl
  publisherIdentity
  signature/provenance
```

下载完成后先验证 descriptor/digest/signature，再把已验证 payload 交给现有 Dictionary validator。
Runtime、Decision Engine、Native Adapter 和 Dictionary hot path 不访问网络。

## 9. 测试 seam 与验收

只从四个现有公开 seam 测试：

1. `glyphshift-workflow::resolve`：纯字典、纯字体、组合、多 Adapter、字体 fallback、冲突和拒绝。
2. `DesktopBackend`：Dictionary v2、Font Profile v1、Workflow v2 的 CRUD、CAS、引用保护、重开
   和无兼容拒绝。
3. Tauri 产品命令/View：Catalog、字体、字典 metadata 和 Workflow Target 组合序列化。
4. Playwright：四个管理页、逐 Target 组合、排序、多选、错误与 960×640/1440×900。

另外保留 workspace test、Clippy、fmt、前端生产构建、架构扫描和本地隐私扫描。真实运行证据仍
只写 `local-test/evidence/`。

## 10. 非目标

- 不实现旧 Dictionary/Workflow schema 的兼容、迁移或双写。
- 不实现在线目录、账号、发布、下载、自动更新或签名服务。
- 不开放任意 DLL、函数、地址、pattern 或 Frida 脚本配置。
- 不实现 macOS Adapter；只完成 Platform/arm64/Catalog seam。
- 不让 Dictionary metadata 影响 Runtime 权限、Adapter 选择或字体解析。
- 不把 Font Profile 打包或复制系统字体文件；首版只引用用户机器上已安装的 family。
