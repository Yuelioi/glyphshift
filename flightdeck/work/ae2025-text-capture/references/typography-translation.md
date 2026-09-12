# AE 2025 统一排版的译文显示

用户试用采集修复版后确认：已有原文和字典译文，但 AE 标签、效果控件及脚本面板仍显示英文，重启目标也不改变结果。本阶段将验收推进到实际中文像素。

## 根因与处理

真实目标的 DirectWrite 诊断已经匹配并生成替换决策；全文统一的 Typography 仍被 `layout_is_uniform` 的空对象条件拒绝。重启无法改变这项格式检查。

现有 Adapter 现在允许全文统一的 Typography，并在绘制时从当前目标布局复制格式。原先保存在创建记录中的 TextFormat 和尺寸不再作为绘制时的格式依据；记录只保留原文及关联序号。新布局使用当前尺寸，复制字体集合、字体、字号、locale、装饰、Typography、段落设置、间距、字偶距、字体回退与可用版本的布局属性。字符范围按译文 UTF-16 长度重建，目标布局保持不变。

局部字号、局部 Typography、局部字符间距、inline object 和 drawing effect 仍按原样绘制并保持观察。如果准备译文失败，在调用原始绘制前释放替换作用域，避免未采用的决策影响下层绘制。没有新增 Adapter、ABI 或 Adobe 私有分支。

恢复完整用户工作流时还复现了脚本面板标题残影：AE 释放非空布局后，将同一地址用于空文字布局；原实现只记录非空创建，没有移除空创建所复用地址的旧原文。原先看不见的空布局因此画出了旧标题的译文。现在每次成功创建都更新关联：可捕获原文时覆盖，否则移除该地址的记录。这个问题与是否带 Typography 无关，但实际中文显示让零宽布局的残影更明显。

接口依据：布局允许设置指定字符范围的 Typography；创建布局只接受给定的格式与边界，因此不能把创建时保存的格式视为当前全部状态。[SetTypography](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextlayout-settypography)、[CreateTextLayout](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritefactory-createtextlayout)。实际迁移正确性由以下像素合同验证。

## 验证

- 最小红例：通过 Native ABI 返回中文译文，Typography 布局仍画出原文；像素预期断言失败。修复后通过。
- 地址复用红例：真实 DirectWrite 释放带文字布局后复用地址创建零宽空布局；原实现在空画布上产生译文像素，修复后没有像素、观察事件或替换作用域。
- [复杂格式合同](../../../../crates/adapters/platform/native-host/tests/native_directwrite_capture_formatting_contract.rs)：每个架构 60 个组合，覆盖两个统一格式场景、三个局部格式场景、DC/位图目标、两种宿主和保持/替换/错误回调。额外在同一布局上验证下一代译文、移除译文及停用恢复。
- x64/x86 的复杂格式合同及既有激活合同全部通过；格式迁移模块定向 Clippy 无新增告警。
- AE 原生合成设置的 `3D Renderer`：第一代中文可见，第二代改变，停用后文本区域与原始截图像素完全一致。生产 Runtime 的两代匹配计数均非零。
- AE 自带 `Create Nulls From Paths` 面板的 `Points Follow Nulls`：相同的生产 Runtime、中文显示、更新和精确像素恢复链路通过。
- 两项真实可见性验收通过仓库 Playwright CLI 执行；临时字典和所有原始截图、日志只在本机忽略目录，未修改用户译文。
- 桌面和 Runtime 通过 `scripts/review-app.ps1 -UseUserData` 同源构建，生产 loader 验证 13 个 Adapter。独立合同切换回用户完整 Adapter Plan 时需重启目标，再恢复用户工作流。
- 包含空布局修复的最终审阅版再次通过完整用户工作流验收：按钮译文可见，标题下方空白区域与无翻译基准的像素精确一致，残影消失。实际目标、审阅包和合同产物的 Adapter 哈希一致。

结论限定在上述统一排版与代表性真实表面；不据此承诺所有局部混排、所有效果或所有插件均可替换。较长译文仍受目标原始布局边界约束。

## 代码

- [DirectWrite 原生入口](../../../../crates/adapters/implementations/native/directwrite-native/src/lib.rs)
- [当前布局格式复制](../../../../crates/adapters/implementations/native/directwrite-native/src/uniform_layout.rs)
- [合成绘制宿主](../../../../test-support/windows-host/src/windows/render.rs)
