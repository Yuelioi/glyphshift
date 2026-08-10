# 阶段计划

1. [x] 建立占用冲突与重叠来源的最小红灯合同，确认当前错误和到达顺序依赖。
2. [x] 在 Runtime 所有权 seam 增加稳定冲突语义，并贯通 Tauri `CommandError` 与中英文 UI 提示。
3. [x] 在 Capture 聚合 seam 引入 Descriptor 驱动的来源权威，保证可写回优先、观察器兜底与完整
   provenance。
4. [x] 运行受影响 Rust 合同、Playwright 页面合同、生产构建和本地 AfterFX 定向复验；清理临时诊断并
   更新 Work。
