# 活动适配器字符串分支覆盖

本表审计当前 13 个产品适配器的实现入口，并汇总已有验证记录。入口范围是源码事实；“未覆盖”不等于已复现缺陷。合成合同、框架像素和真实软件结果分开理解，不能用“采集到文字”替代可见替换与停止恢复验收。

| 适配器 | 当前字符串入口 | 替换 / 刷新 / 恢复边界 | 下一项缺口 |
| --- | --- | --- | --- |
| GDI ExtTextOut | ExtTextOutW；字距、裁剪及部分字形索引 | 绘制时替换；依赖目标重绘，下一次原文绘制恢复 | ANSI、未知 glyph-index、逐字与轮廓路径不等于完整正文 |
| GDI TextOut | TextOutW | 同上；有首批实机验收 | TextOutA 和自绘缓存须另查 |
| USER32 DrawText | DrawTextW / DrawTextExW | 矩形绘制路径；有首批实机验收 | ANSI、控件自身缓存及非文本绘制 |
| GDI+ | GdipDrawString | 系统像素、启停与 Runtime 合同已验证 | 路径文字、纹理缓存不从 DrawString 推导 |
| DirectWrite | CreateTextLayout → D2D DrawTextLayout | 单格式 layout；有像素和恢复合同；刷新入口为空 | 当前停用清除关联，复用旧 layout 再启用需补合同；晚附加、其他 layout 创建入口及 glyph run |
| Qt Painter | 四种 QPainter drawText；支持的 MSVC 动态 Qt 5/6 与命名空间 | 绘制替换和 QWidget 重绘；两代与恢复合同 | 静态链接、其他命名空间、QStaticText、富文本与路径分支 |
| Qt Quick | Qt 6.8.3 x64 QQuickText | GUI 线程对象修改和恢复，已有合成与限定实机证据 | 输入框、富文本、自绘；AutoText 原文/译文格式资格需补回归 |
| GTK 3 / Pango | gtk_render_layout | layout 副本、局部属性透传；原生 ABI 合同 | 直接 Pango/Cairo、GTK 4、预渲染缓存 |
| raylib | 动态 raylib 5.5 DrawTextEx | 动态字形与帧边界资源回收合同 | 静态链接、其他绘制/代码点入口是否汇入、RenderTexture 缓存 |
| Unity Mono | TMP_Text.set_text / UnityEngine.UI.Text.set_text | 主线程 setter、快照、换代恢复合同 | SetText 重载是否绕过 setter，UI Toolkit / NGUI / IL2CPP 不在该实现内 |
| MonoGame | 标准 DrawString 重载 + MeasureString | 六种调用、实际 CLR/GPU 合同、换代恢复和动态缺字 | 自绘纹理、非受支持 CLR/签名；长会话字体配额与复杂 shaping |
| VGUI 本地化 | Find / GetValueByIndex，已识别标准 Label | 限定实机标准标签刷新/恢复通过；仍未完整验收 | 格式化文本、自定义 Label、其他版本与 Garry’s Mod 待实机 |
| CatSystem2 | UTF-8 对象；新增经典 ANSI 结构 profile | UTF-8 正文/历史及经典当前句；合成刷新恢复，经典实机两代及恢复通过 | 经典历史、其他 ABI/版本/代码页、混合对象及字体回退 |

来源：各 Adapter 实现目录、产品 catalog、[既有边界审计](adapter-audit.md)、[扩展合同](expanded-adapters.md)、[经典分支 Slice](../slices/catsystem2-classic.md)。这里只列当前产品项，不纳入归档 UIA 或尚未进入 Bundle 的实验采集器。

## 排序

1. 已有红例优先：经典 CatSystem2 当前句已补齐，下一步用户 App 验收。
2. 已有实现的具体生命周期/格式风险：DirectWrite layout 再启用、Qt Quick AutoText、Unity SetText 重载分别建立独立失败合同，再修实现。不能通过扩大类型匹配或无界枚举宣布覆盖。
3. 新框架：JUCE 适合作为下一项；Blender 需独立桥接，Mod Manager 的实际字符串入口仍待定位。

每项新增分支都应记录：完整正文入口、候选编码、调用约定、写回所有权、显示缓存重建、字典次代更新、停止可见恢复。未经后四项验证的入口只算采集证据。
