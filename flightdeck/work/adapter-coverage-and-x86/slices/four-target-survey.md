# 四类已运行目标初查

## 范围与结果

用户授权先提交字典导入及图标改动，再粗查 Fluffy Mod Manager、Blender、《灰色的果实》与 Vovious，比较可实现性。前一轮功能提交为 `d4a629b`；提交前活动测试 511 通过、0 失败、25 忽略。

四个目标完成窗口截图、模块/PE 结构、局部二进制标记与短时只读 Frida 采样。脚本均已卸载并断开，不写译文，不终止目标。原始截图、地址、程序路径与进程信息仅留在本地忽略目录。

| 目标 | 已证实事实 | 下一步与边界 |
| --- | --- | --- |
| 灰色的果实 | x86；引擎资源布局为 CatSystem2；运行内存中存在 `.?AVkcFEScriptObjString@@`，未发现现适配器要求的 `.?AVkcFEScriptObjStringUTF8@@`。当前汉化程序含保护壳。 | 优先补 CatSystem2 旧字符串分支。必须从运行时结构推导 setter/show/tick 和编码，不把 UTF-8 配置强套旧对象；中文补丁可能改变原有多字节编码，需要正文换句与替换/恢复验证。 |
| Vovious | x64；可执行文件含 JUCE v6.1.6 及 juce TextEditor/Typeface 等 RTTI；JUCE 静态链接，无文字函数导出。 | 新建 JUCE 级适配器具复用价值，需识别完整字符串绘制/布局入口及对应对象生命周期。仅加载 DirectWrite 模块不能证明经过现有 CreateTextLayout 拦截。 |
| Blender | x64；Python 3.13 与 BLF 内部标记，截图为自绘 UI。 | 官方 Python 翻译注册和区域重绘 API 可用于受控概念验证；产品仍需稳定桥接、生命周期、字典换代与完整采集方案，不能把附加组件翻译注册等同全界面可覆盖。 |
| Fluffy Mod Manager | x64；OpenGL 自绘；无可用导出，未检出 Qt/JUCE/raylib/GLFW 常见标记。 | 字符串层入口尚未定位，不能把 OpenGL 字形/贴图直接视为完整文字。现阶段不优先做仅适用于此程序的地址补丁。 |

短时采样配合重绘请求，四个目标未触发所采样 GDI 文字入口；静态/缓存界面下的零调用只说明本轮无信号，不证明目标所有页面永久不走 GDI。尚未进行新句推进、完整字符串替换或刷新恢复实测。

## 推荐顺序

先补 CatSystem2 旧字符串分支，再研究 JUCE。Blender 有清晰官方入口，适合单独桥接 Slice；Fluffy 在找到可复用字符串层前暂后置。上述为初查后的工程判断，并非已实现或验收承诺。

## 一手资料

- [JUCE Graphics](https://docs.juce.com/master/classjuce_1_1Graphics.html)：绘制入口接收完整 String，包含单行、多行和 fitted text。
- [Blender 翻译指南](https://developer.blender.org/docs/handbook/translating/translator_guide/)：附加组件通过 Python 字典注册翻译；不能据此保证任意内建文字替换。
- [Blender Python translations](https://docs.blender.org/api/3.3/bpy.app.translations.html)：register/unregister 与上下文键契约，实际接入须核对目标版本。
- [Fluffy 作者说明](https://www.patreon.com/FluffyQuack/posts/fluffy-manager-68199696)：OpenGL 初始化相关说明，与本地模块事实一致。
