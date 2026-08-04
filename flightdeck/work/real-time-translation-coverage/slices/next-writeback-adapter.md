# 下一种实时写回 Adapter 选择

Status: In progress

## Outcome

选出一个能显著扩展 Windows 桌面软件实时翻译覆盖的文字路径，并固定合成与授权真实目标的可见验收；
不把只读采集、成功注入或架构完整度当作选择理由。

## Delivery

- [ ] 盘点正式 Runtime Bundle 与现有原型的观察、写回和真实目标证据。
- [ ] 对仍未覆盖的主要 Windows 文字路径列出可拦截入口、原文取得方式、写回方式和停用恢复风险。
- [ ] 选择一个具有真实目标、跨软件复用价值和有界实现范围的候选；明确否决其他候选的当前理由。
- [ ] 固定首个合成合同与授权真实目标 smoke，不包含本机路径或真实原文。

## Current

当前生产写回集中在 GDI、USER32 DrawText 与 GDI+。Direct2D `DrawText` 原型可以在合成 render target
写回，但授权目标没有命中；Console 与 UIA 只能采集。已知的大类缺口包括 DirectWrite/Direct2D 的
retained text、WPF/WinUI、Chromium/CEF/Electron、Qt 混合后端和 GPU 画布，但框架名称本身不能证明
具体文字路径。

## Next

检查 Bundle Descriptor、现有 Adapter 合同和本地授权测试入口，形成简洁支持矩阵；随后只针对最有
真实验证条件的两到三个候选查阅官方 API 文档，选择一个进入合成原型。
