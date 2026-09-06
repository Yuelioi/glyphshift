# 执行计划

- [x] 为 observation batch drain 与短暂观测并发建立稳定失败合同。
- [x] 在不阻塞 Adapter 热路径的前提下消除 drain 造成的生产端丢弃，保留有界队列、序列校验和暂停语义。
- [ ] 分别验收 AE 2020/2022 的启用菜单、禁用菜单和合成设置对话框；AE 2022 已通过，AE 2020 待重启后复验。
- [x] 构建并验证同一候选的 Desktop Release 与 Runtime Bundle。
- [x] 修复持续观测下 checkpoint 空闲等待饥饿；持续生产回归由红转绿，AE 效果参数捕获与中文回写实机通过。
