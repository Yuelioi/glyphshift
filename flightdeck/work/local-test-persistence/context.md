# 持久本地测试根上下文

## Boundaries

- `local-test/` 保存机器配置、外部授权样本、运行数据、截图、日志、捕获文本、临时目录与可复用测试
  制品；它是本机工作区，不是仓库资源。
- `target/` 可以被构建工具或用户清理，不能保存任何机器专属数据的唯一副本。
- tracked 测试只能包含确定性 harness 和合成 fixture；真实路径、PID、窗口标题和原文通过本机环境变量
  提供。
- 不读取旧根作为兼容回退。路径迁移失败应显式失败，避免两套本机状态长期分叉。
- 提交前拒绝 `local-test/` 下的 staged 路径，并扫描 staged 新增文本中的本机绝对路径与临时证据链接。

## Verification discipline

- 路径改动只运行受影响的脚本语法、Git ignore、Runtime 隐私合同与 Playwright 配置检查。
- 不因纯路径迁移运行全仓测试。
- 原始迁移清单、文件名和本机数据不进入 Flightdeck。
