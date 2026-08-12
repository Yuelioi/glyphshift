# 成熟设计系统校准基线

这些资料用于校准 Glyphshift 自有设计合同，不表示直接采用或仿制任一外部设计系统。

## Windows 字体与布局

- [Fluent 2 Typography](https://fluent2.microsoft.design/typography) 把 Windows Caption 定义为
  12px/16px、Body 定义为 14px/20px，并强调稳定的字号语义与对比度。Glyphshift 可以保持高密度，
  但应把大量 9px/10px 文字视为需要实测可读性的产品决定，而不是自然默认值。
- [Fluent 2 Layout](https://fluent2.microsoft.design/layout) 使用 4px 为主的 spacing ramp，并要求在
  不同窗口宽度中保持分组、基线和信息关系。审计时应检查同类页头、工具栏、表单和详情动作是否使用
  同一节奏，而不只检查单个数值是否合法。
- [Fluent 2 Accessibility](https://fluent2.microsoft.design/accessibility) 要求结构与 heading 顺序可预测、
  键盘焦点不丢失，并在文本缩放时避免裁切。桌面基准仍是 960×640 与 1440×900，但需要把系统缩放和
  长文案视为窄窗口压力，而不是只按 CSS viewport 判断。

## 工具栏与动作层级

- [Fluent 2 Toolbar](https://fluent2.microsoft.design/components/web/react/core/toolbar/usage) 要求工具栏保持
  单行、空间不足时进入 overflow；相关动作应成组，危险动作与普通动作分离，图标动作同时提供 tooltip
  和 accessible label。
- [Carbon Data Table](https://carbondesignsystem.com/components/data-table/usage/) 建议表格工具栏只承载全局
  动作且最多约五项；选中行后使用批量动作栏。少于三项的行级动作可直接展示，更多动作宜进入 overflow，
  以减少每行持续噪声。

## 数据集合、状态与分页

- [Cloudscape Table View](https://cloudscape.design/patterns/resource-management/view/table-view/) 区分空集合、
  无搜索结果、加载与错误状态，并要求无结果时给出清除过滤或恢复动作；分页适合需要定位与返回具体页面的
  大集合。
- [Cloudscape Table](https://cloudscape.design/components/table/) 明确支持服务端过滤、分页、空态、加载、
  行内编辑和列偏好。Glyphshift 的 Probe 服务端分页方向正确，审计重点是不同集合页是否用同样的状态语法。

## 对本项目的使用边界

- 外部字号与触控规格不是照搬指标；Glyphshift 是鼠标/键盘优先的 Windows 高密度工具。
- 外部系统只帮助识别可预期性、可读性和动作层级问题；颜色、品牌、产品术语和信息架构继续由
  `DESIGN.md` 决定。
- 任何建议必须同时经过本项目 Playwright 证据、现有交互合同和真实桌面任务验证。
