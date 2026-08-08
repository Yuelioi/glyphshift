# Probe 到 GDI+ 实时更新

Status: Stopped

## Outcome

证明用户在 Probe 中修改已绑定 Dictionary 的译文后，不需要停止探针或重启目标软件，下一代译文会
发布到现有 GDI+ Runtime，并在目标下一次绘制时生效；停止工作流后恢复目标原文。

## Delivery

- [x] Probe 行内编辑保存到绑定 Dictionary，并只向具有 `TextReplace` 的已选 Adapter 发布下一代预览。
- [x] 桌面应用契约验证首次预览与编辑后的第二代 Runtime Publication 内容和 Adapter 范围。
- [x] 确定性 Windows GDI+ 宿主验证首版替换、运行中第二代发布、诊断代次和停止恢复。
- [x] 授权真实宿主 smoke 支持由本机环境变量提供原文、首版译文、第二版译文和严格命中要求；不输出
  原文列表，不把本机信息写入仓库。
- [ ] 在干净重启的授权 AE 进程中完成首版可见替换、第二版可见更新与停止恢复。

## Current

应用层与 Runtime 层的确定性合同均已通过。授权 AE 的首版、第二版与停止恢复可见验收没有完成；该
旧尾项已明确停止，不再作为 Unity 引擎覆盖主线或恢复入口。未完成的真实验收不会被记作成功证据；
只有未来发布证据明确重新需要时，才另开有界 Work。

## Verification

- `cargo test -p glyphshift-desktop-shell --lib`
- `cargo test -p glyphshift-desktop-runtime --test windows_runtime_contract --no-run`
- 使用带确定性测试目标的本地 Runtime Bundle 运行
  `desktop_runtime_updates_gdiplus_translation_without_restarting_the_target`：通过。
- 真实目标的原始日志仅保存在 `local-test/evidence/`，不进入 Flightdeck。

## Acceptance

1. Probe 编辑后产生更高的预览代次，内容包含新的 `source → translation`，范围只包含已选写回 Adapter。
2. 同一 GDI+ 目标进程对同一原文先后使用两版译文；诊断分别记录两个代次的
   `Matched + Replaced`。
3. 停止 Runtime 后目标恢复原文。
4. 真实目标验收前检查是否残留旧 GlyphShift 模块；存在时要求重启，不能继续叠加注入。

## References

- [产品契约](../../../../PRODUCT.md)
- [AE 文字渲染路径](../../../knowledge/rendering/ae2020-text-render-map.md)
- [效果控件中文写回路径](../../../knowledge/rendering/effect-controls-cjk-source-route.md)
