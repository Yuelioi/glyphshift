# Glyphshift 桌面 UI 深化调研

> 范围：高密度 Windows 桌面管理器的页面骨架、表面层级、设置页、详情编辑页与明暗主题。
> 目标不是换皮，而是把现有管理页、详情页和设置页收敛为同一种产品语言。

## 结论

Glyphshift 当前的主要问题不是某个颜色值，而是页面骨架和背景角色没有统一：管理页使用“页头 +
满宽数据表面”，Software 详情退化为裸表单，Settings 使用窄内容列，Workflow 详情又同时叠加画布、
外框、侧栏和内容区。浅色主题的最高亮表面接近纯白并覆盖大面积区域，进一步放大了割裂感。

下一步应保留现有高密度 Windows 管理器方向，并收敛为三类稳定角色：

1. **应用框架**：标题栏、详情头和导航框架；同一主题中保持连续。
2. **页面画布**：页面周围留白与列表外部背景；浅色使用低亮度灰白，与纯白 Table 内容面区分。
3. **工作表面**：表格、表单和当前任务内容；只在真实分区处使用边界或背景差，不重复套卡片。

管理页继续使用“页头 + 满宽数据面板”。详情页统一使用全宽详情头和占满剩余空间的工作表面；
Workflow 因存在四个真实任务分区而保留左侧单层导航，Software 只有一组资料时不创建伪导航，改用
左右对齐的表单行。Settings 采用全窗口、单列、立即生效的设置列表，并让外层工作表面与其他页面
一致。

## 本地审计

| 现状 | 分类 | 影响 | 改进方向 |
| --- | --- | --- | --- |
| Software 管理页有标准页头和表格框架，详情页只有三项裸表单与两条横线 | 概念不一致 | 同一对象从列表进入详情后像进入了另一套产品 | 复用详情头和工作表面；表单采用标签/说明与控件左右对齐 |
| Settings 有标准页头，但正文是独立窄列与裸分隔线 | 一次性实现 | 页面空白区与管理页不一致 | 工作表面占满剩余区域，内部内容限制可读宽度 |
| Workflow 使用画布、外框、侧栏底色、活动项底色和内容底色 | 表面角色重叠 | 相近灰色块过多，活动项像浮在侧栏上的白块 | 侧栏只承担框架色；活动项使用低强度选择色；右区保持单一内容表面 |
| 浅色层级缺少稳定角色 | 缺失主题级规则 | 灰色混用时显脏，纯白无边界时又刺眼 | 页面框架使用冷灰，Table 内容面使用纯白，靠表头、分页和薄边界分层 |
| 详情头目前浮在页面内，没有形成稳定框架 | 缺失共享模式 | Workflow、Software、Dictionary、Probe 的主体起点不够明确 | 详情头提升为全宽固定框架，统一返回、状态和主要动作 |

Impeccable 静态检测没有发现机械式布局反模式；本次问题属于视觉层级和共享模式不足，不能用检测
结果替代实际截图判断。

## Source Matrix

| 类型 | 一手资料 | 可复用结论 | 不照搬的部分 |
| --- | --- | --- | --- |
| Windows 平台设置 | [Microsoft：Guidelines for app settings](https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings) | 设置页应占满窗口、正文可限制约 1000–1100px、相关设置按单列分组、变更立即生效；标题、说明、图标与右侧控件可以形成稳定行结构 | 不为只有两个设置引入多级设置导航或层层卡片 |
| Windows 导航 | [Microsoft：NavigationView](https://learn.microsoft.com/en-us/windows/apps/design/controls/navigationview) | 返回是页面框架的一部分；单选导航应具有稳定选中模型 | Software 没有多个真实分区，不创建只有一个条目的侧栏 |
| Fluent 2 | [Color tokens](https://fluent2.microsoft.design/color-tokens/)、[Design tokens](https://fluent2.microsoft.design/design-tokens) | 用语义 alias 管理背景、文字、边界和交互状态；明暗主题保持角色一致，只更换值 | 不引入 Mica/Acrylic；Glyphshift 当前使用稳定实色更合适 |
| Adobe Spectrum | [Using color](https://spectrum.adobe.com/page/using-color/)、[Color system](https://spectrum.adobe.com/page/color-system/) | 大区域背景层用于应用框架和内容层级，不用于单个组件装饰；专业工具可让框架与主内容使用不同层级 | 不恢复 Adobe 式复杂面板坞站，Glyphshift 仍是轻量管理器 |
| IBM Carbon | [Color overview](https://preview.carbondesignsystem.com/building-blocks/foundations/color/overview) | 浅色层交替、深色层逐级变亮；背景、Layer、Field、Border 分别按角色命名 | 不复制 Carbon 的品牌蓝、尺寸或企业仪表盘结构 |
| 生产桌面工作台 | [VS Code Theme Color Reference](https://code.visualstudio.com/api/references/theme-color) | 工作台、侧栏、面板、设置行、输入框、悬停和边界各有稳定语义角色；分隔主要依赖背景与单边界 | 不开放用户级全量主题自定义，也不复制编辑器 Tab 模型 |
| 现有组件库 | [Nuxt UI](https://ui.nuxt.com/) | 继续通过 CSS 变量与语义颜色统一现有 Input、Select、Table、Modal；无需新增 UI 库 | 不用组件默认色替代 Glyphshift 自己的应用框架 token |
| 可复核开源实现 | [VS Code repository](https://github.com/microsoft/vscode)、[MIT License](https://github.com/microsoft/vscode/blob/main/LICENSE.txt) | 可用于核对成熟桌面工作台的表面角色和设置行状态，许可允许参考实现方式 | 不复制源代码或品牌资产，只采纳可验证的结构原则 |

## Glyphshift 视觉方案

### 页面骨架

```text
Title bar / primary navigation       application frame
└─ Page
   ├─ list state: page header + full data surface
   └─ detail state: full-width detail header
      └─ editor surface
         ├─ simple editor: aligned form rows
         └─ compound editor: section rail + one content plane
```

- 列表页标题继续使用图标、标题、说明和右侧动作；数据面板占满剩余高度。
- 详情头横向占满模块，返回和保存的位置保持稳定；主体不再从一组悬浮元素开始。
- Software 使用一块工作表面和横向表单行；避免只有一项的侧栏，也避免表单缩在左上角。
- Workflow 左栏只显示四个真实任务分区；活动项使用 `selection`，不再以另一块高亮表面覆盖侧栏。
- Settings 外层工作表面填满剩余高度，内部最大宽度约 900px；语言和主题是两条设置行。

### 颜色与表面

- 使用钴蓝作为唯一强调色；状态色仍只表达成功、警告和失败。
- 浅色页面框架采用偏冷灰白，Table 内容面使用纯白；两者通过表头、分页和薄边界分层。
- 深色画布、框架和内容逐级变亮，但相邻大区域不超过一个层级。
- 输入框使用 Field/Inset 角色，不直接继承任意页面背景。
- 大区域通过一种手段分隔：背景差或 1px 边界；不同时使用明显阴影和边框。
- Hover、Selected、Focus 使用独立语义 token，不把静态层级颜色拿来充当交互状态。

### 密度与响应式

- 保留 11–12px 正文、9–10px元数据和 16px 页面边距，不把桌面管理器放大成网页后台。
- 1440×900 使用标签/说明与控件左右对齐；960×640 时表单行折为单列，详情头动作仍可见。
- 表格、长目录和 Workflow 内容区各自滚动，标题栏、详情头和底部分页保持稳定。
- 当前分页已经保证 Vue 只渲染当前页，不叠加虚拟列表；本轮不引入新的渲染库。

## 实施边界

1. 调整共享主题 token、详情头、Software 详情、Settings 和 Workflow 编辑器。
2. 共享 token 会自然改善其他管理页，但不重做词典、探针的业务布局。
3. 不增加无执行意义的设置、导航层级、动画、阴影或装饰性图形。
4. 不更换 Nuxt UI，不引入新的状态管理或数据模型。
5. 任何布局改动不得破坏 Esc、未保存保护、表格分页、搜索和既有可访问名称。

## 验收

- Software 列表/详情、Settings、Workflow 基础配置和词典配置具有可识别的同一页面骨架。
- 深色与浅色共享相同表面角色；浅色 Table 纯白内容面边界清楚，暗色主按钮不过度发亮。
- Workflow 侧栏和内容区只有一次明确分隔，活动项不形成第三块孤立底色。
- 1440×900 与 960×640 下，详情头不跳动，内容滚动不带动框架，主操作不被裁切。
- 键盘焦点、悬停、选中、禁用、错误和空态保持可见；颜色不作为唯一状态信号。
- Desktop build、完整 Playwright、明暗主题视觉截图和最终 UI detector 全部通过。
