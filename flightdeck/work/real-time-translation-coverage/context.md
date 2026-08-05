# Work Context

## 产品优先级

1. **实时翻译**：Adapter 能观察原文、命中 Dictionary，并在目标软件中产生可见译文。
2. **字典作业闭环**：Probe 发现原文后，用户能直接编辑、保存、导出 Dictionary，并验证实时效果。
3. **采集辅助**：不能写回的 Adapter 仍可生成原文和帮助制作 Dictionary，但必须显示为“仅采集”。
4. **架构优化**：共享 Hook、多流身份、跨启动区域 Binding 等只有在前三项出现真实阻塞时才恢复。

## 支持等级

- **已验证实时翻译**：合成合同与授权真实目标都证明观察、字典命中、写回和停用恢复；界面可见结果
  经过人工或像素级验收。
- **候选实时翻译**：Adapter 具有 `TextReplace` 合同，但尚缺真实目标命中或可见验收。
- **仅采集原文**：能够稳定产生 Observation，但没有安全原位写回能力。
- **未覆盖**：没有文字信号，或目标文字由尚未支持的进程、协议或渲染路径拥有。

成功注入、发现进程、读取 UIA、捕获 Console 输出和 Dictionary 命中都不能单独提升为“已验证实时
翻译”。产品状态必须说明证据停在哪一级。

## 选择 Adapter 的原则

- 优先选择能覆盖一类软件的稳定文字 API、组件或协议，而不是给单个软件写地址表。
- 候选必须先有一个可授权真实目标或确定性系统组件，能触发代表性文字并视觉确认替换。
- 第一版只处理文字本身；字体、布局和区域选择仅在真实替换案例暴露阻塞时进入同一切片。
- 合成测试证明参数、失败开放和停用恢复；真实测试证明该软件确实经过这条路径。两者缺一不可。
- observe-only 结果可以进入 Probe 与 Dictionary 作业，但不能被包装成实时翻译能力。
- 同一目标进程的候选 Adapter 必须独立激活；部分不兼容时保留健康技术，且只把 Runtime 明确回执的
  Adapter 能力标记为 Active。
- WPF 的 retained-mode 真实目标已经证明：加载 DirectWrite/Direct2D 模块不等于命中公开绘制入口；
  UIA 可观察而六条 Native 路径零命中时，能力等级仍是“仅采集原文”。
- 全局设置 WPF Dependency Property 会改变应用对象与 Binding 状态，不属于安全绘制时写回。需要
  Profiler/ReJIT 的托管路径必须作为独立 Engine-aware Agent 验证，不能伪装成 Native Adapter。

## 不变量

- Dictionary Entry 仍只有非空 `source + translation`。
- Core、Desktop 与 GUI 不按软件品牌、可执行文件名或 Adapter ID 分支。
- 真实程序位置、进程信息、截图、日志和原文只保存在仓库忽略的本机测试目录。
- 当前不为共享 Hook、Observation Stream、OCR、AI 翻译或在线服务扩建基础设施。
