# 字典、字体与 Adapter 可组合模型最终复核

Status: Complete
Date: 2026-08-02

## 1. 结论

对 GNU gettext、Mozilla Fluent、Android、Unity Localization、XUnity.AutoTranslator、
WinGet、VS Code Extension、npm 和 OCI 的一手资料复核后，Glyphshift 当前提出的方向没有
结构性问题：

1. **Dictionary、Font Profile、Platform、Technology、Adapter、Workflow 应是独立模型。**
   它们之间的关系放在 Workflow Binding，而不是互相嵌入 ID。
2. **Dictionary 应保持执行无关，但不必退化成两个裸字符串。**翻译上下文、译者注释、
   复数/变体、占位符、来源引用和受控的语义 annotation 都属于语言资产；字体、Hook、
   Adapter、平台、架构和运行权限不属于语言资产。
3. **Technology 是分类轴，Adapter 才是执行与选择单元。**一个 Adapter 可以拥有多个技术
   标签，一个 Workflow 可以启用多个 Adapter；技术标签本身不激活任何代码。
4. **在线 Dictionary 应采用“纯 payload + 外部 Artifact Descriptor”。**字典正文保存
   schema、发布元数据和 entries；Catalog 在正文之外保存 digest、size、下载地址、发布者
   身份和签名/来源证明。摘要不能放进它自己所摘要的正文中。
5. **无需设计旧 schema 兼容。**首个正式结构直接使用 `schemaVersion: 1`；加载器只接受
   当前已知版本，后续格式演进再提升 schema version。内容发布版本使用独立的
   `releaseVersion`，不能和 schema version 混用。

本轮复核带来两个需要明确写进技术方案的修正：

- “字典独立”指不携带执行策略，不等于禁止语言学 metadata。
- “Adapter 自描述”需要同时防止 presentation 字段影响执行；Registry 只能信任强类型的
  machine descriptor，名称和说明只能进入 Catalog/UI 投影。

## 2. 主流翻译资源如何处理文字、字体与渲染

### 2.1 证据矩阵

| 体系 | 核心翻译资产 | 字体/渲染位置 | 对 Glyphshift 的含义 |
|---|---|---|---|
| GNU gettext | PO entry 由 `msgid`、`msgstr`、可选 `msgctxt`、注释、来源引用和 flags 等组成 | PO 格式没有字体或绘制入口字段 | Dictionary 可以有上下文和翻译工作流信息，但不应有 Font/Adapter |
| Mozilla Fluent | Message、Term、variant、attribute 和可插值 pattern | 宿主 DOM/UI 负责具体样式；DOM Overlay 只允许受控的语义结构 | “语义 annotation”可以存在，但不能借此注入任意样式或执行数据 |
| Android Resources | string、string-array、plurals 等保存在 values 资源 | 字体是 `res/font` 资源，由 layout/style/`fontFamily` 组合 | 文字资源和字体资产是不同资源类型，组合发生在 UI 层 |
| Unity Localization | String Table 与 Asset Table 是不同集合 | 字体可作为 Asset 单独本地化；Metadata 也可扩展 | 分离 String/Asset 值得借鉴，但 Glyphshift 必须约束 metadata 边界 |
| XUnity.AutoTranslator | 翻译文本文件保存文字映射 | `OverrideFont`、TMP fallback 和 UI resizing 位于独立配置 | 即使同一 Runtime 同时做翻译和字体替换，配置也不必进入翻译 entry |

[GNU gettext 的 PO File Entries](https://www.gnu.org/software/gettext/manual/html_node/PO-File-Entries.html)
把一个 entry 定义为原文与译文之间的关系，并包含 translator comments、自动注释、来源
reference、flags、旧原文、`msgid` 与 `msgstr`。这证明“语言资产”可以比 key/value 更丰富，
但没有理由携带字体或绘制 API。

[Fluent Writing Text](https://projectfluent.org/fluent/guide/text.html)说明 Message、Term、variant
和 attribute 的值都是文本 pattern；[Fluent Attributes](https://projectfluent.org/fluent/guide/attributes.html)
用于把 placeholder、ARIA label、title 等同一组件上的可翻译文字组织为一个翻译单元。
[DOM Overlays](https://projectfluent.org/dom-l10n-documentation/dom-overlays.html)允许受控的语义
标记，但实际 DOM 和视觉样式仍由宿主控制。因此 Glyphshift 可以保留 context、placeholder
约束和语义标签，但不能把 `fontFamily`、DLL 或 Hook 参数伪装成“通用 metadata”。

Android 的 [String resources](https://developer.android.com/guide/topics/resources/string-resource)
和 [Font resources](https://developer.android.com/guide/topics/resources/font-resource)是两个不同
资源类型：字符串位于 values 资源，字体文件/字体族位于 `res/font`，再由布局或 style 应用。
[Styles and themes](https://developer.android.com/develop/ui/views/theming/themes)进一步把字号、颜色、
字体等视觉属性放进 style/theme。物理文件是否同目录不是重点，资源所有权和组合层才是重点。

Unity 官方将文字放在 String Table，将图片、音频等非文本资产放在 Asset Table；
[Unity Localization 指南](https://docs.unity3d.com/Manual/best-practice-guides/ui-toolkit-for-advanced-unity-developers/localization.html)
明确让文本属性绑定 String Table、非文本属性绑定 Asset Table。需要注意，Unity 的
[Metadata](https://docs.unity3d.com/Packages/com.unity.localization@1.5/manual/Metadata.html)
扩展能力非常宽，甚至允许嵌入字体数据、Locale 特定组件值和自定义代码。它不是 Glyphshift
应该照搬的安全边界，反而证明 metadata 必须白名单化或命名空间化。

[XUnity.AutoTranslator 官方配置](https://github.com/bbepis/XUnity.AutoTranslator#configuration)
把翻译文件目录、Text Framework 开关、`OverrideFont`、`FallbackFontTextMeshPro` 和 UI
resizing 分开。其部分 resizer/scoping 文件虽然与翻译文件共用目录，但仍是不同格式和加载
路径。这支持“概念独立、运行时组合”，而不是要求所有内容必须位于同一个配置对象。

### 2.2 Glyphshift Dictionary 允许和禁止的内容

允许：

- source、translation、stable entry ID；
- source/target locale；
- translator note、source reference、review state；
- disambiguation context、placeholder contract、plural/variant 等语言学结构；
- 受控 tags 和 namespaced semantic annotations。

禁止：

- `hookTypeId`、`adapterIds`、DLL、函数、地址或脚本入口；
- platform、architecture、ABI 和运行权限；
- font family、fallback、字号、字重和字体文件路径；
- Adapter 激活参数、Feature 开关和 Runtime 状态。

这里的“禁止”同时适用于 Dictionary 顶层 metadata 和单条 entry metadata，避免通过扩展区
重新制造已经拆掉的耦合。

## 3. 在线分发的主流做法

### 3.1 身份、格式版本和发布版本是不同概念

[npm package.json](https://docs.npmjs.com/cli/v11/configuring-npm/package-json/)要求发布包使用
name + version 形成唯一身份，并要求 version 可由 SemVer 解析；repository、author、license、
homepage 和兼容性字段各有独立语义。[VS Code Extension Manifest](https://code.visualstudio.com/api/references/extension-manifest)
同样区分 publisher/name/version、`engines.vscode`、entrypoint 和 Marketplace 展示字段。

Glyphshift 因此应区分：

| 字段 | 含义 | 变化时机 |
|---|---|---|
| `schemaVersion` | JSON/包结构的解析契约 | 字段结构或语义发生不兼容变化 |
| `dictionaryId` | 字典的稳定、全局唯一身份 | 不随发布变化 |
| `releaseVersion` | 字典内容的一次发布版本 | entries 或发布元数据变化 |
| `sourceRevision` | 内容来源的上游 revision | 上游源发生变化 |

首版不需要兼容旧 schema；直接定义 `schemaVersion: 1`。Schema version 建议使用整数，不用
SemVer。Dictionary release version 使用 SemVer，便于 Catalog 比较更新。

### 3.2 展示与机器事实分离

[WinGet Manifest](https://learn.microsoft.com/en-us/windows/package-manager/package/manifest)
明确说明，为了分离 installer 校验与本地化 metadata，多文件 manifest 应拆成 version、
defaultLocale、locale 和 installer 文件。Installer 侧保存 architecture、platform、最低系统
版本、URL 和 SHA-256；locale 侧保存 publisher、package name、description、tags、homepage
和 license。

VS Code 采用相同原则：`engines`、`main`、`browser` 等字段决定运行兼容性，displayName、
description、category 和 icon 用于展示；`package.nls.json` / `package.nls.<locale>.json`
外置本地化字符串，不改变执行描述。
[VS Code package.nls 说明](https://code.visualstudio.com/updates/v1_72#_the-packagenlsjson-file)
和[平台特定扩展](https://code.visualstudio.com/api/working-with-extensions/publishing-extension#platform-specific-extensions)
分别证明展示本地化与平台 artifact 选择是两个独立维度。

Glyphshift 不必在首版物理拆成多个文件，但逻辑上必须形成三种对象：

```text
Machine Descriptor
  执行身份、平台、架构、ABI、能力、入口和配置 schema

Presentation
  locale、name、summary、description、tags 和 documentation

Artifact Descriptor
  mediaType、size、digest、download URL、publisher identity、signature/provenance
```

Registry resolve 只读取 Machine Descriptor 和受信 Artifact Descriptor；Presentation 只能进入
Catalog/UI，不能改变 Feature、兼容性、入口或权限。

### 3.3 完整性和来源应位于 payload 外部

[OCI Content Descriptor](https://github.com/opencontainers/image-spec/blob/main/descriptor.md)
使用 `mediaType`、`digest`、`size`、`urls` 和 annotations 描述外部内容，并要求消费不可信
来源时按照 digest 验证。它的 [Image Index](https://github.com/opencontainers/image-spec/blob/main/image-index.md)
在 artifact descriptor 上声明 OS、architecture 和 OS version；
[OCI Annotations](https://github.com/opencontainers/image-spec/blob/main/annotations.md)则定义 created、
authors、source、version、revision、vendor、licenses、title、description 和 documentation 等
来源/展示字段。

[npm package-lock](https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/)也在解析结果中
分开保存 `resolved` 和 `integrity`。npm Registry 签名绑定 package name、version 和 tarball
integrity；[npm registry signatures](https://docs.npmjs.com/about-registry-signatures/)说明这用于
发现镜像或代理篡改。[npm provenance](https://docs.npmjs.com/generating-provenance-statements/)
再提供源码、构建环境与发布者的可验证关联。

因此在线 Dictionary 推荐两层：

```text
CatalogRelease
  dictionaryId
  releaseVersion
  artifact: { mediaType, size, digest, downloadUrl }
  publisherIdentity
  signature / provenance
  publishedAt

DictionaryPayload
  schemaVersion
  metadata
  entries
```

`digest` 不进入 `DictionaryPayload`，否则会产生自哈希问题。包内自报的 publisher、digest 或
下载地址也不能覆盖 Catalog 的权威值。安装时间、本机路径、启用状态和更新通道属于本地安装
数据库，不写回 Dictionary。

## 4. Adapter Catalog 的最终边界

### 4.1 Platform、Technology 与 Adapter 是三个轴

- **Platform**：`windows`、`macos` 等目标环境事实，用于 Registry compatibility resolve。
- **Technology**：`gdi`、`gdiplus`、`directwrite`、`accessibility` 等受控分类标签，用于过滤、
  分组和说明；一个 Adapter 可以声明多个标签。
- **Adapter**：具体可加载、可协商 Feature、可激活的执行实现，是 Workflow 的选择单元。

Adapter ID 应全局稳定且不依赖本地化名称，也不必带平台前缀。例如内建实现可使用
`glyphshift.ext-text-out` 和 `glyphshift.gdip-draw-string`；显示名分别是 `ExtTextOutW` 与
`GdipDrawString`，UI 在独立区域展示 Platform 和 Technology。

Technology 可以在筛选器中多选，Adapter 也可以在 Workflow 中多选，但二者含义不同：

- 多选 Technology：筛选 Catalog；
- 多选 Adapter：形成实际执行计划。

Workflow 中的多选结果应保存为有序 `AdapterPlan`，而不是无序集合，以便以后表达 priority、
fallback、conflict 和 supersedes。首版即使只按顺序并行启用，也不应丢失顺序。

### 4.2 Catalog 分层

```text
AdapterPackage
  machine
    adapterId
    descriptorVersion
    releaseVersion
    platforms[]
    architectures[]
    technologies[]
    ABI
    capabilities[]
    applyModel
    placement
    entrypoint
    configurationSchema

  presentation[]
    locale
    name
    summary
    description
    tags
    risks
    documentation

  artifact
    digest
    size
    publisherIdentity
    signature
```

Presentation 缺少当前语言时可以按 default locale 回退，但不能因为 locale 文件缺失而改变
Adapter 能否加载。反过来，平台不兼容时 UI 可以展示“不可用原因”，不能靠隐藏技术标签来
代替 Registry 的兼容性判断。

## 5. Glyphshift 最小组合模型

```text
DictionaryAsset ─────┐
FontProfile ─────────┼─> WorkflowTarget ─> Runtime Publication
SoftwareExtension ───┤          │
AdapterCatalog ──────┘          └─ AdapterPlan[]
```

### DictionaryAsset

```text
schemaVersion
dictionaryId
releaseVersion
metadata
  defaultLocale
  localized name/description
  sourceLocale/targetLocale
  authors/license/homepage/repository/sourceRevision
  createdAt/updatedAt/tags
entries[]
```

面向特定软件制作的字典，可以在 metadata 中声明 `subjects` 用于 Catalog 搜索和适用性提示，
例如目标软件身份与已验证版本；该声明不能选择 Adapter，也不能直接参与 Runtime Hook。

### FontProfile

```text
fontProfileId
name
ordered candidates/fallbacks
weight/style/coverage policy
optional semantic selectors
```

Font Profile 不引用 Dictionary。字体是否真实存在、当前 Adapter 是否支持 Font Substitute，
分别由 Font Catalog 与 Adapter Feature resolve 决定。

### WorkflowTarget

```text
softwareExtensionId
dictionaryBindings[]  // dictionaryId + enabled + priority + semantic scope
fontBindings[]        // fontProfileId + enabled + semantic scope
adapterPlan[]         // adapterId + enabled + priority
```

所有跨资产关系都在 Binding。复制 Workflow 不复制 Dictionary 或 Font Profile；同一资产可以被
多个 Workflow 复用。

## 6. Metadata 治理

在线版会放大 metadata 的安全边界。Unity 允许 metadata 带字体和代码，说明“扩展区域”如果不
治理，很容易重新耦合甚至形成执行入口。Glyphshift v1 建议：

1. 核心 metadata 使用强类型字段。
2. 扩展字段必须位于 `metadata.extensions`，键使用反向域名或 publisher namespace。
3. Loader 忽略未知的纯数据扩展；不得把扩展解释成代码、路径或 Adapter 配置。
4. 明确拒绝保留名称：`hook`、`adapter`、`platform`、`architecture`、`font`、`entrypoint`、
   `script`、`permission` 和 `runtime`。
5. 在线 Catalog 对 title、description、tags 等字段做长度限制；Runtime 从不读取展示字段。

## 7. 最终建议

可以按该模型进入技术方案和实施，不需要再做旧结构兼容设计。推荐实施顺序是：

1. 先定义新 Dictionary v1（含 `schemaVersion` 与受控 metadata），删除 Hook/Font 属性。
2. 建立独立 Font Profile 与 Font Catalog。
3. 将 Platform、Technology、Machine Descriptor、Presentation Catalog 拆开。
4. 把 UI 单选 Hook 改为有序 Adapter Plan，多 Adapter 在 Workflow 层组合。
5. 最后实现纯组合编译：Dictionary Bindings + Font Bindings + Adapter Plan 生成 Runtime
   Publication。
6. 在线 Catalog 暂不实现网络功能，但现在固定 Artifact Descriptor seam，避免以后把 digest、
   签名或下载状态塞回 Dictionary payload。

复核未发现需要否定该方案的主流实践。需要持续守住的边界只有两个：Dictionary metadata
不能演变为执行配置，Presentation 不能成为 Registry 的兼容性事实来源。
