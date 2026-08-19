# 稳定上下文

- `CONTEXT.md` 是单一领域 glossary，只定义 Glyphshift 特有概念及应避免的混用词，不承载实现、版本、
  测试或发布规则。
- `PRODUCT.md` 记录受众、主要任务、产品承诺和明确边界；实现细节只有在直接约束用户体验时才保留。
- `DESIGN.md` 记录当前已实现的视觉系统，不维护未来愿景；token 以 `main.css`、共享组件和已验证界面为
  事实来源。
- README 面向最终用户，中英文事实与结构一致。只有通过隐私检查的用户截图才能加入两份 README；
  含真实安装路径、进程信息或其他机器事实的图片必须先重新截取或脱敏。
- Glyphshift 的词典支持任意源语言和目标语言组合；简体中文与 English 只描述应用自身当前界面语言，
  README 不得把产品窄化为汉化工具。
- 用户确认的发布截图使用 `preview/` 作为唯一跟踪目录；机器测试截图仍只属于本地证据区。
- 用户已授权当前发布前改动创建 Git commit；最终截图仍是后续独立补充。
- tag 发布属于当前仓库：`vMAJOR.MINOR.PATCH` 必须与三处桌面版本一致，使用仓库 `GITHUB_TOKEN`
  创建 Release，不硬编码账户或依赖个人 PAT。
- Glyphshift 自有代码使用 MIT License；根 `LICENSE`、Rust workspace、桌面 npm package 与中英文
  README 必须保持一致。首个公开仓库由当前用户的个人账号拥有，首个 tag 使用桌面版本 `v0.2.0`。
- 首个公开 tag 不携带 `npm audit` high/critical 告警；允许在既有 semver 范围内更新传递依赖锁版本以
  消除发布阻塞的安全告警，并记录剩余低风险。
