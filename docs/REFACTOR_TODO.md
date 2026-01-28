# 因果指纹系统重构计划

## 阶段一：Solana 合约核心架构
- [ ] 新增 TaskAccount 账户结构
- [ ] 新增 FingerprintSubmission 账户结构
- [ ] 新增 GlobalFingerprint 账户结构
- [ ] 实现 submit_base 指令
- [ ] 实现 issue_challenge 指令
- [ ] 实现 submit_fingerprint 指令

## 阶段二：链下共识算法
- [ ] 新增 cosine_similarity_clustering.rs
- [ ] 新增 spectral_analysis.rs
- [ ] 新增 causal_fingerprint.rs
- [ ] 集成 nalgebra 进行谱分析
- [ ] 实现余弦相似度计算
- [ ] 实现离群点检测算法

## 阶段三：合约聚合与奖励
- [ ] 实现 aggregate_consensus 指令
- [ ] 实现 update_global_fingerprint 指令
- [ ] 添加惩罚/奖励机制
- [ ] 实现指纹稳定性计算

## 阶段四：Agent 模块重构
- [ ] Agent 添加基准预测功能
- [ ] Agent 添加因果响应计算
- [ ] Agent 添加谱特征提取
- [ ] 更新 Agent 数据类型

## 阶段五：信誉系统重构
- [ ] 重构 ReputationScore 以支持全局指纹
- [ ] 添加逻辑一致性更新原因
- [ ] 添加谱发散性更新原因
- [ ] 添加逻辑同质性检测

## 阶段六：测试与集成
- [ ] 编写合约单元测试
- [ ] 编写共识算法测试
- [ ] 编写端到端集成测试
- [ ] 性能基准测试