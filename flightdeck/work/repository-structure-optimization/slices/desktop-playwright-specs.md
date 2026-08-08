# Desktop Playwright spec 拆分

Status: Complete

## Goal

保持 44 项用户可见桌面合同、默认产品模型、确定性 Tauri mock 和本机证据目录不变，把 1757 行单一 spec
按 Software/Management、Dictionary、Probe、Settings、Workflow 功能面组织，共享 fixture 只保留一份。

## Baseline

- `tests/desktop.spec.ts` 1757 行：前 171 行是共享 model/helper，172-175 是默认 beforeEach，后续 44 项测试。
- Playwright 固定单 worker、无重试，输出到 `local-test/desktop-playwright/`；截图也只写
  `local-test/evidence/`，不进入 tracked 资源。
- 现有测试从页面角色/test id/localStorage/Tauri mock 验证行为，不依赖 in-app browser。

## Plan

- [x] 提取共享 `productModel` fixture/model/helper，不注册跨 spec 的隐式 hook。
- [x] 拆分 management/software、dictionary、probe、settings/help、workflow 五个 spec。
- [x] 每个 spec 显式安装同一 default model beforeEach，保持隔离与可读性。
- [x] 用 Playwright `--list` 核对 44 项发现数，再运行完整 44 项和前端 build。
- [x] Standards/Spec 自审测试名称集合、fixture 单一来源、证据路径与用户可见断言。

## Result

- 删除 1757 行 `desktop.spec.ts`，提取 171 行 `fixtures/productModel.ts`。
- 拆为 management 220 行、dictionary 447 行、probe 506 行、settings 187 行、workflow 257 行。
- 五个 spec 各自显式注册相同 default model hook；共享模块只导出数据和 helper，不含测试生命周期副作用。
- 原 44 个测试名称集合与拆分后完全一致；Playwright 全 testDir 仍发现 60 项（另含既有 6 项 component
  system 和 10 项 visual）。

验证通过：拆分功能 spec 44/44（单 worker，约 1.9 分钟）、`npm run build`；所有截图路径仍在
`local-test/evidence/`，Playwright 输出仍在 `local-test/desktop-playwright/`。Vite 构建会
刷新 generated declaration，已将无关 `auto-imports.d.ts`/`components.d.ts` 还原为 `HEAD` 内容。

Standards 自审确认每个 spec 低于 600 行、import 只保留实际 helper、没有隐式跨文件 hook。Spec 自审
确认 44 个测试名逐项一致，测试体仅按原连续区段迁移，fixture model/helper 只有 export/type import 变化；
用户可见角色/test id、Tauri mock、localStorage key 与本机证据路径未变。

## Decisions

- 不使用共享模块顶层 hook，避免 ESM 缓存让多个 spec 的 hook 注册顺序隐式化。
- 按产品功能面拆文件；跨功能“workflow diagnostics”归 Workflow，“help/theme/admin/viewport”归 Settings。
