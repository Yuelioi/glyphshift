# 可录制全局快捷键设置

Status: Complete; automated validation intentionally deferred at user request

## Outcome

用户可在设置页点击软件快速捕获或取词翻译快捷键控件后直接按下一组组合键。Glyphshift 立即探测
该组合能否注册为 Windows 全局快捷键；只有探测和真实替换注册都成功时才保存设置，系统占用、
两个 Glyphshift 动作互相冲突、无效输入或存储失败均保留原快捷键。

## Product boundary

- 不提供自由文本输入；录制只接受一次真实 `keydown` 组合，Esc 取消。
- 至少包含 Ctrl、Alt 或 Win 之一，Shift 可组合，避免占用普通字符输入。
- 探测不改变当前全局注册；保存时先注册候选，再释放旧快捷键，失败时回滚。
- 启动注册、两类等待提示、事件反馈和设置展示读取同一份设置。
- 快捷键是应用级设备偏好，不进入 Dictionary、Software、Workflow 或 Probe。

## Steps

- [x] 扩展 App Settings 与 Desktop 动态快捷键状态、探测命令及原子重绑定。
- [x] 设置页实现点击录制、组合键正规化、冲突/无效/检测状态和即时保存。
- [x] 同步中英文文案、产品合同、设计合同与 Desktop 错误映射。
- [ ] 定向 Rust、Playwright、构建与视觉检查；用户明确要求本轮停止测试，未执行且不冒充通过。

## Implementation notes

- App Settings 新增 `softwareCaptureShortcut` 与 `interactiveTranslationShortcut`，旧设置文件缺字段时
  仅使用默认值，不建立旧产品兼容分支。
- Desktop 启动注册、等待态返回与捕获事件都读取各自动态状态；全局 handler 通过注册 ID 分派。
- 探测采用短暂注册后立即释放；真正切换持有捕获状态锁，先注册候选、再释放旧键，存储失败时回滚。
- 设置页只有键帽按钮和内联状态；录制期间 Esc 取消，冲突后可直接继续按另一组组合键。
