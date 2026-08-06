# 前端状态与视图 seam

Status: Complete

## Goal

保持桌面 UI、浏览器确定性 fallback、Tauri command、i18n key 与 Playwright 用户合同不变，复核并收敛
`useWorkspace.ts`、`CaptureView.vue` 和本地化 catalog 的变化原因。

## Baseline

- `useWorkspace.ts` 800 行，单一公共 composable 内混合共享模型持久化、Workflow、Dictionary 与 Software
  三类 action；只有 `App.vue` 消费完整 facade。
- `CaptureView.vue` 920 行，主要由 Probe 列表/详情状态与模板组成；其中自绘滚动条的 DOM 测量和 pointer
  capture 生命周期可独立于 Probe 业务。
- `en-US.ts` 767 行、`zh-CN.ts` 761 行，分别是完整 locale catalog；英文通过 `MessageShape<typeof zhCN>`
  编译期校验 key/嵌套结构。

## Plan

- [x] 把 Workspace 共享状态/持久化与 Workflow、Dictionary、Software actions 组织为私有 feature modules，
  保持 `useWorkspace()` 返回字段和唯一调用者不变。
- [x] 提取 Capture 自绘滚动条 composable；保留 Probe polling、paging、view-state 和模板业务在组件 owner。
- [x] 对两份 locale catalog 做结构/key 审核；若没有独立加载/发布 seam，则记录保留整 catalog 的决定。
- [x] 运行生产构建、组件结构测试与完整 Playwright；审核生成文件、用户可见行为和隐私边界。

## Decisions

- Workspace 按业务 capability 分 module，不创建泛化 `utils`/`store`；共享 module 只拥有持久 model 和跨能力
  snapshot/message 原语。
- Capture 不拆成大量 props/emits 子组件：列表、详情、创建与设置共同围绕一个 `useProbeRuns` selection owner，
  强拆会扩大浅 interface；只有 DOM 滚动生命周期具备独立输入/输出。
- Locale 暂以一个语言一个 catalog 为候选最终结构；拆成几十个 namespace 文件会增加 translator 导航与
  合并成本，除非 key 审核发现不一致或构建需要按 namespace 延迟加载。

## Result

- `useWorkspace.ts` 从 800 行降为 51 行稳定 facade；共享 state 96 行，Workflow 296、Dictionary 227、
  Software 238 行。原 45 个返回字段集合与 42 个具名函数的 TypeScript AST/打印结果均精确一致。
- `CaptureView.vue` 从 920 行降至 871 行；88 行 `useCaptureScrollbar` 独占 DOM 测量、pointer capture 和
  resize listener，Probe polling/paging/view-state 仍保留在业务 owner。
- 中文和英文 catalog 各有 661 个叶子 key，集合完全一致；`MessageShape<typeof zhCN>` 继续提供编译期
  shape 检查，因此保留一语言一文件，不引入 namespace 聚合层。
- `vue-tsc --noEmit`、生产构建和完整 Playwright 60/60 通过。构建生成的 declaration 文件已恢复到
  checkpoint；只保留结构改动。Vite 仍报告既有的单一大 chunk 警告，本切片没有改变加载边界。

## Review

- Standards：feature module 以业务能力命名，共享 state 只提供 snapshot/message/persistence 原语；没有新增
  `utils`/`common` 或多层转发。Capture composable 的 listener start/stop 仍在原 mount/unmount 位置成对调用。
- Spec：browser fallback、Tauri command 名、Workspace facade、Probe 5000-entry 后端分页、view-state key、
  自绘滚动条 DOM/test id 和所有 i18n key 保持；完整用户合同证明列表、编辑、设置和多语言行为未变。
