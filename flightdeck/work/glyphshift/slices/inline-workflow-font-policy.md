# Inline Workflow Font Policy

Status: Complete

## Outcome

删除顶级 Font Profile 和无法兑现的 Location 绑定。每个 Workflow Target 直接拥有至多一个字体
策略，包含有序字体候选和 `dictionary_matches` / `all_observations` 两种覆盖范围；默认只修改最终
词典集合命中的文字。

## Decisions

- 字体策略不是独立资产，不占顶级导航，也没有单独 CRUD、引用或删除生命周期。
- `dictionary_matches` 复用有序词典合并后的最终 source 集合；未命中字典的文字保持原字体。
- `all_observations` 对当前 Target 所选、且声明 FontSubstitute 的 Adapter 捕获文字全量换字体。
- 当前 Adapter 没有区域信号，Software Location、`main-ui`、全部位置和指定位置均不进入用户合同。
- Font Policy 只由 Workflow Target 持久化；Probe Run 不复制字体设置。

## Delivery

- [x] Workflow Core 以单一内联 Font Policy 编译两种覆盖语义。
- [x] Desktop Backend/Tauri 删除 Font Profile 资产接口与用户可见 Location。
- [x] Workflow `/3` 持久化字体候选和 coverage，不兼容读取旧 Font Profile Binding。
- [x] 桌面顶栏删除字体 Tab，Workflow Editor 提供一个紧凑字体策略区。
- [x] 中英文文案、浏览器预览模型、测试 fixture 和产品文档一致更新。

## Verification

- [x] Rust 合同证明默认只修改词典命中文字，全量模式仍可独立换字体。
- [x] Playwright 证明顶栏无字体、工作流无 `main-ui`，字体策略可保存两种 coverage。
- [x] Desktop build、全 Rust workspace、fmt、Clippy、架构检查和视觉检查通过。

## Current

Workflow Core、Desktop Backend、Tauri API v12、Workflow `/3` 与 Vue UI 已统一为内联 Font
Policy。27 项 Playwright、全 Rust workspace、Clippy、fmt、架构检查和本地截图检查通过。

## Next

继续 Dictionary Catalog 与生产 Runtime Bundle 交付；实机字体覆盖行为仍需授权目标软件验收。
