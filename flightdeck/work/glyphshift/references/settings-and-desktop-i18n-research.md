# Glyphshift 全局设置与桌面端多语言调研

> 调研范围：成熟桌面软件的设置作用域，以及 Vue 3 + Tauri 桌面应用的下一阶段国际化边界。本文只基于官方文档、标准和官方源码形成设计建议，不包含既有版本迁移方案。

## Research Read

### 结论先行

Glyphshift 应借鉴成熟软件对“设置作用域”的控制，而不是照搬它们的设置栏目。真正属于全局设置的内容很少：应用外观、界面语言、生命周期、更新、隐私诊断、网络和账号级默认策略。Dictionary、Font、Adapter、目标软件及其组合关系都不是全局设置，应继续分别由资源管理器和 Workflow 管理。

下一阶段多语言推荐采用 `vue-i18n` Composition API 管理前端核心 UI；Tauri/Rust 后端返回稳定的结构化错误码与类型化参数，由前端把错误码映射到翻译 key。**不推荐后端直接传 vue-i18n key**，更不推荐后端返回已经本地化的句子。在线 Adapter/Dictionary 的市场文案也不能依赖宿主应用的翻译 key，应该随包或目录记录携带自己的本地化 presentation metadata、默认 locale 和 schema 版本。

一句话边界：**Settings 管应用，Manager 管资源，Workflow 管组合，Artifact metadata 管在线身份，前端 i18n 管人类可读的宿主 UI。**

### 本地现状阅读

当前桌面端已经具备适合继续演进的领域边界：Dictionary、Font、Adapter、Workflow 独立，Adapter 描述中已有稳定 ID、名称、摘要、平台与技术信息，Workflow 通过 Adapter Plan 进行组合。Settings 页面目前也明确没有全局 hook、字体或翻译开关。

当前的国际化缺口主要有三类：

1. Vue 页面、组合逻辑和浏览器预览错误中仍有硬编码中文；项目尚未引入 i18n 运行时。
2. 多数 Tauri command 仍以字符串作为错误返回值，前端难以稳定分类、插值和本地化。
3. 未来在线 Adapter/Dictionary 的文案具有独立发布节奏，不能假设其版本与宿主应用翻译资源同步。

因此不能用一条“所有地方都传 key”的管线解决三类问题。核心 UI、结构化后端错误、动态市场文案应分别建模；只有 UI 与展示文案会消费用户界面 locale，目标文本/字典内容语言走 Workflow 的另一条领域管线。

## Source Matrix

### 成熟桌面软件

| 产品 | 官方资料呈现的设置边界 | 适合 Glyphshift | 产品特定、不应照搬 |
| --- | --- | --- | --- |
| PowerToys | General 中集中处理版本/更新、主题、启动、托盘、备份、实验与诊断，同时把具体能力留在各 Utility 页面。[General 官方文档](https://learn.microsoft.com/en-us/windows/powertoys/general) | 界面语言、主题、开机启动、托盘与关闭行为、更新检查、诊断授权、配置备份 | Utility 总开关、Quick Access、Windows Insider 实验和快捷键冲突；“始终以管理员运行”也不应成为 Glyphshift 的普通全局偏好，高权限目标更适合连接时预检和一次性提权操作 |
| VS Code | 明确区分 User 与 Workspace 设置，并进一步提供 Profile；全局偏好与工作内容不是同一作用域。[Settings 官方文档](https://code.visualstudio.com/docs/configure/settings)、[Profiles 官方文档](https://code.visualstudio.com/docs/configure/profiles) | 清晰的作用域、单项恢复默认、只看已修改；界面语言和主题可独立选择并跟随系统。[Display Language](https://code.visualstudio.com/docs/configure/locales)、[Themes](https://code.visualstudio.com/docs/configure/themes) | Default → User → Remote → Workspace → Folder → Language 的复杂覆盖链；Glyphshift 的 Target/Workflow 应通过显式 Binding 组合，而不是靠隐式设置优先级 |
| Docker Desktop | General、Updates、Notifications 等属于应用行为；Resources、WSL、Engine、Kubernetes、Builders 等属于 Docker 领域。[Settings 官方文档](https://docs.docker.com/desktop/settings-and-maintenance/settings/) | 主题、登录时启动、更新、诊断、通知；在线服务成熟后可参考代理模式 | CPU/内存/磁盘 VM 配额、WSL integration、Docker Engine、Kubernetes、Builders 和容器网络 |
| OBS Studio | Settings 包含 General、Stream、Output、Audio、Video、Hotkeys、Advanced；Scene 和 Source 作为工作内容独立存在。[官方界面概览](https://obsproject.com/kb/obs-studio-overview)、[官方源码](https://github.com/obsproject/obs-studio) | 主题、托盘、风险操作确认、普通/高级设置分层 | 推流、编码器、分辨率、音频设备、Scene/Source 都是 OBS 业务配置；不能因为 OBS 把部分业务项放进 Settings，就把 Glyphshift 的 Adapter Plan 或 Workflow 放入全局设置 |
| Zotero | General、Sync、Cite、Export、Advanced 分区；同步、数据目录、数据库完整性和组件更新与具体文献库内容分开。[General](https://www.zotero.org/support/preferences/general)、[Sync](https://www.zotero.org/support/preferences/sync)、[Advanced](https://www.zotero.org/support/preferences/advanced) | 账号关联、同步默认策略、按需下载、打开数据目录、完整性检查、错误上报授权 | 引文样式、OpenURL、导入、Feed、文字处理器插件和 WebDAV 细节均为 Zotero 领域配置 |

隐私方面，VS Code 把诊断级别划分为 `off`、`crash`、`error`、`all`，并说明如何查看实际发送事件。这比模糊的“帮助改进产品”开关更可审计。[VS Code Telemetry 官方文档](https://code.visualstudio.com/docs/configure/telemetry)

### Vue、Tauri 与语言标准

| 主题 | 一手资料 | 对本方案的约束 |
| --- | --- | --- |
| Vue 3 接入方式 | Vue I18n 官方建议在 `legacy: false` 下使用 Composition API，并通过 `useI18n` 取得 Composer。[Composition API](https://vue-i18n.intlify.dev/guide/advanced/composition) | 与当前 Vue 3 Composition API 代码一致；核心壳层用 global scope，局部组件默认继承全局 locale |
| fallback | Vue I18n 支持区域标签的隐式回退、单 locale、数组和 decision map；开发期可以暴露 missing/fallback warning。[Fallbacking](https://vue-i18n.intlify.dev/guide/essentials/fallback) | 不依赖模糊的自动截断处理简繁中文；显式定义 `zh-Hans`/`zh-Hant` decision map 和最终默认 locale |
| 类型安全 | Vue I18n 可以以 master locale 资源推导消息 schema，并由 TypeScript 检查缺 key 或错误 key。[TypeScript Support](https://vue-i18n.intlify.dev/guide/advanced/typescript) | 核心 UI 使用稳定语义 key 和一份完整 master schema；不使用中文原文作为 key |
| 按需加载 | Vue I18n 支持动态导入 locale message，并建议同步设置文档语言属性。[Lazy loading](https://vue-i18n.intlify.dev/guide/advanced/lazy) | P0 语言数量少时可直接打包；语言增多后按 locale 拆包，但 locale resolver 与资源 schema 不变 |
| Tauri 错误返回 | Tauri command 的成功值和错误都可序列化；官方示例展示了 tagged custom error，并明确可以用 code 对应 TypeScript enum。[Calling Rust from the Frontend](https://v2.tauri.app/develop/calling-rust/#error-handling) | 后端用稳定 code + 参数形成 IPC contract，不必退化为字符串，也不需要让 Rust 知道前端翻译资源树 |
| 系统 locale | Tauri OS 插件的 `locale()` 返回 BCP 47 language tag 或 `null`。[OS plugin `locale`](https://v2.tauri.app/reference/javascript/os/#locale) | `system` 是持久偏好；实际 locale 每次启动/系统变化时解析，检测失败走产品默认值 |
| locale lookup | RFC 4647 的 Lookup 用有序语言范围寻找一个最佳结果，逐级截断，并要求调用方明确无匹配时的默认项。[RFC 4647](https://www.rfc-editor.org/rfc/rfc4647.html#section-3.4) | 动态市场文案采用 exact → 显式映射 → base → artifact default 的单结果选择；不能在无匹配时返回空文本 |
| 组件库 locale | Nuxt UI 可用于 Vue，并提供集成国际化/locale 管理的组件。[Nuxt UI Components](https://ui.nuxt.com/docs/components/?page=4) | 切换 locale 时必须同时同步应用翻译器和 Nuxt UI 组件 locale，避免日期、选择器和无障碍标签仍停留在旧语言 |

## Patterns

### 1. 主流产品的共同设置边界

成熟桌面产品反复出现以下模式：

1. 全局设置只控制所有业务对象共享的客户端行为。
2. 项目、库、Profile、Scene、扩展等业务对象拥有自己的配置和生命周期。
3. 更新、安全、隐私等应用政策不允许被某个业务对象偷偷覆盖。
4. 不受当前平台或能力支持的设置，要么隐藏，要么显示不可用原因；不能保存一个没有执行意义的值。
5. 高风险、低频、面向排障的内容进入 Advanced，并配有说明、确认和恢复入口。
6. 设置开关必须有真实执行路径。没有 updater、telemetry、账号或云同步服务时，不提前展示空开关。
7. “检查更新”“打开日志目录”“导出诊断包”“立即同步”“检查完整性”“恢复默认”是命令，不是持久布尔值。

由此可得到适合 Glyphshift 的稳定分层：

```text
AppSettings
  appearance        # locale、theme
  lifecycle         # autostart、close behavior、tray
  privacy           # 只在真实采集/发送存在时配置
  connectivity      # 在线服务成熟后配置代理/下载默认策略
  notifications     # 应用级通知默认策略

Account / Online Service
  account link、credential reference、sync defaults

Dictionary Artifact
  content + artifact metadata + remote identity/version/state

Font Profile
  font candidates + fallback

Adapter Catalog
  platform + technology + capabilities + availability/version

Workflow Target
  adapter plan + dictionary binding + font binding
```

### 2. 推荐的全局设置分类与优先级

#### P0：现在建立真实模型和 UI

| 分类 | 设置/动作 | 建议模型 | 说明 |
| --- | --- | --- | --- |
| 外观 | 界面语言 | `localePreference = system \| <supported-bcp47-tag>` | 保存的是偏好而非检测结果；第一阶段至少完整支持 `zh-CN` 与 `en-US` |
| 外观 | 主题 | `theme = system \| light \| dark` | 系统模式不应在保存时被改写为当前明暗值 |
| 生命周期 | 开机启动 | `launchAtLogin: boolean` | 对常驻型翻译工具是真正全局；设置是否可用由平台能力决定 |
| 生命周期 | 关闭行为 | `closeBehavior = quit \| keepRunning` | 必须与托盘状态和运行中 Workflow 提示一致，避免用户误以为已经退出 |
| 生命周期 | 托盘运行 | 可由 `closeBehavior` 派生或作为明确能力状态 | 不建议产生互相矛盾的两个开关 |
| 设置基础设施 | schema 版本 | `settingsSchemaVersion = 1` | 当前未发布，不写旧版兼容分支；只预留版本字段和未来迁移入口 |
| 设置基础设施 | 稳定 ID、默认值、平台/能力条件、是否需重启 | 设置描述元数据 | UI 与持久层共享定义，禁止按显示文案识别设置 |
| 恢复 | 单项恢复/全部恢复 | command | 恢复动作不持久化为设置值 |
| 本地诊断 | 日志级别/保留策略；打开日志目录、导出诊断包 | preference + commands | 诊断内容必须去敏；“打开/导出”是动作 |

P0 同时设立一条硬边界：Dictionary、Font、Adapter、Workflow、目标软件、hook 技术和翻译启停字段不得进入 AppSettings。默认字典、默认字体或默认 Adapter 看似方便，实际会再次制造隐式耦合，应由 Workflow 模板或创建向导表达。

#### P1：对应能力真正上线时加入

| 分类 | 设置/动作 | 上线前提 |
| --- | --- | --- |
| 更新 | 自动检查、自动下载、稳定/预览渠道、立即检查 | 真实发布渠道和 updater 已存在；此前只保留扩展点，不展示 UI |
| 隐私 | `off \| crash \| error \| all`、查看待发送内容 | 真实遥测或错误上报器存在；默认值、发送目的和内容可解释 |
| 在线账号 | 关联/解除账号、同步状态、立即同步 | 在线字典账号服务存在；凭据只保存系统凭据库引用 |
| 在线字典默认策略 | 自动同步默认值、`eager \| onDemand` 下载默认值 | 目录/订阅协议稳定；具体字典允许拥有自己的订阅和固定版本状态 |
| 网络 | `system \| direct \| manual` 代理 | 在线服务对自定义代理有现实需求；优先使用系统代理 |
| 通知 | 按类别启停 | 已有稳定通知类别和真实通知通道 |
| 数据维护 | 备份/恢复、完整性检查、受控数据目录迁移 | 数据格式和校验流程稳定；这些主要是动作而非持续开关 |
| 设置规模治理 | 搜索、只看已修改、高级分组 | 设置数量增长到需要检索时 |
| 可访问性 | 减少动效、UI 缩放或高对比策略 | 组件与样式实际支持；默认跟随操作系统 |

不建议创建暂时无行为的“云端模式”“自动更新”“发送诊断”占位开关。可以先保留 schema/服务接口版本，但 UI 只呈现可兑现的能力。

### 3. 核心 UI 的 vue-i18n 方案

核心应用文案采用稳定、语义化 key，例如 `settings.appearance.language`、`errors.dictionary.inUse`，而不是中文或英文原文。推荐结构：

```text
i18n/
  index              # createI18n、resolver、切换入口
  locales/
    zh-CN             # 当前 master schema
    en-US             # P0 必须完整
  error-message-map  # backend error code -> frontend message key
```

采用 `legacy: false` 和 Composition API。应用壳层使用 global scope；只有真正可复用、需要自带消息的独立组件才使用 local scope，且默认继承 global locale。不要让页面各自保存 locale 状态。

当前代码以中文为主，因此第一阶段可以让 `zh-CN` 成为类型推导用的 master schema，减少迁移风险；`en-US` 在合入前也必须完整。未来若要改变产品默认语言，只调整 resolver/default，不更改语义 key。

开发与 CI 应检查：

- 所有声明支持的 core locale 都满足 master schema。
- 不存在运行时拼接翻译 key 的业务路径。
- 插值参数名和必要 plural 分支一致。
- 开发环境保留 missing/fallback warning；测试或生产环境按需关闭噪声。
- locale 切换同步 Vue I18n、Nuxt UI 组件 locale 和文档 `lang`。

### 4. 三类 locale 必须正交

Glyphshift 至少存在三种外形相似、语义完全不同的语言字段：

| 语言概念 | 回答的问题 | 所属模型 | 是否受 Settings 的界面语言控制 |
| --- | --- | --- | --- |
| UI locale | Glyphshift 自己的按钮、菜单、错误提示用什么语言？ | `AppSettings.localePreference` + runtime `effectiveLocale` | 是 |
| 目标软件/字典内容语言 | 被拦截文本是什么语言、字典把什么源语言映射到什么目标语言？ | Dictionary metadata / Workflow 的翻译策略 | 否 |
| Adapter/Dictionary presentation locale | 市场卡片中的名称、摘要、说明优先显示哪一份本地化文案？ | Artifact `presentation` + `defaultLocale` | 以 UI locale 作为“查找偏好”，但独立 fallback |

约束：

- 用户把 Glyphshift UI 切换成英文，不得改变正在运行 Workflow 的源语言、目标语言或所绑定字典。
- Dictionary 的 `sourceLanguage`、`targetLanguage` 或更一般的 `contentLanguages` 是内容元数据，不是 presentation locale，也不是 AppSettings。
- Adapter 通常处理技术/API 而不是自然语言；它可以有本地化说明，但不应因此获得一个“翻译目标语言”字段。
- presentation locale 选择失败，只影响市场卡片展示；不能让 Adapter/Dictionary 被判定为不可用。
- 目标软件语言若需要自动检测或显式覆盖，应属于 Dictionary/Workflow 翻译策略，不能复用 UI locale resolver。

### 5. UI locale preference 与 fallback 边界

区分两个值：

- `localePreference`：用户保存的 `system` 或显式 BCP 47 tag。
- `effectiveLocale`：本次运行真正加载的受支持 locale。

解析优先级：

```text
显式用户选择
  -> 系统 locale（仅 preference = system）
  -> canonicalize
  -> exact supported locale
  -> 显式 decision map（尤其 zh-Hans / zh-Hant）
  -> language base
  -> 产品默认 zh-CN
```

建议首版 decision map 至少明确：

```text
zh-CN / zh-SG / zh-Hans-* -> zh-CN
zh-TW / zh-HK / zh-MO / zh-Hant-* -> zh-TW（仅在资源完整上线后声明支持）
en-* -> en-US
其他 -> zh-CN
```

如果 P0 只交付 `zh-CN` 与 `en-US`，就不能把 `zh-TW` 宣称为已支持；繁体系统 locale 在此阶段按公开规则落到产品默认值，并在真正加入完整繁体资源后扩展 decision map。不能把 `zh` 无条件等价为简体并把这种隐式行为藏在库默认值里。

核心 UI 的 key fallback 与市场文案的 locale lookup 是两个问题：核心资源由同一版本构建，缺 key 应在 CI 阻止；fallback 只作运行时保险。市场文案是独立发布内容，缺少某个 locale 是合法状态，必须按 RFC 4647 Lookup 和 artifact default 得到一个确定结果。目标软件/字典内容语言不参与这两种 fallback。

### 6. “后端传 key”的最终判断

**最终判断：否决“Rust 后端直接传 vue-i18n key，前端收到后直接 `t(key)`”作为通用协议。**

后端应返回领域/传输契约，而不是前端资源树路径：

```ts
type CommandError = {
  schemaVersion: 1
  code: 'dictionary.in_use' | 'workflow.not_found' | 'adapter.unavailable'
  args: Record<string, string | number | boolean>
  diagnosticId?: string
}
```

前端维护显式、可检查的映射：

```text
dictionary.in_use      -> errors.dictionary.inUse
workflow.not_found     -> errors.workflow.notFound
adapter.unavailable    -> errors.adapter.unavailable
unknown code           -> errors.unknown + safe diagnosticId
```

理由如下：

1. 错误码是领域/API contract；翻译 key 是 UI 实现细节。key 重命名不应迫使 Rust 或第三方 Adapter 升级。
2. 结构化 code 可以与 TypeScript/Rust 枚举对应，便于穷尽测试、遥测归类和未知版本兜底；Tauri 官方错误序列化模型直接支持这种形式。
3. `args` 可以做类型与敏感信息审查；让后端传整句或模板会混淆语法顺序、plural、HTML 转义和隐私边界。
4. 宿主前后端即使当前一起发版，未来动态 Adapter、Dictionary 市场内容也会独立发版，直接 key 会形成无法可靠协商的跨版本依赖。

允许的例外很窄：前端内部模块可以互传自己的 message key，但它不应越过 Tauri IPC、在线 API 或插件/包边界。若某个后端结果只是开发日志，可返回技术性 `detail`，但不能直接作为用户界面文案显示。

错误 contract 只预留 `schemaVersion: 1`，当前不写旧格式兼容分支。接收方仍应对未知 `code` 使用通用提示，因为“较新后端/扩展返回新错误码”是正常前向健壮性，不是旧版本迁移方案。

### 7. 动态 Adapter/Dictionary 市场文案

动态内容不能引用宿主 `vue-i18n` key。推荐把机器信息与展示信息分离：

```json
{
  "schemaVersion": 1,
  "id": "stable-artifact-id",
  "artifactVersion": "1.0.0",
  "defaultLocale": "en-US",
  "presentation": {
    "en-US": {
      "name": "...",
      "summary": "...",
      "description": "..."
    },
    "zh-CN": {
      "name": "...",
      "summary": "...",
      "description": "..."
    }
  }
}
```

其中：

- `id`、平台、技术、能力、版本、依赖、校验和等是 locale-neutral 机器字段。
- `presentation` 是独立元数据区域；`defaultLocale` 必填并保证对应记录完整。
- 宿主选择文案时使用 exact → decision map → base → artifact `defaultLocale`；仍失败时显示稳定技术 ID 和通用缺失说明，不能显示空白。
- 平台名、技术名、能力枚举等由宿主认识的稳定 ID 可以映射为宿主翻译；第三方未知 tag 使用包提供 label 或安全原值。
- 市场服务可以按 locale 下发 presentation，但包清单仍需声明 `schemaVersion`、`defaultLocale` 和可用 locales，以支持缓存、离线与版本审计。
- 文案不得携带可执行 HTML；富文本若未来需要，应使用受限格式和独立 schema/清洗策略。

在线字典还需要明确三种不同归属：

| 归属 | 内容 |
| --- | --- |
| 全局在线默认策略 | 是否自动同步、新订阅的默认下载策略、系统/直连/手动代理 |
| Dictionary metadata | `origin`、远端 ID、作者、许可证、`sourceLanguage`/`targetLanguage`/`contentLanguages`、`schemaVersion`、`artifactVersion`、presentation、签名、发布时间 |
| 每字典本地状态 | 已订阅/未订阅、固定版本、最后同步、可用更新、冲突状态 |

Workflow 只引用 Dictionary，不保存在线账号、下载地址、presentation 或全局同步策略。这样同一字典可以被多个 Workflow 复用，在线身份也不会污染组合模型。

## Local Application

### 对 Glyphshift 方案的直接落点

1. 保持现有 Dictionary、Font、Adapter、Workflow 解耦，不在 Settings 中增加“默认 hook”“默认字体”“默认字典”或全局翻译开关。
2. 新建一个浅而稳定的 `AppSettings` 聚合，只承载 P0 全局项，并包含 `settingsSchemaVersion: 1`。当前没有发布历史，因此不实现兼容旧版本的迁移器，只留未来按版本迁移的入口。
3. Settings 页面从当前说明页升级为真实的“外观 / 生命周期 / 诊断”页面；账号、在线、更新只在对应服务存在后进入 P1。
4. 核心 Vue UI 引入 `vue-i18n` Composition API；硬编码中文按页面逐步迁移到语义 key，首批资源为完整 `zh-CN` 与 `en-US`。
5. 新增唯一 locale resolver，统一用户偏好、Tauri 系统 locale、Vue I18n、Nuxt UI 与 `document.lang`，禁止页面自行判断语言。
6. Tauri command error 从 `String` 收敛到 `CommandError { schemaVersion, code, args, diagnosticId? }`；前端用集中映射转换成核心翻译 key。
7. Adapter/Dictionary 市场 schema 增加独立 presentation metadata、`defaultLocale` 与 schema/artifact version，不存宿主翻译 key。
8. 动态元数据 lookup 与 core UI fallback 可以复用 BCP 47 规范化工具，但保持不同的最终兜底规则；字典内容语言/Workflow 目标语言使用独立领域类型和解析路径。

### 建议的不变量

- 任一 AppSettings 字段都必须回答：“它是否同时影响所有 Workflow/资源，且不依赖某个具体对象？”回答否时不进入全局设置。
- 任一 IPC 错误必须有稳定 `code`；用户可见文本只能由前端 locale 层产生。
- 任一在线 artifact 必须有稳定 ID、schema 版本、artifact 版本、默认 locale 和完整的默认 presentation。
- 任一声明为“支持”的 core locale 必须在 CI 中通过完整 key 检查。
- UI locale、artifact presentation locale、Dictionary/Workflow 内容语言不得共享一个无语义的 `locale` 字段。
- 未知错误码、未知 locale、未知第三方 tag 都必须有安全、可诊断且不崩溃的 fallback。
- 设置 UI 不展示尚无执行路径的开关。

### 明确不做

- 不实现未发布旧设置或旧翻译资源的兼容层。
- 不引入 VS Code 式多层设置覆盖系统。
- 不把 Adapter/Dictionary 市场文案编译进宿主核心 locale 文件。
- 不让 Rust、在线服务或第三方包依赖前端 vue-i18n key。
- 不把账号凭据、下载地址或同步状态混入 Workflow。
- 不因为未来可能支持 macOS 就提前展示无效平台设置；平台差异由能力条件和 Adapter catalog 表达。

## Next Step

建议按以下顺序进入技术方案与实施，先稳定契约，再铺 UI：

1. **冻结边界与 ADR**：确认 AppSettings、CommandError、Artifact presentation 三个 schema 的职责和 `schemaVersion: 1`；把“后端传错误码，不传翻译 key”写成架构决策。
2. **P0 设置基础设施**：实现设置仓储、默认值、稳定 ID、能力条件、恢复命令；只接入真实可执行的语言、主题、启动/关闭行为和本地诊断。
3. **核心 i18n 骨架**：接入 `vue-i18n` Composition API、master schema、`zh-CN`/`en-US`、locale resolver，以及 Nuxt UI/`document.lang` 同步。
4. **结构化 IPC 错误**：定义 Rust/TypeScript 对应的 tagged error；先覆盖 Settings 和新增路径，再逐 command 清理字符串错误。未知 code 统一走安全兜底。
5. **页面文案迁移**：优先应用壳、导航、Settings 和全局错误，再迁移 Dictionary、Font、Adapter、Workflow 页面；用 CI 阻止缺 key 和原文 key。
6. **在线 schema 预留**：为 Dictionary/Adapter 定义 presentation metadata、default locale、远端身份和版本字段；当前只定义版本 1，不实现旧版兼容和无后端的市场开关。
7. **P1 随服务启用**：发布渠道、账号同步、市场下载、代理和遥测服务分别就绪后，再开放对应设置与动作。
8. **验证**：为 locale resolution、fallback decision map、错误码映射、未知 code、artifact default locale 和设置恢复行为建立确定性测试；桌面 UI 使用仓库既有 Playwright 测试入口验证切换语言、重启持久化和不同能力状态。

实施前最后一个产品决策只需确认：P0 是否立即提供 `en-US`。本调研建议“是”；若暂时只发布中文，也应先完成结构化 key 和 locale resolver，但 UI 不应把未完整的语言列为可选项。
