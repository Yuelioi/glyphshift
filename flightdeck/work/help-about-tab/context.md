# 稳定上下文

- “关于”是帮助页第五个横向 Tab，不进入标题栏一级导航。
- 项目仓库固定为 `https://github.com/Yuelioi/glyphshift`；作者主页固定为
  `https://space.bilibili.com/4279370`。
- About 显示完整应用版本，并复用 package 版本元数据，不再建立另一份手写版本。
- 外部链接使用 `@tauri-apps/plugin-opener` 打开系统默认浏览器；Tauri capability 只新增 GitHub 与
  Bilibili 所需域名。
- 中文与 English 必须具有相同结构；浏览器失败需要可读错误，链接按钮具有包含目标名称的可访问名。
- GUI 验证使用仓库 Playwright runner；不运行归档 UIA 测试。
