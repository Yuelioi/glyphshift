# Unity Mono 中文字体回退

## Current

Aooni 菜单中文方框已通过可恢复对照复现，确认本次原因是当前 TextMeshPro 字体图集缺少目标字形。
实际菜单使用 `NotoSans_SDF`，原字体覆盖英文、不覆盖测试中文；加载的其他 TMP 图集也未覆盖测试中文。
独立测试会话不接管用户 Runtime，实验前确认没有活动 Glyphshift Runtime 模块。

在同一个可见对象上完成：原文基线 → 原字体下中文方框 → 新字体下首代中文 → 新增字形的第二代中文 → 原文恢复。
机器断言通过：原字体缺字、新图集添加成功、两代字形覆盖、两代可见文字像素变化、恢复后的文字区域像素与基线一致、
原字体与原材质引用恢复。人工查看确认实际字图是中文，非另一种占位符。测试结束保留原始菜单状态。

这只是主线程 API 实验，正式 Native Adapter 尚未实现字体回退，当前 App 仍不能据此声明问题已修复。
原始截图、字体路径、运行实例、探查与复跑脚本全部保存在 local-test；不作为仓库构建的必需资源。

## 已验证的入口

- 标准 UI 主线程调度后读取对象与字体，使用 Mono 元数据确认参数类型，不能只凭方法名和参数数量选择重载。
- `Font.CreateDynamicFontFromOSFont` 创建的系统动态字体在本次目标中被 TextCore 拒绝，`LoadFontFace` 返回非成功状态，
  `TMP_FontAsset.CreateFontAsset` 返回空。此路线不能冒充已解决。
- `Font` 的字符串构造函数接受字体文件路径；对目标可读取的字体文件创建 Font 后，TextCore 成功加载，
  `TMP_FontAsset.CreateFontAsset(Font)` 与 `TryAddCharacters(string, out string, bool)` 成功生成两代所需字形。
- 修改测试对象的 font 后中文可见；恢复时同时还原原 font、fontSharedMaterial 和原文，只有还原文字不足以清理显示状态。
- 实验创建的字体资源只在测试进程内驻留；正式实现必须补资源预算、主线程回收与多次启停合同，不能直接照搬实验寿命。

接口依据：[Unity 字体绑定](https://github.com/Unity-Technologies/UnityCsReference/blob/master/Modules/TextRendering/TextRendering.bindings.cs)、
[TMP FontAsset API](https://docs.unity3d.com/Packages/com.unity.textmeshpro@3.0/api/TMPro.TMP_FontAsset.html)。
目标元数据与实际像素才是本次版本行为的验证依据，不由最新在线源码推导所有 Unity 版本。

## Next

按语言的用户字体配置已落在 AppSettings 的 `languageFallbackFonts`。下一步将此配置接入公共字体选择与 Runtime publication，再由具体引擎执行缺字回退；目前只有设置和持久化，没有运行时消费。不得在 Unity 内硬编码某个语言必须使用某个字体。

将上述窄能力接入 `unity-mono-standard-ui-native` 的主线程 writeback 生命周期：
先检查原字体及已有 fallback 覆盖，仅在译文缺字时准备自己的字体资源；保存原 font/material，字体由目标修改时不得覆盖目标的新值。
使用原文恢复标记区分“写入译文”和“撤销译文”，不要在恢复过程中再次补字体。

正式接入还需验证：字体查找与缺文件透传、图集预算与新增字形、无缺字文字保持原字体、对象回收、非主线程 setter、
两代 publication 与停用恢复、反复启停的资源回收。TMP 与 uGUI 分别声明和验收，不能把本次 TMP 实验算作 uGUI 已通过。
之后通过配套 Runtime Bundle 实机复核，再启动新版 App 给用户验证。
