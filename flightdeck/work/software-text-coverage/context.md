# 范围与约束

- 用户目标是完整采集并能原位翻译，不能停留在提取。框架分支按版本、编码、调用约定和缓存生命周期识别，不按游戏名或固定地址打补丁。
- Houdini 当前报告的现象：部分工具栏标签与悬停提示正文保持英文，探针里找不到对应内容；已有部分界面能翻译。不预设为 Qt 适配器缺陷或字体缺失。
- Vovious 的静态 JUCE 6.1.6 已证明两个 canonical `GlyphArrangement` 入口可唯一恢复，但计划要求的 `addJustifiedText` / `addLineOfText` callable 边界缺失，当前 canonical 三入口 profile 因此 fail-closed；只有新的通用 seam 或其他精确 JUCE profile 才重开，不按软件名或固定地址补丁。Blender 的 Python/BLF、自绘 Mod Manager 仍需各自建立可重复字符串入口证据；运行库存在或导出存在不足以证明可替换。
- 同时覆盖已有适配器的具体分支风险：DirectWrite 复用旧 layout 后重新启用、Qt Quick AutoText 候选格式、Unity SetText 重载，以及矩阵记录的其他边界。
- VGUI 与 CatSystem2 继续保持实验或未完整验收。每个新增分支都验证完整正文、两代译文、可见刷新和停止恢复；合成合同与实机证据分开记录。
- 不运行归档 UIA/OCR。原始截图、捕获文字、路径、进程信息和实验脚本只存本地忽略目录，恢复时重新读取目标状态。
- 最新 App 必须经指定 review 脚本同步构建桌面壳和 Runtime Bundle。局部原生实验的驻留工件可能要求目标正常重启后再进行 App 验收。

- Qt translation-service 当前以 Wireshark 4.6.8 / Qt 6.10.3 为 production 动态真实代表：精确 `QCoreApplication::translate` profile 已完成真实 V2 context 观测、`plural_n` 采集并 fail-open、placeholder 保持合同、三代原生窗口标题刷新与停用恢复。QGIS LTR 3.44.12 / Qt 5.15.13 是新的 research 真实目标：首轮实机证明大部分菜单可替换，同时暴露已创建嵌套 menu action / tooltip 的缓存刷新缺口；真实 Qt 5.15.13 本地红例与目标 Qt5Core 二进制证明现已闭环 `qMalloc -> UCRT malloc` / `QEvent deleting destructor -> 同一 UCRT free`，`research-qt5` 因而可安全 post queue-owned `LanguageChange`，同一红例已转绿。默认 production native build 仍只接受 Qt 6.10.3；Qt 5.15.13 继续 research-only 并等待第二代更新与停用恢复复验，Qt 5.15.18 仍 future-call-only。共享 native text V2 继续承载 `context/disambiguation/n` 作为观测证据；Dictionary 保持 schema v3 / source-only，同 source 非复数调用共享一条译文，context/disambiguation 可作为 Probe/AI 提示但不进入词典 key。plural 暂时只观察不替换，Qt replacement 必须保持 placeholder 多重集合；旧 source-only host 对非复数 contextual / disambiguated 调用可按 source 决策，对 plural 与未知负 `n` 继续 fail-open。

- 软件调查结论按 `references/software/<软件名>.md` 保存，每款软件一份；Work 首页只作导航与当前任务交接。原始日志、截图、地址、二进制和实验脚本留在本地忽略目录，不复制到研究结果中。暂缓或取消的调查明确记录，后续恢复前重新确认范围。
