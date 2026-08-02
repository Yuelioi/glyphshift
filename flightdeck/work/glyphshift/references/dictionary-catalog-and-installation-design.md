# Dictionary Catalog 与可信安装技术设计

Status: Ready for implementation
Date: 2026-08-03

## 1. 结论

在线 Dictionary 分发由一个深 `DictionaryDistribution` Module 承担。它对产品层只暴露查询、
安装和安装状态三个行为，内部隐藏 Catalog 访问、presentation locale 回退、release 解析、限量
下载、摘要校验、发布者信任验证、Dictionary payload 校验、原子落盘和崩溃恢复。

现有 `DesktopBackend` 不应继续同时拥有 Dictionary payload codec、Catalog、下载和安装状态。
实施时先把 `DictionaryArtifact` 解析与验证提取为可复用的 `DictionaryPackage` Module，再让
Distribution Module 和 Desktop Backend 通过同一 package interface 消费它。Runtime、Decision、
Workflow resolve 和 Native Adapter 永远不访问 Catalog 或网络。

## 2. 五种权威事实

| 事实 | 唯一职责 | 明确不拥有 |
| --- | --- | --- |
| Dictionary Payload | Dictionary ID、release version、便携 metadata、entries | URL、digest、签名、安装时间 |
| Catalog Release | release 发现、索引字段、presentation、Artifact Descriptor | 本地路径、当前工作副本、Workflow 状态 |
| Artifact Statement | 签名绑定的不可变制品事实 | mirror URL、presentation、安装状态 |
| Dictionary Installation | 本机资产与已验证 release 的来源关联 | Dictionary 内容、启用状态、UI locale |
| Active Dictionary | 当前被编辑和 Workflow 引用的本地工作副本 | 对其当前内容未经验证的发布者声明 |

Dictionary ID + release version 唯一标识 Dictionary Release；本地 `revision` 只做工作副本 CAS。
Catalog Presentation 与 Dictionary Metadata 可以都含名称和说明，但权威范围不同：payload 中的
字段是离线默认展示，Catalog Presentation 是发现页面的 locale-specific 投影，永不写回 payload。

## 3. 必须守住的场景

1. Catalog 有 `en-US` 和 `zh-CN`，当前 UI locale 是 `zh-Hans-CN`：先尝试精确/父级匹配，再回退
   release 的 default presentation；不能回退到另一条任意 presentation。
2. Mirror URL 改变但 digest 未变：无需重新签名；URL 不进入 Artifact Statement。
3. 下载内容通过 digest 和签名，但 payload 的 Dictionary ID 或 release version 与 release 不同：
   安装仍必须拒绝。
4. descriptor 自报 publisher 与 Trust Verifier 确认的身份不同：以验证结果为准并拒绝冒名。
5. 用户安装后编辑词典：Runtime 使用当前工作副本，Installation 派生状态变为 `modified`，原签名
   只证明保留在制品存储中的已验证 release，不证明当前编辑内容。
6. 更新遇到 unmanaged 或 modified 工作副本：不得静默覆盖；需要显式 replace/fork 决策。
7. Catalog、网络或信任服务不可用：本地 Dictionary 和已启用 Workflow 继续工作。
8. 安装中途退出：重开后只能得到旧工作副本或完整新工作副本，不能留下半个 JSON 或伪 installed。

## 4. 深 Module 与 seam

```text
Vue / Tauri commands
        |
DesktopApplication
        |
DictionaryDistribution  <----- external interface: query / install / installations
   |          |          |
   |          |          +-- DictionaryInstallStore (filesystem + tempdir tests)
   |          +------------- ArtifactTrustVerifier port
   +------------------------ DictionaryDistributionPort
                                  production HTTP adapter
                                  in-memory test adapter
        |
DictionaryPackage  <--------- decode / validate / encode Dictionary payload
        |
DesktopBackend / Workflow resolve / Runtime Publication
```

### 4.1 外部 interface

```text
DictionaryDistribution
  query(CatalogQuery, requestedPresentationLocale) -> CatalogPage
  install(InstallRequest) -> DictionaryInstallationView
  installations() -> DictionaryInstallationView[]
```

`InstallRequest` 只包含 Catalog ID、Dictionary ID、精确 release version 和明确的 replacement
policy。调用方不负责选择 mirror、计算 hash、调用 verifier、解析 payload、安排写入顺序或恢复
事务；这些复杂度全部留在 Module 内。

### 4.2 内部 ports

`DictionaryDistributionPort` 是远端但由产品拥有的 seam。production HTTP adapter 负责协议与认证，
in-memory adapter 负责确定性合同测试：

```text
query(query) -> CatalogReleaseSummary[]
release(DictionaryReleaseKey) -> CatalogRelease
fetch(DictionaryArtifactDescriptor) -> bounded byte stream
```

`ArtifactTrustVerifier` 是信任机制 seam。它接收强类型 Artifact Statement 与 Signature Envelope，
返回经过验证的 Publisher Identity；production adapter 可以在发布策略确定后使用第一方 keyring、
Sigstore 或 TUF，测试 adapter 只接受显式 fixture。Core 不通过字符串分支假装验签。

## 5. Catalog 与 presentation contract

首版 Catalog 文档使用 `glyphshift.dictionary-catalog/1`：

```json
{
  "schema": "glyphshift.dictionary-catalog/1",
  "catalogId": "glyphshift.official",
  "releases": [
    {
      "dictionaryId": "dictionary.menu.zh-cn",
      "releaseVersion": "1.2.0",
      "sourceLocale": "en-US",
      "targetLocale": "zh-CN",
      "defaultPresentationLocale": "zh-CN",
      "presentations": [
        {
          "locale": "zh-CN",
          "name": "菜单简体中文",
          "summary": "常用菜单与对话框文字",
          "tags": ["menu", "dialogs"]
        },
        {
          "locale": "en-US",
          "name": "Simplified Chinese menus",
          "summary": "Common menu and dialog text",
          "tags": ["menu", "dialogs"]
        }
      ],
      "artifact": {
        "mediaType": "application/vnd.glyphshift.dictionary+json;version=2",
        "size": 4096,
        "digest": { "algorithm": "sha256", "value": "<sha256>" },
        "downloadUrls": ["https://<catalog-host>/<artifact>"],
        "publisherIdentity": "publisher.example",
        "signature": {
          "scheme": "<configured-scheme>",
          "keyId": "<key-id>",
          "value": "<signature-envelope>"
        }
      }
    }
  ]
}
```

约束：

- Dictionary ID、Catalog ID 和 locale 使用受控标识；release version 必须是 SemVer。
- 同一 release 的 presentation locale 唯一，default locale 必须真实存在。
- name、summary 和 tags 有长度、数量上限；未知字段按 schema 合同拒绝，不进入 Runtime。
- production URL 只接受 HTTPS；下载字节数不得超过 descriptor size 或产品硬上限。
- query 只返回已选择的 presentation 与 effective locale，页面不自己实现 fallback。

Presentation fallback 顺序为：规范化后精确 locale、逐级父 locale、release default locale。UI
locale、Dictionary source/target locale 和 Presentation locale 是三个不同输入。

## 6. 签名边界

Artifact signature 绑定 `glyphshift.dictionary-artifact-statement/1` 的强类型字段：

```text
schema
dictionaryId
releaseVersion
mediaType
size
digestAlgorithm
digestValue
publisherIdentity
```

不绑定 download URL、mirror 顺序、presentation、发布时间、安装时间或本机状态。签名 adapter
拥有 envelope scheme 与 canonical encoding；产品代码不拼接临时 JSON 作为待签文本。

校验顺序固定：

1. Catalog/release contract 与大小硬上限。
2. 限量读取并验证实际 size。
3. 计算并常量时间比较 SHA-256 digest。
4. Trust Verifier 验证 statement、signature 和 publisher policy。
5. `DictionaryPackage` 解析 payload schema 并校验所有字段。
6. 比较 payload Dictionary ID/release version 与已验证 statement。
7. 全部成功后才允许进入本地安装事务。

Catalog 本身的认证与 Artifact signature 是两件事。Artifact signature 保护发布内容身份；Catalog
transport/index authenticity 由 Distribution adapter 的协议负责。Presentation 被篡改不会获得
执行权限，但仍应由生产 Catalog 认证保护用户体验。

## 7. 本地安装模型

```text
dictionaries/<dictionary-id>.json
dictionary-artifacts/sha256/<digest>.json
dictionary-installations/<dictionary-id>.json
dictionary-installations/.transactions/<dictionary-id>.json
```

- `dictionaries/` 保持当前可编辑、可被 Workflow 引用的 active working copy。
- `dictionary-artifacts/` 保存内容寻址、只写一次的已验证 release payload，为 provenance 与恢复
  提供不可变来源。
- `dictionary-installations/` 保存 `glyphshift.dictionary-installation/1`，不保存绝对路径。
- `.transactions/` 是安装 Module 的崩溃恢复实现细节，不进入产品 View 或 Flightdeck。

Installation Record 至少包含 Catalog ID、Dictionary ID、release version、media type、size、digest、
verified Publisher Identity、Signature Envelope、installed time 和安装时 payload revision。它不包含
download URL、当前 UI locale、Workflow 启用状态或目标软件事实。

安装事务顺序：

1. 在同一数据根目录暂存下载并完成全部验证。
2. 以 digest 发布不可变 artifact；已存在且内容匹配时复用。
3. 写入包含目标 record 的 pending transaction。
4. 按 replacement policy 原子发布 active working copy。
5. 原子发布 Installation Record，删除 transaction。

重开时若 transaction 存在：active copy 匹配新 digest 时完成 record；否则保留旧 active copy 与旧
record 并移除未完成事务。任何恢复路径都不能把未验内容标成 installed。

Installation 状态由当前事实派生：

- `verified`：active copy digest 与 Installation Record 一致。
- `modified`：record 和已验证 artifact 存在，但 active copy 已被本地编辑。
- `missing`：record 存在但 active copy 缺失；Runtime 不消费它。
- `unmanaged`：active Dictionary 没有 Installation Record，是本地创建或导入资产。

## 8. Desktop 与 UI seam

`DesktopSnapshot` 后续为每个 Dictionary Summary 增加 installation summary：状态、installed release、
verified publisher 和可选 update release。普通 UI 不展示 URL、signature bytes、key ID、本地路径或
digest；这些只进入诊断/详情能力存在后的受控表面。

Dictionary 页面仍是唯一主导航入口，本地 Library 为默认视图，Catalog 是同页的明确模式，不新增
顶栏一级导航。安装、更新和 modified replacement 都通过结构化 `CommandError/1` 返回语义 code。

首版错误至少区分：Catalog unavailable、release missing、artifact too large、size mismatch、digest
mismatch、untrusted publisher、invalid signature、invalid payload、release identity mismatch、local
changes conflict 和 storage failure。UI 提供恢复动作，不显示 Rust 或 verifier 原句。

## 9. 测试 interface

只从 Module interface 和产品 seam 验收：

1. `DictionaryPackage`：Dictionary `/2` round-trip、未知 schema、非法 metadata/entry、ID/version。
2. `DictionaryDistribution` + in-memory adapters：query、locale fallback、mirror、size/digest/signature、
   publisher mismatch、payload identity mismatch、幂等安装与 explicit replace。
3. File install store + tempdir：重开、modified/missing/unmanaged、pending transaction 四种恢复点。
4. Desktop Backend：本地创建与已安装 Dictionary 共享 resolve，但 provenance 不进入 Dictionary
   metadata 或 Runtime Publication。
5. Tauri/Playwright：Catalog 离线、查询、安装、modified 冲突、双语言 presentation fallback。
6. 架构扫描：Runtime/Decision/Native crate 不依赖 Catalog、HTTP、signature 或 installation crate。

## 10. 实施顺序

1. 从 `glyphshift-desktop-backend` 提取 `DictionaryPackage` codec/validator，保持现有 `/2` 文件与
   Desktop 行为不变。
2. 新建深 `DictionaryDistribution` Module，先用 in-memory Distribution/Trust adapters 锁定 interface。
3. 实现内容寻址 artifact store、Installation Record、transaction recovery 与 derived state。
4. 让 Desktop Backend 通过 package/store interface 读取 active dictionaries，增加 installation view。
5. 加入 Tauri commands 与结构化错误，再实现 Dictionary 页 Catalog 模式。
6. Catalog endpoint、production trust adapter 和发布工具具备真实配置后再打开网络；没有 endpoint
   或 key policy 时不内置示例 URL、不提供伪“在线”状态。

## 11. 非目标

- 不让 Runtime、Decision、Workflow resolve 或 Adapter 访问网络。
- 不信任 payload 自报的 digest、download URL、publisher 或 signature。
- 不自动覆盖 unmanaged/modified Dictionary，不静默丢弃本地编辑。
- 不把 Presentation 写回 Dictionary Metadata，不用 UI locale 改变 Runtime 内容语言。
- 不在签名策略未确定前自创密码算法、硬编码 key 或声称生产验签完成。
- 不实现账号、付费、评论、评分、自动更新或发布服务。
