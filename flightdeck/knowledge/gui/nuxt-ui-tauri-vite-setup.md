# Nuxt UI 与 Tauri 前端约定

GUI 使用 Vue 3、Vite、Nuxt UI 4、Tailwind CSS 4 和 Tauri 2。Vite 固定使用 `1420` 且 `strictPort`；Tauri 的 `devUrl`、`frontendDist`、前置 build 命令必须与之保持一致。

## 组件与内容约定

- 应用根包在 `<UApp>` 中，图标统一使用 `i-tabler-*`。
- 界面文案使用中文；颜色优先使用 Nuxt UI 语义 token，避免散落裸色板。
- 页面和组件样式以 Tailwind 工具类为主。Vue 单文件组件默认不写 `<style>`；只有动态值、
  浏览器能力或 Tailwind 无法可靠表达的规则才允许少量 CSS，并在代码旁说明原因。
- `assets/css/main.css` 只承载 Tailwind/Nuxt UI 引入、设计令牌、根元素、滚动条与
  reduced-motion 等真正全局的规则，不存放页面选择器或组件样式。测试钩子可以保留语义
  class，但不能依赖它承载视觉规则。
- 下拉选择优先使用 `USelect` / `USelectMenu`，并通过 item slots 呈现状态；除非明确需要浏览器原生行为，不使用裸 `<select>`。
- `USelectMenu` 的 item value 不能是空字符串；“未选择”必须使用内部 sentinel，再在 model 层映射为无选择状态。
- Nuxt UI primitive 负责控件、表格和浮层；项目组件只封装重复的产品语义。管理型页面统一复用
  页头、搜索/筛选/显示列/批量操作/分页框架、表单弹窗与确认弹窗，不在各页面复制这些结构。
- Nuxt UI 的密度、颜色和交互变体集中写入 AppConfig。业务 Vue 不直接写原生
  button/input/select/textarea/table/dialog，不直接导入图标 Vue 组件，也不手工携带主按钮视觉类；
  用源码扫描合同阻止这些做法回流。
- sticky `UTable` 与 `UPopover`/`UDropdownMenu` 重叠时，浮层 content 必须使用统一且高于表头的
  z-index；否则浮层虽然可见，实际点击仍可能被 sticky 表头截获。
- `UModal` 的 overlay 与 content 必须显式位于菜单和 sticky 表格之上，并用实际重叠点的
  `elementFromPoint` 回归验证；只检查可见性无法发现底层表头穿透弹窗的层级问题。
- 表格“显示列”使用 `UDropdownMenu` checkbox item：文字在左、选中勾在右，并用独立分组提供
  “恢复默认列”；不要用 `UPopover` 包一列 `UCheckbox` 模拟菜单。真正的表单 checkbox 仍在
  AppConfig 统一覆盖为 `items-center`。
- 管理表分页由共享框架完整提供首页、上一页、页码、下一页、末页和每页数量，页面只维护
  `page`/`pageSize` 状态。长列表滚动条在全局根规则统一使用细滚动条、透明轨道、圆角 thumb，
  并隐藏 WebView 的原生 scrollbar button。
- `useHook()` 是 IPC 和轮询的单例入口；`useTheme()` 同样保持单例。
- 可编辑行的本地状态使用完整复合身份键，不能只用英文原文，因为同一原文可能属于不同分类或语境。
- 大列表使用 `@tanstack/vue-virtual`；虚拟列表的 count 必须随过滤结果响应更新。

## 主题与无边框窗口

Tailwind 4 会裁剪未被直接引用的自定义 theme 变量。Glyphshift 的 `signal` 色阶必须使用
`@theme static`，在 `vite.config.ts` 中声明为 Nuxt UI primary，并显式连接
`--ui-color-primary-*` 与 `--ui-primary`；只通过运行时颜色名映射会出现“编译成功但
primary 无颜色”。

无边框窗口的拖拽区域和窗口按钮必须是不同元素。窗口最小化、最大化、关闭和拖拽需要在 Tauri capabilities 中逐项授权。

## 验证

```powershell
cd gui
npx vue-tsc --noEmit
npm run build
npm run test:visual
```

视觉验证使用 Playwright 启动 Chromium，并 stub `window.__TAURI_INTERNALS__.invoke`，为四个路由提供稳定假数据后分别检查暗色/亮色截图。类型检查不能替代视觉检查；至少验证 `--ui-primary` 非空、主要按钮背景色正确、长列表没有全量渲染。

完整发布验证由 [便携分发 smoke](../../work/gui-productization/slices/portable-distribution-smoke.md)负责。
