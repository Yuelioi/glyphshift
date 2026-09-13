# Qt Quick 标准文字

`windows.qt.quick-text` 是独立的 RetainedObject Adapter，用于现有 Qt Painter 绘制入口无法覆盖的 QML 文字对象。当前只接受动态 Qt **6.8.3 或 6.11.1 / Windows x64 / MSVC ABI**，版本和必需导出不匹配时拒绝激活。

支持 QQuickText 派生标签、使用这些标签的菜单和提示。只处理 PlainText 和保守判断的 AutoText；TextInput、TextEdit、富文本、静态链接 Qt、自绘纹理及其他 Qt 版本不在范围内。

## 生命周期

- 通过进程内 DispatchMessageW detour 将生命周期请求送入 Qt 窗口线程；不依赖调用控制线程存活。
- 在 GUI 线程使用原生计时器发现动态标签，决策使用保留的原文；宿主 setter 更新具有优先权。
- Qt getter/setter 和 Runtime 决策回调期间不持有标签表锁；对象销毁后清除记录，并以对象代号区分地址复用。
- 停用在 GUI 线程恢复仍由本 Adapter 持有的译文，等待恢复完成后才返回成功；超时返回失败，不冒充已恢复。
- 钩子代码保留到目标进程退出。停用后不再采集/替换，但更新此 DLL 需要重启目标；Runtime Catalog 明示这一边界。

## 验证入口

纯状态合同在相邻 `qt-quick` 包中。原生合成合同以受控 Qt 安装为前提，所有运行库位置从环境变量提供：

```powershell
cargo build -p glyphshift-adapter-qt-quick-native
cargo test -p glyphshift-adapter-qt-quick
# Set GLYPHSHIFT_QT_BIN, GLYPHSHIFT_QT_PLUGIN_PATH and GLYPHSHIFT_QML_IMPORT_PATH.
cargo test -p glyphshift-adapter-qt-quick-native --test retained_contract -- --ignored --test-threads=1
```

原生合同覆盖控制线程退出、代次更新、binding 更新、动态标签销毁、停用恢复、再次激活及输入/富文本不被采集。机器截图和真实 Alias 输入输出只放在本地测试目录。

构建和接入使用 `scripts/build-runtime-bundle.ps1`；桌面审阅必须使用 `scripts/review-app.ps1` 同步构建 shell 与 Runtime。Qt Quick 以动态包接入，不给产品编排层增加实现 crate 的静态依赖。
