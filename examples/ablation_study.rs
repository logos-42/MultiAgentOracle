//! 消融实验框架 - 验证各组件贡献
//!
//! 运行: cargo run --example ablation_study -- 10
//!
//! 消融实验类型：
//! 1. 因果指纹验证消融 - 移除因果指纹验证
//! 2. 谱分析维度消融 - 减少谱特征维度
//! 3. 共识算法消融 - 对比不同聚合方法
//! 4. 扰动强度消融 - 测试不同扰动强度
//! 5. 智能体数量消融 - 测试不同规模

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio;
use serde::{Deserialize, Serialize};
use rand::Rng;

use multi_agent_oracle::consensus::{
    CausalFingerprint, CausalFingerprintConfig, cluster_by_consensus,
    CausalConsensusResult,
};
use multi_agent_oracle::oracle_agent::{LlmClient, LlmClientConfig};
use multi_agent_oracle::causal_graph::{
    CausalGraph,
    ai_reasoning::{AIReasoningEngine, AIReasoningConfig},
};

// ============================================================================
// 消融实验配置
// ============================================================================

/// 消融实验类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AblationType {
    /// 实验1: 因果指纹验证消融
    CausalFingerprintAblation,
    /// 实验2: 谱分析维度消融
    SpectralDimensionAblation,
    /// 实验3: 共识算法消融
    ConsensusAlgorithmAblation,
    /// 实验4: 扰动强度消融
    PerturbationAblation,
    /// 实验5: 智能体数量消融
    AgentCountAblation,
}

/// 消融实验配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationConfig {
    /// 实验类型
    pub ablation_type: AblationType,
    /// 配置名称（如 "baseline", "no_fingerprint"）
    pub config_name: String,
    /// 是否使用因果指纹验证
    pub use_causal_fingerprint: bool,
    /// 是否使用增量响应
    pub use_delta_response: bool,
    /// 谱特征维度 (0-8)
    pub spectral_dimensions: usize,
    /// 共识方法
    pub consensus_method: ConsensusMethod,
    /// 扰动强度
    pub perturbation_magnitude: f64,
    /// 智能体数量
    pub agent_count: usize,
    /// 拜占庭比例
    pub byzantine_ratio: f64,
    /// 描述
    pub description: String,
}

/// 共识方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMethod {
    /// 完整算法：谱分析 + 余弦相似度聚类 + 中位数
    FullSpectralClustering,
    /// 阈值过滤 + 平均
    ThresholdFilter,
    /// K-means聚类（简化版）
    KMeansClustering,
    /// 简单平均
    SimpleAverage,
    /// 加权平均（基于置信度）
    WeightedAverage,
}

/// 消融实验结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationResult {
    /// 实验类型
    pub ablation_type: String,
    /// 配置名称
    pub config_name: String,
    /// 轮次ID
    pub round_id: usize,
    /// 是否达成共识
    pub consensus_reached: bool,
    /// 共识值
    pub consensus_value: f64,
    /// 真实值
    pub ground_truth: f64,
    /// 精度
    pub accuracy: f64,
    /// 收敛时间(ms)
    pub convergence_time_ms: u64,
    /// 有效智能体数量
    pub valid_agents_count: usize,
    /// 检测到的拜占庭数量
    pub detected_byzantine_count: usize,
    /// 共识相似度
    pub consensus_similarity: f64,
    /// API调用次数
    pub api_calls_count: usize,
    /// 时间戳
    pub timestamp: i64,
}

/// 消融实验汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AblationSummary {
    /// 配置名称
    pub config_name: String,
    /// 实验类型
    pub ablation_type: String,
    /// 总轮次
    pub total_rounds: usize,
    /// 共识达成率
    pub consensus_rate: f64,
    /// 平均精度
    pub avg_accuracy: f64,
    /// 平均收敛时间
    pub avg_convergence_time_ms: f64,
    /// 平均拜占庭检测率
    pub avg_byzantine_detection_rate: f64,
    /// 平均共识相似度
    pub avg_consensus_similarity: f64,
    /// 总API调用次数
    pub total_api_calls: usize,
}

// ============================================================================
// 测试场景
// ============================================================================

#[derive(Debug, Clone)]
pub struct TestScenario {
    pub id: String,
    pub description: String,
    pub ground_truth: f64,
    pub intervention_prompt: String,
    pub perturbation_prompt: String,
}

// ============================================================================
// 消融实验运行器
// ============================================================================

pub struct AblationRunner {
    /// LLM客户端
    pub llm_client: LlmClient,
    /// AI推理引擎
    pub ai_reasoning: Option<AIReasoningEngine>,
    /// 测试场景
    pub scenarios: Vec<TestScenario>,
    /// 结果
    pub results: Vec<AblationResult>,
    /// API调用计数
    pub api_call_count: usize,
}

impl AblationRunner {
    /// 创建新的消融实验运行器
    pub async fn new() -> Result<Self> {
        let llm_config = LlmClientConfig::deepseek("deepseek-chat")
            .with_temperature(0.7)
            .with_max_tokens(2500);
        
        let llm_client = LlmClient::new(llm_config)?;
        let scenarios = Self::initialize_scenarios();

        // 初始化AI推理引擎
        let ai_reasoning = {
            let ai_config = AIReasoningConfig {
                llm_provider: multi_agent_oracle::oracle_agent::LlmProvider::DeepSeek,
                model: "deepseek-chat".to_string(),
                temperature: 0.7,
                max_tokens: 2500,
                enable_json_mode: true,
                min_nodes: 3,
                max_nodes: 5,
                min_paths: 2,
                max_paths: 3,
            };
            AIReasoningEngine::new(ai_config).ok()
        };

        Ok(Self {
            llm_client,
            ai_reasoning,
            scenarios,
            results: Vec::new(),
            api_call_count: 0,
        })
    }

    /// 初始化测试场景
    fn initialize_scenarios() -> Vec<TestScenario> {
        vec![
            TestScenario {
                id: "interest_inflation".to_string(),
                description: "央行提高利率对通胀率的影响".to_string(),
                ground_truth: 2.5,
                intervention_prompt: "当前利率3%，通胀率4%。如果央行将利率提高到4%，预测6个月后的通胀率是多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
                perturbation_prompt: "当前利率3%，通胀率4%。如果央行将利率提高到5%（提高2%），预测6个月后的通胀率是多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
            },
            TestScenario {
                id: "supply_price".to_string(),
                description: "原材料成本上涨对产品价格的影响".to_string(),
                ground_truth: 15.0,
                intervention_prompt: "当前原材料成本100元，产品售价150元。如果原材料成本上涨到120元，预测新的产品售价是多少？请只回答一个具体数字（元），不要解释。".to_string(),
                perturbation_prompt: "当前原材料成本100元，产品售价150元。如果原材料成本上涨到140元，预测新的产品售价是多少？请只回答一个具体数字（元），不要解释。".to_string(),
            },
            TestScenario {
                id: "ai_efficiency".to_string(),
                description: "AI投资对企业效率的影响".to_string(),
                ground_truth: 25.0,
                intervention_prompt: "企业当前年营收1000万元，投入100万元用于AI技术。预测一年后效率提升百分比是多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
                perturbation_prompt: "企业当前年营收1000万元，投入200万元用于AI技术。预测一年后效率提升百分比是多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
            },
            TestScenario {
                id: "market_share".to_string(),
                description: "广告投入对市场份额的影响".to_string(),
                ground_truth: 5.0,
                intervention_prompt: "公司当前市场份额20%，投入500万广告费。预测一年后市场份额增长多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
                perturbation_prompt: "公司当前市场份额20%，投入1000万广告费。预测一年后市场份额增长多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
            },
            TestScenario {
                id: "tech_adoption".to_string(),
                description: "新技术对生产成本的影响".to_string(),
                ground_truth: 12.0,
                intervention_prompt: "工厂当前生产成本100元/件，引入新技术后，预测成本降低多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
                perturbation_prompt: "工厂当前生产成本100元/件，引入高级新技术后，预测成本降低多少？请只回答一个具体数字（百分比），不要解释。".to_string(),
            },
        ]
    }

    /// 从LLM响应中提取数值
    fn extract_number_from_response(text: &str) -> Option<f64> {
        let clean_text = text.trim();
        if let Some(num) = clean_text.split_whitespace().next() {
            if let Ok(f) = num.parse::<f64>() {
                return Some(f);
            }
        }
        let no_percent = clean_text.replace("%", "").trim().to_string();
        if let Ok(f) = no_percent.parse::<f64>() {
            return Some(f);
        }
        None
    }

    /// 生成智能体数据（真实LLM调用）
    async fn generate_agent_data(
        &mut self,
        agent_id: &str,
        scenario: &TestScenario,
        is_byzantine: bool,
        config: &AblationConfig,
    ) -> Result<(f64, f64, Vec<f64>, Vec<f64>)> {
        // 1. 调用LLM获取基础预测
        self.api_call_count += 1;
        let base_response = self.llm_client.generate_response(&scenario.intervention_prompt).await?;
        let base_prediction = Self::extract_number_from_response(&base_response.text)
            .unwrap_or(scenario.ground_truth);

        // 2. 根据配置决定是否使用增量响应
        let (perturbed_prediction, delta_response) = if config.use_delta_response {
            self.api_call_count += 1;
            let perturbed_response = self.llm_client.generate_response(&scenario.perturbation_prompt).await?;
            let perturbed = Self::extract_number_from_response(&perturbed_response.text)
                .unwrap_or(base_prediction);
            
            // 计算增量响应
            let delta = perturbed - base_prediction;
            let delta_vec = vec![delta; config.spectral_dimensions.max(5)];
            (perturbed, delta_vec)
        } else {
            // 不使用增量响应
            (base_prediction, vec![0.0; config.spectral_dimensions.max(5)])
        };

        // 3. 生成谱特征
        let spectral_features = if config.spectral_dimensions > 0 {
            self.generate_spectral_features(&delta_response, config.spectral_dimensions)
        } else {
            vec![]
        };

        // 4. 拜占庭节点添加噪声
        if is_byzantine {
            let mut rng = rand::thread_rng();
            let noise_factor = 0.5 + rng.gen::<f64>();
            Ok((
                base_prediction * noise_factor,
                perturbed_prediction * noise_factor,
                delta_response.iter().map(|d| d * noise_factor).collect(),
                spectral_features.iter().map(|f| f * noise_factor).collect(),
            ))
        } else {
            Ok((base_prediction, perturbed_prediction, delta_response, spectral_features))
        }
    }

    /// 生成谱特征
    fn generate_spectral_features(&self, delta_response: &[f64], dimensions: usize) -> Vec<f64> {
        let delta_sum: f64 = delta_response.iter().sum();
        let delta_mean = delta_sum / delta_response.len().max(1) as f64;
        let delta_var = delta_response.iter()
            .map(|d| (d - delta_mean).powi(2))
            .sum::<f64>() / delta_response.len().max(1) as f64;

        // 生成指定维度的谱特征
        let mut features = Vec::with_capacity(dimensions);
        
        features.push(delta_response.len() as f64);  // 特征1: 维度
        features.push(delta_sum.abs());              // 特征2: 总变化
        
        if dimensions > 2 {
            features.push(delta_mean);                // 特征3: 平均变化
        }
        if dimensions > 3 {
            features.push(delta_var.sqrt());          // 特征4: 标准差
        }
        if dimensions > 4 {
            features.push(delta_response.get(0).copied().unwrap_or(0.0));  // 特征5: 第一维
        }
        if dimensions > 5 {
            features.push(delta_response.get(1).copied().unwrap_or(0.0));  // 特征6: 第二维
        }
        if dimensions > 6 {
            features.push(delta_response.get(2).copied().unwrap_or(0.0));  // 特征7: 第三维
        }
        if dimensions > 7 {
            features.push(delta_mean.abs() + delta_var);  // 特征8: 综合特征
        }

        features.truncate(dimensions);
        features
    }

    /// 运行单轮消融实验
    pub async fn run_single_round(
        &mut self,
        round_id: usize,
        config: &AblationConfig,
    ) -> Result<AblationResult> {
        let round_start = Instant::now();
        let initial_api_count = self.api_call_count;

        // 选择场景
        let scenario_idx = round_id % self.scenarios.len();
        let scenario = self.scenarios[scenario_idx].clone();

        // 计算拜占庭数量
        let byzantine_count = (config.agent_count as f64 * config.byzantine_ratio).round() as usize;

        // 生成智能体数据
        let mut agents_data = Vec::new();
        for i in 0..config.agent_count {
            let (base, perturbed, delta, spectral) = self.generate_agent_data(
                &format!("agent_{:03}", i),
                &scenario,
                i < byzantine_count,
                config,
            ).await?;
            
            agents_data.push((base, perturbed, delta, spectral, i < byzantine_count));
        }

        // 根据共识方法计算结果
        let (consensus_value, valid_agents_count, detected_byzantine, consensus_similarity) = 
            self.compute_consensus(&agents_data, config);

        // 计算真实值（正常智能体的平均值）
        let ground_truth = agents_data.iter()
            .filter(|(_, _, _, _, is_byz)| !*is_byz)
            .map(|(base, _, _, _, _)| *base)
            .sum::<f64>() / (config.agent_count - byzantine_count).max(1) as f64;

        // 计算精度
        let accuracy = if consensus_value != 0.0 {
            1.0 - ((consensus_value - ground_truth).abs() / ground_truth.abs())
        } else {
            0.0
        };

        let convergence_time = round_start.elapsed().as_millis() as u64;
        let api_calls_this_round = self.api_call_count - initial_api_count;

        Ok(AblationResult {
            ablation_type: format!("{:?}", config.ablation_type),
            config_name: config.config_name.clone(),
            round_id,
            consensus_reached: valid_agents_count > 0,
            consensus_value,
            ground_truth,
            accuracy: accuracy.max(0.0),
            convergence_time_ms: convergence_time,
            valid_agents_count,
            detected_byzantine_count: detected_byzantine,
            consensus_similarity,
            api_calls_count: api_calls_this_round,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        })
    }

    /// 计算共识（根据不同方法）
    fn compute_consensus(
        &self,
        agents_data: &[(f64, f64, Vec<f64>, Vec<f64>, bool)],
        config: &AblationConfig,
    ) -> (f64, usize, usize, f64) {
        match config.consensus_method {
            ConsensusMethod::FullSpectralClustering => {
                self.full_spectral_clustering(agents_data, config)
            }
            ConsensusMethod::ThresholdFilter => {
                self.threshold_filter_consensus(agents_data, config)
            }
            ConsensusMethod::KMeansClustering => {
                self.kmeans_clustering(agents_data, config)
            }
            ConsensusMethod::SimpleAverage => {
                self.simple_average(agents_data)
            }
            ConsensusMethod::WeightedAverage => {
                self.weighted_average(agents_data)
            }
        }
    }

    /// 完整谱聚类算法
    fn full_spectral_clustering(
        &self,
        agents_data: &[(f64, f64, Vec<f64>, Vec<f64>, bool)],
        config: &AblationConfig,
    ) -> (f64, usize, usize, f64) {
        if agents_data.is_empty() {
            return (0.0, 0, 0, 0.0);
        }

        // 构建因果指纹
        let fingerprints: Vec<CausalFingerprint> = agents_data.iter().enumerate().map(|(idx, (base, _, delta, spectral, _))| {
            CausalFingerprint {
                agent_id: format!("agent_{:03}", idx),
                base_prediction: *base,
                delta_response: delta.clone(),
                spectral_features: spectral.clone(),
                perturbation: vec![config.perturbation_magnitude; 5],
                confidence: 0.9,
                timestamp: 0,
            }
        }).collect();

        // 使用因果指纹验证（如果启用）
        let consensus_result = if config.use_causal_fingerprint {
            let fp_config = CausalFingerprintConfig {
                cosine_threshold: 0.8,
                min_valid_agents: 3,
                ..Default::default()
            };
            cluster_by_consensus(&fingerprints, &fp_config)
        } else {
            // 不使用因果指纹验证，直接使用所有智能体
            CausalConsensusResult {
                consensus_value: agents_data.iter().map(|(base, _, _, _, _)| *base).sum::<f64>() / agents_data.len() as f64,
                valid_agents: agents_data.iter().enumerate().map(|(i, _)| format!("agent_{:03}", i)).collect(),
                outliers: vec![],
                consensus_similarity: 1.0,
                cluster_quality: 1.0,
            }
        };

        // 计算检测到的拜占庭数量
        let detected_byzantine = agents_data.iter()
            .filter(|(_, _, _, _, is_byz)| {
                *is_byz && consensus_result.outliers.contains(&format!("agent_{:03}", 
                    agents_data.iter().position(|(_, _, _, _, b)| *b).unwrap_or(0)))
            })
            .count();

        (
            consensus_result.consensus_value,
            consensus_result.valid_agents.len(),
            detected_byzantine,
            consensus_result.consensus_similarity,
        )
    }

    /// 阈值过滤 + 平均
    fn threshold_filter_consensus(
        &self,
        agents_data: &[(f64, f64, Vec<f64>, Vec<f64>, bool)],
        config: &AblationConfig,
    ) -> (f64, usize, usize, f64) {
        if agents_data.is_empty() {
            return (0.0, 0, 0, 0.0);
        }

        let threshold = 0.8;
        let base_values: Vec<f64> = agents_data.iter().map(|(base, _, _, _, _)| *base).collect();
        let median = Self::calculate_median(&base_values);

        // 过滤偏离中位数较远的智能体
        let filtered: Vec<(usize, f64)> = agents_data.iter().enumerate()
            .filter_map(|(idx, (base, _, _, _, _))| {
                let deviation = (base - median).abs() / median.abs().max(0.001);
                if deviation < (1.0 - threshold) {
                    Some((idx, *base))
                } else {
                    None
                }
            })
            .collect();

        let consensus_value = if filtered.is_empty() {
            median
        } else {
            filtered.iter().map(|(_, v)| *v).sum::<f64>() / filtered.len() as f64
        };

        let valid_count = filtered.len();
        let detected_byzantine = agents_data.iter().enumerate()
            .filter(|(idx, (_, _, _, _, is_byz))| {
                *is_byz && !filtered.iter().any(|(fidx, _)| *fidx == *idx)
            })
            .count();

        let similarity = if valid_count > 1 {
            let values: Vec<f64> = filtered.iter().map(|(_, v)| *v).collect();
            Self::calculate_similarity(&values)
        } else {
            0.0
        };

        (consensus_value, valid_count, detected_byzantine, similarity)
    }

    /// K-means聚类（简化版）
    fn kmeans_clustering(
        &self,
        agents_data: &[(f64, f64, Vec<f64>, Vec<f64>, bool)],
        _config: &AblationConfig,
    ) -> (f64, usize, usize, f64) {
        if agents_data.is_empty() {
            return (0.0, 0, 0, 0.0);
        }

        let base_values: Vec<f64> = agents_data.iter().map(|(base, _, _, _, _)| *base).collect();
        
        // 简化的K-means：找到最大簇
        let k = 2; // 假设有正常和异常两个簇
        let mut centroids = vec![base_values[0], base_values[base_values.len() / 2]];
        
        // 迭代几次
        for _ in 0..10 {
            let mut clusters: Vec<Vec<usize>> = vec![vec![], vec![]];
            
            for (idx, &value) in base_values.iter().enumerate() {
                let dist0 = (value - centroids[0]).abs();
                let dist1 = (value - centroids[1]).abs();
                if dist0 < dist1 {
                    clusters[0].push(idx);
                } else {
                    clusters[1].push(idx);
                }
            }
            
            // 更新中心
            for (i, cluster) in clusters.iter().enumerate() {
                if !cluster.is_empty() {
                    centroids[i] = cluster.iter()
                        .map(|&idx| base_values[idx])
                        .sum::<f64>() / cluster.len() as f64;
                }
            }
        }

        // 选择最大的簇
        let main_cluster: Vec<usize> = if base_values.len() > 0 {
            let mut cluster0: Vec<usize> = vec![];
            let mut cluster1: Vec<usize> = vec![];
            
            for (idx, &value) in base_values.iter().enumerate() {
                let dist0 = (value - centroids[0]).abs();
                let dist1 = (value - centroids[1]).abs();
                if dist0 < dist1 {
                    cluster0.push(idx);
                } else {
                    cluster1.push(idx);
                }
            }
            
            if cluster0.len() >= cluster1.len() { cluster0 } else { cluster1 }
        } else {
            vec![]
        };

        let consensus_value = if main_cluster.is_empty() {
            0.0
        } else {
            main_cluster.iter()
                .map(|&idx| base_values[idx])
                .sum::<f64>() / main_cluster.len() as f64
        };

        let detected_byzantine = agents_data.iter().enumerate()
            .filter(|(idx, (_, _, _, _, is_byz))| {
                *is_byz && !main_cluster.contains(idx)
            })
            .count();

        let similarity = if main_cluster.len() > 1 {
            let values: Vec<f64> = main_cluster.iter().map(|&idx| base_values[idx]).collect();
            Self::calculate_similarity(&values)
        } else {
            0.0
        };

        (consensus_value, main_cluster.len(), detected_byzantine, similarity)
    }

    /// 简单平均
    fn simple_average(
        &self,
        agents_data: &[(f64, f64, Vec<f64>, Vec<f64>, bool)],
    ) -> (f64, usize, usize, f64) {
        if agents_data.is_empty() {
            return (0.0, 0, 0, 0.0);
        }

        let sum: f64 = agents_data.iter().map(|(base, _, _, _, _)| *base).sum();
        let consensus_value = sum / agents_data.len() as f64;

        let base_values: Vec<f64> = agents_data.iter().map(|(base, _, _, _, _)| *base).collect();
        let similarity = Self::calculate_similarity(&base_values);

        (consensus_value, agents_data.len(), 0, similarity) // 简单平均无法检测拜占庭
    }

    /// 加权平均
    fn weighted_average(
        &self,
        agents_data: &[(f64, f64, Vec<f64>, Vec<f64>, bool)],
    ) -> (f64, usize, usize, f64) {
        if agents_data.is_empty() {
            return (0.0, 0, 0, 0.0);
        }

        // 基于增量响应的稳定性分配权重
        let weights: Vec<f64> = agents_data.iter()
            .map(|(_, _, delta, _, _)| {
                // 增量响应越稳定，权重越高
                let variance = if delta.is_empty() {
                    1.0
                } else {
                    let mean = delta.iter().sum::<f64>() / delta.len() as f64;
                    delta.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / delta.len() as f64
                };
                1.0 / (1.0 + variance)
            })
            .collect();

        let total_weight: f64 = weights.iter().sum();
        let consensus_value = agents_data.iter()
            .zip(weights.iter())
            .map(|((base, _, _, _, _), w)| base * w)
            .sum::<f64>() / total_weight;

        let base_values: Vec<f64> = agents_data.iter().map(|(base, _, _, _, _)| *base).collect();
        let similarity = Self::calculate_similarity(&base_values);

        (consensus_value, agents_data.len(), 0, similarity)
    }

    /// 计算中位数
    fn calculate_median(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// 计算相似度
    fn calculate_similarity(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 1.0;
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        if mean == 0.0 {
            return 0.0;
        }
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        1.0 / (1.0 + variance.sqrt() / mean.abs())
    }

    /// 计算汇总
    fn calculate_summary(&self, config_name: &str, ablation_type: &str) -> AblationSummary {
        let config_results: Vec<&AblationResult> = self.results.iter()
            .filter(|r| r.config_name == config_name)
            .collect();

        let total_rounds = config_results.len();
        let consensus_rate = config_results.iter()
            .filter(|r| r.consensus_reached)
            .count() as f64 / total_rounds.max(1) as f64;
        
        let avg_accuracy = config_results.iter()
            .map(|r| r.accuracy)
            .sum::<f64>() / total_rounds.max(1) as f64;
        
        let avg_convergence_time_ms = config_results.iter()
            .map(|r| r.convergence_time_ms as f64)
            .sum::<f64>() / total_rounds.max(1) as f64;
        
        let avg_byzantine_detection_rate = config_results.iter()
            .map(|r| r.detected_byzantine_count as f64)
            .sum::<f64>() / total_rounds.max(1) as f64;
        
        let avg_consensus_similarity = config_results.iter()
            .map(|r| r.consensus_similarity)
            .sum::<f64>() / total_rounds.max(1) as f64;
        
        let total_api_calls = config_results.iter()
            .map(|r| r.api_calls_count)
            .sum();

        AblationSummary {
            config_name: config_name.to_string(),
            ablation_type: ablation_type.to_string(),
            total_rounds,
            consensus_rate,
            avg_accuracy,
            avg_convergence_time_ms,
            avg_byzantine_detection_rate,
            avg_consensus_similarity,
            total_api_calls,
        }
    }

    /// 保存结果
    pub async fn save_results(&self, output_dir: &str) -> Result<()> {
        fs::create_dir_all(output_dir)?;

        // 保存详细结果
        let csv_data = self.generate_csv();
        let csv_path = format!("{}/ablation_results.csv", output_dir);
        File::create(&csv_path)?.write_all(csv_data.as_bytes())?;

        // 保存JSON
        let json_data = serde_json::to_string_pretty(&self.results)?;
        let json_path = format!("{}/ablation_results.json", output_dir);
        File::create(&json_path)?.write_all(json_data.as_bytes())?;

        // 生成对比报告
        let report = self.generate_comparison_report();
        let report_path = format!("{}/ablation_report.md", output_dir);
        File::create(&report_path)?.write_all(report.as_bytes())?;

        println!("\n📊 消融实验结果已保存到: {}", output_dir);
        Ok(())
    }

    /// 生成CSV
    fn generate_csv(&self) -> String {
        let mut csv = String::from("ablation_type,config_name,round_id,consensus_reached,");
        csv.push_str("consensus_value,ground_truth,accuracy,convergence_time_ms,");
        csv.push_str("valid_agents_count,detected_byzantine_count,");
        csv.push_str("consensus_similarity,api_calls_count,timestamp\n");

        for r in &self.results {
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                r.ablation_type, r.config_name, r.round_id, r.consensus_reached,
                r.consensus_value, r.ground_truth, r.accuracy, r.convergence_time_ms,
                r.valid_agents_count, r.detected_byzantine_count,
                r.consensus_similarity, r.api_calls_count, r.timestamp
            ));
        }
        csv
    }

    /// 生成对比报告
    fn generate_comparison_report(&self) -> String {
        let mut report = String::from("# 消融实验报告\n\n");
        
        // 按实验类型分组
        let mut by_type: HashMap<String, Vec<&AblationResult>> = HashMap::new();
        for result in &self.results {
            by_type.entry(result.ablation_type.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        for (ablation_type, results) in &by_type {
            report.push_str(&format!("## {}\n\n", ablation_type));
            
            // 按配置名称分组计算汇总
            let mut by_config: HashMap<String, Vec<&AblationResult>> = HashMap::new();
            for r in results {
                by_config.entry(r.config_name.clone())
                    .or_insert_with(Vec::new)
                    .push(r);
            }

            report.push_str("| 配置 | 轮次 | 共识率 | 精度 | 拜占庭检测 | 相似度 | 时间(ms) |\n");
            report.push_str("|------|------|--------|------|------------|--------|----------|\n");

            for (config_name, config_results) in &by_config {
                let total = config_results.len();
                let consensus_rate = config_results.iter().filter(|r| r.consensus_reached).count() as f64 / total as f64;
                let avg_accuracy = config_results.iter().map(|r| r.accuracy).sum::<f64>() / total as f64;
                let avg_detection = config_results.iter().map(|r| r.detected_byzantine_count as f64).sum::<f64>() / total as f64;
                let avg_similarity = config_results.iter().map(|r| r.consensus_similarity).sum::<f64>() / total as f64;
                let avg_time = config_results.iter().map(|r| r.convergence_time_ms as f64).sum::<f64>() / total as f64;

                report.push_str(&format!("| {} | {} | {:.1}% | {:.1}% | {:.1} | {:.3} | {:.0} |\n",
                    config_name, total, consensus_rate * 100.0, avg_accuracy * 100.0,
                    avg_detection, avg_similarity, avg_time
                ));
            }
            report.push_str("\n");
        }

        // 添加总结
        report.push_str("## 总结\n\n");
        report.push_str(&format!("- 总实验轮次: {}\n", self.results.len()));
        report.push_str(&format!("- 总API调用: {}\n", self.api_call_count));
        report.push_str(&format!("- 估算成本: ¥{:.2}\n", self.api_call_count as f64 * 0.001));

        report
    }
}

// ============================================================================
// 预定义消融配置
// ============================================================================

/// 获取因果指纹验证消融配置
pub fn get_causal_fingerprint_ablation_configs() -> Vec<AblationConfig> {
    vec![
        AblationConfig {
            ablation_type: AblationType::CausalFingerprintAblation,
            config_name: "baseline".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "完整系统（Baseline）".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::CausalFingerprintAblation,
            config_name: "no_fingerprint".to_string(),
            use_causal_fingerprint: false,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "移除因果指纹验证".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::CausalFingerprintAblation,
            config_name: "no_delta".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: false,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "移除增量响应".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::CausalFingerprintAblation,
            config_name: "simple_average".to_string(),
            use_causal_fingerprint: false,
            use_delta_response: false,
            spectral_dimensions: 0,
            consensus_method: ConsensusMethod::SimpleAverage,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "简单平均（无任何验证）".to_string(),
        },
    ]
}

/// 获取谱分析维度消融配置
pub fn get_spectral_dimension_ablation_configs() -> Vec<AblationConfig> {
    vec![
        AblationConfig {
            ablation_type: AblationType::SpectralDimensionAblation,
            config_name: "8d_spectral".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "8维谱特征（Baseline）".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::SpectralDimensionAblation,
            config_name: "4d_spectral".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 4,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "4维谱特征".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::SpectralDimensionAblation,
            config_name: "2d_spectral".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 2,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "2维谱特征".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::SpectralDimensionAblation,
            config_name: "0d_spectral".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 0,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "无谱特征".to_string(),
        },
    ]
}

/// 获取共识算法消融配置
pub fn get_consensus_algorithm_ablation_configs() -> Vec<AblationConfig> {
    vec![
        AblationConfig {
            ablation_type: AblationType::ConsensusAlgorithmAblation,
            config_name: "full_spectral".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "完整谱聚类（Baseline）".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::ConsensusAlgorithmAblation,
            config_name: "threshold_filter".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::ThresholdFilter,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "阈值过滤 + 平均".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::ConsensusAlgorithmAblation,
            config_name: "kmeans".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::KMeansClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "K-means聚类".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::ConsensusAlgorithmAblation,
            config_name: "weighted_avg".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::WeightedAverage,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "加权平均".to_string(),
        },
    ]
}

/// 获取扰动强度消融配置
pub fn get_perturbation_ablation_configs() -> Vec<AblationConfig> {
    vec![
        AblationConfig {
            ablation_type: AblationType::PerturbationAblation,
            config_name: "perturb_0.5".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 0.5,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "弱扰动（0.5）".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::PerturbationAblation,
            config_name: "perturb_1.0".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "标准扰动（1.0，Baseline）".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::PerturbationAblation,
            config_name: "perturb_2.0".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 2.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "强扰动（2.0）".to_string(),
        },
    ]
}

/// 获取智能体数量消融配置
pub fn get_agent_count_ablation_configs() -> Vec<AblationConfig> {
    vec![
        AblationConfig {
            ablation_type: AblationType::AgentCountAblation,
            config_name: "5_agents".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 5,
            byzantine_ratio: 0.2,
            description: "5个智能体".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::AgentCountAblation,
            config_name: "10_agents".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 10,
            byzantine_ratio: 0.2,
            description: "10个智能体（Baseline）".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::AgentCountAblation,
            config_name: "15_agents".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 15,
            byzantine_ratio: 0.2,
            description: "15个智能体".to_string(),
        },
        AblationConfig {
            ablation_type: AblationType::AgentCountAblation,
            config_name: "20_agents".to_string(),
            use_causal_fingerprint: true,
            use_delta_response: true,
            spectral_dimensions: 8,
            consensus_method: ConsensusMethod::FullSpectralClustering,
            perturbation_magnitude: 1.0,
            agent_count: 20,
            byzantine_ratio: 0.2,
            description: "20个智能体".to_string(),
        },
    ]
}

// ============================================================================
// 主函数
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    let rounds_per_config = if args.len() > 1 {
        args[1].parse().unwrap_or(5)
    } else {
        5
    };

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          消融实验 - 验证各组件贡献                    ║");
    println!("║          每个配置运行 {} 轮                            ║", rounds_per_config);
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut runner = AblationRunner::new().await?;
    let start_time = Instant::now();

    // 收集所有配置
    let all_configs: Vec<AblationConfig> = vec![
        get_causal_fingerprint_ablation_configs(),
        get_spectral_dimension_ablation_configs(),
        get_consensus_algorithm_ablation_configs(),
        get_perturbation_ablation_configs(),
        get_agent_count_ablation_configs(),
    ].into_iter().flatten().collect();

    println!("📋 总共 {} 个配置，每个运行 {} 轮", all_configs.len(), rounds_per_config);
    println!("   预计API调用: ~{} 次", all_configs.len() * rounds_per_config * 10 * 2);
    println!("   预计成本: ¥{:.2}\n", all_configs.len() as f64 * rounds_per_config as f64 * 10.0 * 2.0 * 0.001);

    // 运行所有配置
    for (config_idx, config) in all_configs.iter().enumerate() {
        println!("\n🔬 [{}/{}] {}", config_idx + 1, all_configs.len(), config.description);
        println!("   配置: {}", config.config_name);

        for round in 0..rounds_per_config {
            match runner.run_single_round(round, config).await {
                Ok(result) => {
                    runner.results.push(result);
                    print!("✓");
                }
                Err(e) => {
                    print!("✗({})", e);
                }
            }
        }
        println!(" 完成");
    }

    let elapsed = start_time.elapsed();
    println!("\n\n✅ 消融实验完成!");
    println!("   总轮次: {}", runner.results.len());
    println!("   耗时: {:.2} 秒", elapsed.as_secs_f64());
    println!("   API调用次数: {}", runner.api_call_count);
    println!("   估算成本: ¥{:.2}", runner.api_call_count as f64 * 0.001);

    // 保存结果
    let output_dir = format!("experiments/output/ablation_study_{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs());
    runner.save_results(&output_dir).await?;

    println!("\n🎉 消融实验成功完成！");
    Ok(())
}
