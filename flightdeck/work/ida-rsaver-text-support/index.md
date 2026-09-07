# IDA 与 R-Saver 汉化定位

Status: Finished

## Goal

定位 IDA 与 R-Saver 不能汉化的原因，修复通用技术入口或连接流程后交给用户在 App 验收。

## Current

IDA 9.2 加载带 `QT` C++ 命名空间的 Qt 6.8.2 Widgets；标准 QString 和 QPainter 导出均缺失，命名空间版本存在。已修复 Qt Painter，按同一命名空间解析 QString、四个 drawText 重载及 QWidget 重绘辅助接口；不按软件品牌硬编码，也未放宽 Qt Quick 版本门控。3 项定向回归及严格 Clippy 通过。

IDA 原生适配器在授权目标中激活成功，菜单文字的中文首代、第二代与停用恢复通过。R-Saver 9.5 在管理员诊断上下文中使用现有 TextOutW 适配器，同样完成俄文标题到中文的两代替换和恢复。两个目标恢复截图的阈值差异像素均为零；只证明所选 UI 标签，不扩展为整软件覆盖。

R-Saver 初始进程为高完整性级别且未加载 Glyphshift Runtime。原生接口不是阻塞点；用户原先 App 中的具体失败表现尚未回复，不能仅由高权限宣称已确定唯一根因。下一步用户应确认 Glyphshift 管理员权限、选择 TextOutW 并使用实际俄语作为源语言。没有操作任何恢复、扫描或写盘功能。

包含 Qt Painter 修复的新 App 已通过规定入口同步构建并启动，生产 Loader 对 11 个 Adapter 及打包副本校验通过；已读取进程令牌确认当前 App 拥有管理员权限。两个目标均恢复原文并保持响应。用户测试前应重启目标，以清除旧 Runtime 和本轮临时诊断 DLL。

用户已明确确认 IDA 与 R-Saver 在新 App 中均可汉化，并要求提交。R-Saver 历史失败的唯一原因未被独立证明，不将本次验收反推为某个单一原因。

## Next

None

## References

- [工作范围](context.md)
- [文字适配器验收原则](../../knowledge/rendering/runtime-text-adapter-validation.md)
