# VGUI 本地化查询的生命周期契约

2026-09-08；只读公开源码调查，未操作目标或修改功能代码。结论是候选方案约束，不是目标兼容性证明。

## 已证实的边界

- **不能只 Hook Find。** l4d2 SDK 的 `TextImage::SetText(char*)` 通过 `FindIndex` → `GetValueByIndex` 获取真正显示的文字，并复制到自身 UTF-16 缓冲；绘制读取副本。[维护者源码](https://github.com/alliedmodders/hl2sdk/blob/l4d2/vgui2/vgui_controls/TextImage.cpp#L89)
- `Label::SetText(char*)` 内的 `Find` 可能只是计算快捷键；因此 Find 命中不等于显示文字被替换。SetText 会使布局失效并请求重绘，但不是无条件调整控件大小。对话变量变化另走 `ConstructString(index, variables)`，再 SetText；不能假设格式化实现必经虚调用 GetValueByIndex。[Valve Label](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/Label.cpp#L267)
- **改表或替换返回指针不会自动改写已有 TextImage 副本。** 即使重绘、重新布局，副本仍是旧值；需要再次 SetText、可靠的控件重建，或覆盖其初始化。直接 Unicode 文本也可能根本不经过本地化查询。[Valve TextImage](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/vgui2/vgui_controls/TextImage.cpp#L182)
- l4d2 头文件声明 `VGUI_Localize004`：Find、GetValueByIndex 返回可写指针，没有声明分配、释放、有效期或线程安全；SetValueByIndex 位于本地化编辑器专用部分，ReloadLocalizationFiles 标为开发用途，未承诺控件刷新。[接口](https://github.com/alliedmodders/hl2sdk/blob/l4d2/public/vgui/ILocalize.h)
- SDK 2013 的 005 调整了虚函数集合，转换/部分格式化改为静态方法，并增加 FindAsUTF8；不能共享 004 的槽位表。[005 接口](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/public/tier1/ilocalize.h)

## 未知项与工程选择

本轮未取得实际目标本地化表实现；公开 `tier1/ilocalize.cpp` 是转换和格式化辅助实现，并非表的 Find/SetValueByIndex/Reload 实现。因此**表是否复制 SetValueByIndex 输入、写入/重载是否使旧指针或索引失效、内部是否通知 UI，均不能作保证**。[公开实现范围](https://github.com/ValveSoftware/source-sdk-2013/blob/master/src/tier1/ilocalize.cpp)

建议先做 Find 与 GetValueByIndex 的只读调用观察，保留格式化调用分类；不原地覆盖未知容量的返回缓冲，不以 ReloadLocalizationFiles 充当恢复操作。若以后采用返回值替换，应由 Adapter 保有稳定、终止正确的 UTF-16 存储；原始表保持不变，停用仅使后续查询返回原值。未知调用者可能保留指针，不能在换代或停用时立即释放旧译文；稳定存储也不等于已经证明所有调用者的可写语义。

**恢复分两层验收：**查询恢复，以及控件副本恢复。仅解除 Hook 不会抹去已复制到控件的译文。若没有可信刷新入口，明确要求重建/重启；“支持恢复”不能只依据查询日志。

**重启必须有时序条件：**Adapter 与字典必须在目标 UI 第一次查询并复制文本前就绪，或在之后可靠地重建该 UI。用户先进入主菜单、再附加，与当前晚附加在缓存问题上相同；轮询进程出现后注入也不能证明抢在初始化前完成。下一阶段应先验证可实现的生命周期边界，再决定是否增加启动接管能力。

## 下一次实验的停止条件

验证入口确实提供目标完整文字 → 在 Hook 已生效后新建的控件出现译文 → 换代后重建显示新版 → 停用后重建显示原文。若现有控件不再查询，立即归类为初始化覆盖/刷新未解决，停止添加绘制补丁；不把它误判成翻译决策失败。
