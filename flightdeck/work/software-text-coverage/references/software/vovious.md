# Vovious

当前决定：保留 JUCE 通用入口方向，未完成适配。

## 已知

初查发现静态 JUCE、TextEditor/Typeface 类型信息，没有可用文字函数导出。DirectWrite 模块加载不等于文字经过当前适配器。

## 恢复时先做

定位接收完整 String 的绘制或布局入口及对象生命周期；用实际界面形成可重复的采集与替换样例，再判断能否泛化为 JUCE 适配器。

初查与官方资料见 [四目标初查](../../../adapter-coverage-and-x86/slices/four-target-survey.md)。
