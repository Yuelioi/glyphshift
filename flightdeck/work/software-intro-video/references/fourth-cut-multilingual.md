# 第四版方向：通用软件与多语言

用户要求先确定真实软件候选并做本地验证，再用于视频展示。

- 第一幕用当前适配器名称和通用软件关键词，不再突出 AE、Adobe、Houdini 或 CG 软件。
- 第二幕使用独立设计的模拟桌面软件界面，标注模拟演示；同一布局依次显示英文、中文、日文、韩文。左侧软件名只使用通过本地验证的常规工具。
- 第五幕改为多语言译文，不以英文到中文作为唯一关系；其余产品截图也移除 AE 演示名称。
- 软件候选要区分“语言缺口有宣传价值”和“代表适配器但自带多语言”；Notepad++ 是后者，已有 DirectWrite 编辑区与传统菜单验证记录。
- 首批候选：Process Explorer（Win32/GDI 候选）、Fiddler Classic（WinForms，GDI/GDI+ 待辨别）、Wireshark（Qt Widgets 候选）、Notepad++（DirectWrite 已有记录）。GTK 3 可另用 Geany 作技术对照。
- 以上新增候选不是支持名单。每款先核对原生语言设置、动态库与实际命中；依次验证多语言可见替换、字典更新、字体缺字、停用恢复，才进入视频品牌展示。不要仅因 WinForms 推断 GDI+ 命中。

参考：
- [Process Explorer 官方页面](https://learn.microsoft.com/en-us/sysinternals/downloads/process-explorer)
- [Fiddler Classic 多语言请求](https://feedback.telerik.com/fiddler/1361475-when-to-support-multiple-languages)
- [Notepad++ 界面语言设置](https://github.com/notepad-plus-plus/npp-usermanual/blob/master/content/docs/preferences.md)
- [Wireshark 语言设置文档议题](https://gitlab.com/wireshark/wireshark/-/issues/19197)

## 第四版制作决定

用户已手动试用并确认上述候选均可用，授权继续视频，不需要再扩充软件。此为用户实测反馈，不写成自动化全面验收。

- 沿用自主修改、产品暗底蓝色视觉及 1080p30；第二幕延长至 12 秒，四种语言各停留约 3 秒，全片 64 秒。
- 第一幕 22 个关键词改为五款常规软件、当前适配器与功能词；不出现 AE、Adobe、Houdini 或 CG 词汇。
- 第二幕左侧为 Notepad++、Process Explorer、Fiddler Classic、Wireshark、Geany；右侧为原创通用设置窗口，保持布局，依次切换英文、中文、日文、韩文，固定可见标注 Simulated interface。
- 第三幕将 Creative apps 改为 Desktop frameworks。第四幕重新采集真实前端的中性 Desktop tools 演示配置，去掉 AE 词条和名称。
- 第五幕以 Open settings 为原文，显示日文、韩文、法文、西班牙文四份译文；独立示意排版，不冒充真实 AI 请求记录。保留服务选择与人工校对文案。
- 分镜顺序：0–8 开场，8–20 模拟界面，20–28 适配器，28–40 字体，40–52 AI 多语言，52–57 工作流，57–64 品牌。
- 复用既有两张镜头卡运动结构；模拟窗口和多语言译文为用户明确授权的原创说明视觉。配乐沿用，不承诺严格卡拍；导出带音乐和无音乐两版。
