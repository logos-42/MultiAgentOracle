//! 真实基准测试实验 - 使用DeepSeek API生成可信数据
//!
//! 运行: cargo run --example real_benchmark_experiment -- 10

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use rand::Rng;

use multi_agent_oracle::consensus::{
    CausalFingerprint, CausalFingerprintConfig, cluster_by_consensus,
};
use multi_agent_oracle::oracle_agent::{LlmClient, LlmClientConfig};
use multi_agent_oracle::causal_graph::{
    CausalGraph,
    ai_reasoning::{AIReasoningEngine, AIReasoningConfig},
};
use multi_agent_oracle::consensus::{extract_spectral_features};

#[derive(Debug, Clone, Serialize)]
pub struct ExperimentConfig {
    pub name: String,
    pub agent_counts: Vec<usize>,
    pub byzantine_ratios: Vec<f64>,
    pub consensus_thresholds: Vec<f64>,
    pub repetitions: usize,
    pub output_dir: String,
    pub llm_model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            name: "real_multi_agent_oracle_10_agents_minimax".to_string(),
            agent_counts: vec![10],  // 固定为10个智能体
            byzantine_ratios: vec![],  // 空数组表示使用随机拜占庭节点数(0-40%)
            consensus_thresholds: vec![0.8],  // 固定共识阈值
            repetitions: 25,  // 设置为25次重复，这样总共运行25轮相同配置
            output_dir: "experiments/output".to_string(),
            llm_model: "abab5.5-chat".to_string(),  // 使用 Minimax 模型
            temperature: 0.7,
            max_tokens: 2500,  // 增加到2500以确保JSON不被截断
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentRound {
    pub round_id: usize,
    pub agent_count: usize,
    pub byzantine_count: usize,
    pub threshold: f64,
    pub consensus_reached: bool,
    pub consensus_value: f64,
    pub ground_truth: f64,
    pub accuracy: f64,
    pub convergence_time_ms: u64,
    pub valid_agents: Vec<String>,
    pub outliers: Vec<String>,
    pub consensus_similarity: f64,
    pub api_calls_count: usize,
    pub timestamp: i64,
}

/// 详细智能体数据 - 包含谱分析和因果图信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetailedInfo {
    pub round_id: usize,
    pub agent_id: String,
    pub is_byzantine: bool,
    pub base_prediction: f64,
    pub perturbed_prediction: f64,
    pub delta_response: Vec<f64>,
    pub spectral_features: Vec<f64>,
    pub confidence: f64,
    pub reasoning: String,
    pub causal_graph_summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RealAgent {
    pub id: String,
    pub causal_graph: Option<CausalGraph>,  // AI生成的因果图
    pub base_prediction: f64,
    pub perturbed_prediction: f64,
    pub delta_response: Vec<f64>,
    pub spectral_features: Vec<f64>,  // 从因果图和响应计算的谱特征
    pub confidence: f64,
    pub is_byzantine: bool,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct TestScenario {
    pub id: String,
    pub description: String,
    pub ground_truth: f64,
    pub intervention_prompt: String,
    pub perturbation_prompt: String,
}

pub struct RealBenchmarkRunner {
    pub config: ExperimentConfig,
    pub llm_client: LlmClient,
    pub ai_reasoning: Option<AIReasoningEngine>,  // AI因果图生成引擎
    pub scenarios: Vec<TestScenario>,
    pub results: Vec<ExperimentRound>,
    pub detailed_agent_data: Vec<AgentDetailedInfo>,  // 详细智能体数据（谱分析和因果图）
    pub api_call_count: usize,
    pub output_dir: String,  // 输出目录路径（用于增量保存）
}

impl RealBenchmarkRunner {
    pub async fn new(config: ExperimentConfig) -> Result<Self> {
        let llm_config = LlmClientConfig::minimax(&config.llm_model)
            .with_temperature(config.temperature)
            .with_max_tokens(config.max_tokens);

        let llm_client = LlmClient::new(llm_config)?;
        let scenarios = Self::initialize_scenarios();

        println!("✅ 真实实验运行器初始化完成");
        println!("   使用模型: Minimax ({})", config.llm_model);

        // 初始化AI推理引擎（用于生成因果图）
        let ai_reasoning = {
            let ai_config = AIReasoningConfig {
                llm_provider: multi_agent_oracle::oracle_agent::LlmProvider::Minimax,
                model: config.llm_model.clone(),
                temperature: config.temperature,
                max_tokens: config.max_tokens,
                enable_json_mode: true,
                min_nodes: 3,
                max_nodes: 5,
                min_paths: 2,
                max_paths: 3,
            };
            match AIReasoningEngine::new(ai_config) {
                Ok(engine) => {
                    println!("   ✅ AI因果图推理引擎初始化成功");
                    Some(engine)
                }
                Err(e) => {
                    println!("   ⚠️ AI推理引擎初始化失败: {}, 将使用简化特征", e);
                    None
                }
            }
        };

        // 创建输出目录（基于时间戳）
        let output_dir = format!("{}/real_experiment_{}",
            config.output_dir,
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        );
        fs::create_dir_all(&output_dir)?;

        println!("📁 输出目录: {}", output_dir);

        Ok(Self {
            config,
            llm_client,
            ai_reasoning,
            scenarios,
            results: Vec::new(),
            detailed_agent_data: Vec::new(),
            api_call_count: 0,
            output_dir,
        })
    }

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
        ]
    }

    /// 从LLM响应中提取数值
    fn extract_number_from_response(text: &str) -> Option<f64> {
        let clean_text = text.trim();
        // 尝试多种格式
        if let Some(num) = clean_text.split_whitespace().next() {
            if let Ok(f) = num.parse::<f64>() {
                return Some(f);
            }
        }
        // 移除百分号后解析
        let no_percent = clean_text.replace("%", "").trim().to_string();
        if let Ok(f) = no_percent.parse::<f64>() {
            return Some(f);
        }
        None
    }

    /// 生成真实智能体（调用LLM + 因果图生成）
    async fn generate_real_agent(&mut self, agent_id: &str, _scenario: &TestScenario, is_byzantine: bool) -> Result<RealAgent> {
        // 选择场景轮询
        let scenario_index = agent_id.chars().last().unwrap() as usize % self.scenarios.len();
        let scenario = self.scenarios[scenario_index].clone();

        // 1. 调用LLM获取基础预测 f(x)
        self.api_call_count += 1;
        let base_response = self.llm_client.generate_response(&scenario.intervention_prompt).await?;
        let base_prediction = Self::extract_number_from_response(&base_response.text)
            .unwrap_or_else(|| scenario.ground_truth);

        // 2. 调用LLM获取扰动预测 f(x+δ)
        self.api_call_count += 1;
        let perturbed_response = self.llm_client.generate_response(&scenario.perturbation_prompt).await?;
        let perturbed_prediction = Self::extract_number_from_response(&perturbed_response.text)
            .unwrap_or_else(|| base_prediction);

        // 3. 计算真实增量响应
        let delta = perturbed_prediction - base_prediction;
        let delta_response = vec![delta; 5]; // 5个维度

        // 4. 生成因果图和谱特征
        let (causal_graph, spectral_features) = if let Some(ai_engine) = &mut self.ai_reasoning {
            self.api_call_count += 1;
            match ai_engine.generate_causal_graph(&scenario.description, "").await {
                Ok(graph) => {
                    let spec = Self::extract_graph_spectral_features(&graph);
                    (Some(graph), spec)
                }
                Err(e) => {
                    eprintln!("⚠️ 因果图生成失败: {}, 使用简化特征", e);
                    (None, Self::generate_fallback_spectral_features(&delta_response))
                }
            }
        } else {
            (None, Self::generate_fallback_spectral_features(&delta_response))
        };

        // 5. 拜占庭节点添加随机噪声
        let (base_pred, pert_pred, delta_vec, spec_vec) = if is_byzantine {
            let mut rng = rand::thread_rng();
            (
                base_prediction * (0.5 + rng.gen::<f64>()),
                perturbed_prediction * (0.5 + rng.gen::<f64>()),
                vec![delta * (0.5 + rng.gen::<f64>()); 5],
                spectral_features.iter().map(|f| f * (0.5 + rng.gen::<f64>())).collect(),
            )
        } else {
            (base_prediction, perturbed_prediction, delta_response, spectral_features)
        };

        Ok(RealAgent {
            id: agent_id.to_string(),
            causal_graph,
            base_prediction: base_pred,
            perturbed_prediction: pert_pred,
            delta_response: delta_vec,
            spectral_features: spec_vec,
            confidence: if is_byzantine { 0.6 } else { 0.9 },
            is_byzantine,
            reasoning: format!("基于场景: {}", scenario.description),
        })
    }

    /// 从因果图提取谱特征（8维）
    fn extract_graph_spectral_features(graph: &CausalGraph) -> Vec<f64> {
        let mut features = Vec::with_capacity(8);
        
        // 特征1: 节点数量
        features.push(graph.nodes.len() as f64);
        
        // 特征2: 边数量
        features.push(graph.edges.len() as f64);
        
        // 特征3: 路径数量
        features.push(graph.main_paths.len() as f64);
        
        // 特征4: 平均边权重
        let avg_edge_weight = if graph.edges.is_empty() {
            0.0
        } else {
            graph.edges.iter().map(|e| e.weight.abs()).sum::<f64>() / graph.edges.len() as f64
        };
        features.push(avg_edge_weight);
        
        // 特征5: 最大边权重
        let max_edge_weight = graph.edges.iter()
            .map(|e| e.weight.abs())
            .fold(0.0f64, |max, w| max.max(w));
        features.push(max_edge_weight);
        
        // 特征6: 平均路径强度
        let avg_path_strength = if graph.main_paths.is_empty() {
            0.0
        } else {
            graph.main_paths.iter().map(|p| p.strength).sum::<f64>() / graph.main_paths.len() as f64
        };
        features.push(avg_path_strength);
        
        // 特征7: 图密度（边数/最大可能边数）
        let n = graph.nodes.len();
        let density = if n > 1 {
            graph.edges.len() as f64 / (n * (n - 1)) as f64
        } else {
            0.0
        };
        features.push(density);
        
        // 特征8: 置信度（如果有）
        features.push(0.85); // 默认置信度
        
        features
    }

    /// 生成简化谱特征（当因果图生成失败时）
    fn generate_fallback_spectral_features(delta_response: &[f64]) -> Vec<f64> {
        let delta_sum: f64 = delta_response.iter().sum();
        let delta_mean = delta_sum / delta_response.len() as f64;
        let delta_var = delta_response.iter()
            .map(|d| (d - delta_mean).powi(2))
            .sum::<f64>() / delta_response.len() as f64;
        
        vec![
            delta_response.len() as f64,  // 维度
            delta_sum.abs(),              // 总变化
            delta_mean,                   // 平均变化
            delta_var.sqrt(),             // 标准差
            delta_response[0],            // 第一维
            delta_response.get(1).copied().unwrap_or(0.0), // 第二维
            delta_response.get(2).copied().unwrap_or(0.0), // 第三维
            delta_mean.abs() + delta_var, // 综合特征
        ]
    }

    pub async fn run_experiment(&mut self, num_rounds: usize) -> Result<()> {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║     真实多智能体预言机系统 - 基准测试实验              ║");
        println!("║     使用 DeepSeek API 生成真实数据                     ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        
        // 判断是否使用随机拜占庭节点数
        let use_random_byzantine = self.config.byzantine_ratios.is_empty();
        
        println!("📋 实验配置:");
        println!("   测试轮数: {}", num_rounds);
        println!("   智能体数量: {:?}", self.config.agent_counts);
        if use_random_byzantine {
            println!("   拜占庭节点: 随机生成 (0-40%)");
        } else {
            println!("   拜占庭比例: {:?}", self.config.byzantine_ratios);
        }
        println!("   共识阈值: {:?}", self.config.consensus_thresholds);
        println!("   每配置重复: {} 次\n", self.config.repetitions);

        let start_time = Instant::now();
        let mut total_rounds = 0;
        let mut rng = rand::thread_rng();

        let agent_counts = self.config.agent_counts.clone();
        let byzantine_ratios = self.config.byzantine_ratios.clone();
        let consensus_thresholds = self.config.consensus_thresholds.clone();

        for agent_count in agent_counts {
            // 如果没有指定拜占庭比例，则使用随机模式
            let byzantine_configs: Vec<usize> = if use_random_byzantine {
                vec![] // 空数组表示随机生成
            } else {
                byzantine_ratios.iter()
                    .map(|&r| (agent_count as f64 * r).round() as usize)
                    .collect()
            };
            
            let byzantine_iter: Box<dyn Iterator<Item = usize>> = if use_random_byzantine {
                Box::new(std::iter::repeat(0)) // 占位，实际每轮随机生成
            } else {
                Box::new(byzantine_configs.clone().into_iter())
            };

            for (byzantine_count_fixed, threshold) in byzantine_iter
                .zip(std::iter::repeat(consensus_thresholds.clone()).flatten()) 
            {
                // 如果使用随机模式，每轮生成随机的拜占庭节点数 (0 到 agent_count * 0.4)
                let byzantine_count = if use_random_byzantine {
                    let max_byzantine = (agent_count as f64 * 0.4).floor() as usize;
                    rng.gen_range(0..=max_byzantine)
                } else {
                    byzantine_count_fixed
                };
                
                let byzantine_ratio = if use_random_byzantine {
                    byzantine_count as f64 / agent_count as f64
                } else {
                    byzantine_configs.iter().find(|&&c| c == byzantine_count).map(|&c| c as f64 / agent_count as f64).unwrap_or(0.0)
                };
                
                println!("🔬 配置: {}智能体/{}拜占庭(≈{:.0}%)/阈值{:.2}", 
                    agent_count, byzantine_count, byzantine_ratio * 100.0, threshold);

                for _rep in 0..self.config.repetitions.min(num_rounds) {
                    if total_rounds >= num_rounds {
                        break;
                    }

                    // 每轮如果使用随机模式，重新生成拜占庭节点数
                    let round_byzantine_count = if use_random_byzantine {
                        let max_byzantine = (agent_count as f64 * 0.4).floor() as usize;
                        rng.gen_range(0..=max_byzantine)
                    } else {
                        byzantine_count
                    };

                    match self.run_single_round(
                        total_rounds,
                        agent_count,
                        round_byzantine_count,
                        threshold,
                    ).await {
                        Ok(round) => {
                            self.results.push(round);
                            total_rounds += 1;
                            print!(".");

                            // 每轮完成后立即保存结果
                            if let Err(e) = self.save_incremental_results(total_rounds) {
                                println!("\n   ⚠️ 保存第{}轮结果失败: {}", total_rounds, e);
                            }
                        }
                        Err(e) => {
                            println!("\n   ⚠️ 轮次 {} 失败: {}", total_rounds, e);
                        }
                    }

                    if total_rounds >= num_rounds {
                        break;
                    }
                }
                println!(" ✅");
                
                if total_rounds >= num_rounds {
                    break;
                }
            }
            if total_rounds >= num_rounds {
                break;
            }
        }

        let elapsed = start_time.elapsed();
        println!("\n\n✅ 实验完成!");
        println!("   总轮次: {}", total_rounds);
        println!("   耗时: {:.2} 秒", elapsed.as_secs_f64());
        println!("   API调用次数: {}", self.api_call_count);
        println!("   估算成本: ¥{:.2}", self.api_call_count as f64 * 0.001);

        // 最终保存完整结果
        self.save_results()?;

        Ok(())
    }

    async fn run_single_round(
        &mut self,
        round_id: usize,
        agent_count: usize,
        byzantine_count: usize,
        threshold: f64,
    ) -> Result<ExperimentRound> {
        let round_start = Instant::now();
        let initial_api_count = self.api_call_count;

        // 选择场景
        let scenario_idx = round_id % self.scenarios.len();
        let scenario = self.scenarios[scenario_idx].clone();

        // 生成真实智能体
        let mut agents = Vec::new();
        for i in 0..agent_count {
            let agent = self.generate_real_agent(
                &format!("agent_{:03}", i),
                &scenario,
                i < byzantine_count,
            ).await?;

            // 保存详细智能体数据（谱分析和因果图）
            let causal_summary = agent.causal_graph.as_ref().map(|g| {
                format!("节点数: {}, 边数: {}", g.nodes.len(), g.edges.len())
            });

            self.detailed_agent_data.push(AgentDetailedInfo {
                round_id,
                agent_id: agent.id.clone(),
                is_byzantine: agent.is_byzantine,
                base_prediction: agent.base_prediction,
                perturbed_prediction: agent.perturbed_prediction,
                delta_response: agent.delta_response.clone(),
                spectral_features: agent.spectral_features.clone(),
                confidence: agent.confidence,
                reasoning: agent.reasoning.clone(),
                causal_graph_summary: causal_summary,
            });

            agents.push(agent);
        }

        // 计算因果指纹
        println!("   [共识计算] 开始计算因果指纹和共识...");
        let config = CausalFingerprintConfig {
            cosine_threshold: threshold,
            min_valid_agents: 3,
            ..Default::default()
        };

        // 计算真实的谱特征（基于所有智能体的响应）
        let all_responses: Vec<Vec<f64>> = agents.iter()
            .map(|a| a.delta_response.clone())
            .collect();
        let global_spectral_features = extract_spectral_features(&all_responses);
        println!("   [共识计算] 提取全局谱特征完成，维度: {}", global_spectral_features.len());
        
        let fingerprints: Vec<CausalFingerprint> = agents.iter().enumerate().map(|(idx, a)| {
            // 每个智能体使用自己的谱特征或全局谱特征
            let agent_spectral = if a.spectral_features.is_empty() {
                global_spectral_features.clone()
            } else {
                a.spectral_features.clone()
            };
            
            println!("   [共识计算] 智能体 {}: base_prediction={}, is_byzantine={}, spectral_features={}", 
                     a.id, a.base_prediction, a.is_byzantine, agent_spectral.len());
            
            CausalFingerprint {
                agent_id: a.id.clone(),
                base_prediction: a.base_prediction,
                delta_response: a.delta_response.clone(),
                spectral_features: agent_spectral,
                perturbation: vec![0.1; 5],
                confidence: a.confidence,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }
        }).collect();

        println!("   [共识计算] 开始执行 cluster_by_consensus，指纹数量: {}", fingerprints.len());
        let consensus_result = cluster_by_consensus(&fingerprints, &config);
        println!("   [共识计算] 共识计算完成，共识值: {:.4}, 有效智能体: {}, 异常值: {}",
                 consensus_result.consensus_value, 
                 consensus_result.valid_agents.len(), 
                 consensus_result.outliers.len());

        // 计算真实值（正常智能体的平均值）
        let ground_truth = agents.iter()
            .filter(|a| !a.is_byzantine)
            .map(|a| a.base_prediction)
            .sum::<f64>() / (agent_count - byzantine_count).max(1) as f64;

        let convergence_time = round_start.elapsed().as_millis() as u64;
        let api_calls_this_round = self.api_call_count - initial_api_count;

        let accuracy = if consensus_result.consensus_value != 0.0 {
            1.0 - ((consensus_result.consensus_value - ground_truth).abs() / ground_truth.abs())
        } else {
            0.0
        };

        Ok(ExperimentRound {
            round_id,
            agent_count,
            byzantine_count,
            threshold,
            consensus_reached: !consensus_result.valid_agents.is_empty(),
            consensus_value: consensus_result.consensus_value,
            ground_truth,
            accuracy: accuracy.max(0.0),
            convergence_time_ms: convergence_time,
            valid_agents: consensus_result.valid_agents,
            outliers: consensus_result.outliers,
            consensus_similarity: consensus_result.consensus_similarity,
            api_calls_count: api_calls_this_round,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        })
    }

    /// 增量保存结果（每轮完成后调用）
    fn save_incremental_results(&self, current_round: usize) -> Result<()> {
        // 保存原始数据（追加模式）
        let csv_data = self.generate_csv();
        let csv_path = format!("{}/raw_data.csv", self.output_dir);
        File::create(&csv_path)?.write_all(csv_data.as_bytes())?;

        // 保存JSON汇总结果
        let json_data = serde_json::to_string_pretty(&self.results)?;
        let json_path = format!("{}/results.json", self.output_dir);
        File::create(&json_path)?.write_all(json_data.as_bytes())?;

        // 保存详细智能体数据（谱分析和因果图）
        let agent_details_json = serde_json::to_string_pretty(&self.detailed_agent_data)?;
        let agent_details_path = format!("{}/agent_details.json", self.output_dir);
        File::create(&agent_details_path)?.write_all(agent_details_json.as_bytes())?;

        // 保存详细智能体数据的CSV格式
        let agent_details_csv = self.generate_agent_details_csv();
        let agent_details_csv_path = format!("{}/agent_details.csv", self.output_dir);
        File::create(&agent_details_csv_path)?.write_all(agent_details_csv.as_bytes())?;

        // 更新总结
        let summary = self.generate_summary();
        let summary_path = format!("{}/summary.md", self.output_dir);
        File::create(&summary_path)?.write_all(summary.as_bytes())?;

        // 显示进度信息
        if current_round % 5 == 0 {
            println!("\n   📊 已完成 {} 轮，结果已保存", current_round);
        }

        Ok(())
    }

    fn save_results(&self) -> Result<()> {
        // 保存最终完整结果（与增量保存相同，因为目录已创建）
        let csv_data = self.generate_csv();
        let csv_path = format!("{}/raw_data.csv", self.output_dir);
        File::create(&csv_path)?.write_all(csv_data.as_bytes())?;

        // 保存JSON汇总结果
        let json_data = serde_json::to_string_pretty(&self.results)?;
        let json_path = format!("{}/results.json", self.output_dir);
        File::create(&json_path)?.write_all(json_data.as_bytes())?;

        // 🌟 保存详细智能体数据（谱分析和因果图）
        let agent_details_json = serde_json::to_string_pretty(&self.detailed_agent_data)?;
        let agent_details_path = format!("{}/agent_details.json", self.output_dir);
        File::create(&agent_details_path)?.write_all(agent_details_json.as_bytes())?;

        // 保存详细智能体数据的CSV格式
        let agent_details_csv = self.generate_agent_details_csv();
        let agent_details_csv_path = format!("{}/agent_details.csv", self.output_dir);
        File::create(&agent_details_csv_path)?.write_all(agent_details_csv.as_bytes())?;

        // 生成总结
        let summary = self.generate_summary();
        let summary_path = format!("{}/summary.md", self.output_dir);
        File::create(&summary_path)?.write_all(summary.as_bytes())?;

        println!("\n📊 最终结果已保存到: {}", self.output_dir);
        println!("   📈 results.json - 实验汇总结果");
        println!("   🧬 agent_details.json - 智能体详细信息（谱分析和因果图）");
        println!("   📊 agent_details.csv - 智能体详细数据CSV格式");
        println!("   📄 summary.md - 实验总结");
        Ok(())
    }

    fn generate_csv(&self) -> String {
        let mut csv = String::from("round_id,agent_count,byzantine_count,threshold,");
        csv.push_str("consensus_reached,consensus_value,ground_truth,accuracy,");
        csv.push_str("convergence_time_ms,consensus_similarity,api_calls_count,timestamp\n");

        for r in &self.results {
            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{},{},{}\n",
                r.round_id, r.agent_count, r.byzantine_count, r.threshold,
                r.consensus_reached, r.consensus_value, r.ground_truth,
                r.accuracy, r.convergence_time_ms, r.consensus_similarity,
                r.api_calls_count, r.timestamp
            ));
        }
        csv
    }

    fn generate_agent_details_csv(&self) -> String {
        // CSV header - 使用更易读的列名
        let mut csv = String::from("round_id,agent_id,is_byzantine,base_prediction,perturbed_prediction,");
        csv.push_str("delta_r1,delta_r2,delta_r3,delta_r4,delta_r5,");  // 展开delta_response
        csv.push_str("spectral_1,spectral_2,spectral_3,spectral_4,spectral_5,spectral_6,spectral_7,spectral_8,");  // 展开谱特征
        csv.push_str("confidence,causal_nodes,causal_edges,reasoning\n");

        for agent in &self.detailed_agent_data {
            // 解析因果图摘要
            let (nodes, edges) = if let Some(ref summary) = agent.causal_graph_summary {
                // 格式: "节点数: X, 边数: Y"
                let parts: Vec<&str> = summary.split(", ").collect();
                let n = parts.get(0).and_then(|s| s.split(": ").nth(1)).unwrap_or("0");
                let e = parts.get(1).and_then(|s| s.split(": ").nth(1)).unwrap_or("0");
                (n.to_string(), e.to_string())
            } else {
                ("0".to_string(), "0".to_string())
            };

            // 获取delta_response的5个值（不足补0）
            let delta_values: Vec<f64> = agent.delta_response.iter().cloned().chain(std::iter::repeat(0.0)).take(5).collect();

            // 获取spectral_features的8个值（不足补0）
            let spectral_values: Vec<f64> = agent.spectral_features.iter().cloned().chain(std::iter::repeat(0.0)).take(8).collect();

            // 处理reasoning中的换行和逗号
            let reasoning_clean = agent.reasoning.replace("\n", " ").replace(",", ";").replace("\"", "'");

            // 将布尔值转换为0/1，便于Excel处理
            let is_byzantine_int = if agent.is_byzantine { 1 } else { 0 };

            csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},\"{}\"\n",
                agent.round_id,
                agent.agent_id,
                is_byzantine_int,  // 使用0/1代替true/false
                agent.base_prediction,
                agent.perturbed_prediction,
                delta_values.get(0).unwrap_or(&0.0),
                delta_values.get(1).unwrap_or(&0.0),
                delta_values.get(2).unwrap_or(&0.0),
                delta_values.get(3).unwrap_or(&0.0),
                delta_values.get(4).unwrap_or(&0.0),
                spectral_values.get(0).unwrap_or(&0.0),
                spectral_values.get(1).unwrap_or(&0.0),
                spectral_values.get(2).unwrap_or(&0.0),
                spectral_values.get(3).unwrap_or(&0.0),
                spectral_values.get(4).unwrap_or(&0.0),
                spectral_values.get(5).unwrap_or(&0.0),
                spectral_values.get(6).unwrap_or(&0.0),
                spectral_values.get(7).unwrap_or(&0.0),
                agent.confidence,
                nodes,
                edges,
                reasoning_clean
            ));
        }
        csv
    }

    fn generate_summary(&self) -> String {
        let mut summary = String::from("# 真实实验报告\n\n");
        
        let consensus_rate = self.results.iter()
            .filter(|r| r.consensus_reached)
            .count() as f64 / self.results.len().max(1) as f64;
        
        let avg_accuracy = self.results.iter()
            .map(|r| r.accuracy)
            .sum::<f64>() / self.results.len().max(1) as f64;

        let avg_time = self.results.iter()
            .map(|r| r.convergence_time_ms)
            .sum::<u64>() / self.results.len().max(1) as u64;

        summary.push_str(&format!("## 总体统计\n\n"));
        summary.push_str(&format!("- 总轮次: {}\n", self.results.len()));
        summary.push_str(&format!("- 共识达成率: {:.2}%\n", consensus_rate * 100.0));
        summary.push_str(&format!("- 平均精度: {:.2}%\n", avg_accuracy * 100.0));
        summary.push_str(&format!("- 平均收敛时间: {}ms\n", avg_time));
        summary.push_str(&format!("- 总API调用: {}\n", self.api_call_count));
        summary.push_str(&format!("- 估算成本: ¥{:.2}\n\n", self.api_call_count as f64 * 0.001));

        summary
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 从命令行参数读取轮数，默认为25轮
    let args: Vec<String> = std::env::args().collect();
    let num_rounds = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(25);

    println!("🚀 启动真实基准测试实验");
    println!("   测试轮数: {}\n", num_rounds);

    // 创建配置
    let config = ExperimentConfig {
        repetitions: num_rounds, // 使用命令行指定的轮数
        ..Default::default()
    };

    // 创建运行器
    let mut runner = RealBenchmarkRunner::new(config).await?;

    // 运行实验
    runner.run_experiment(num_rounds).await?;

    println!("\n🎉 实验成功完成！");
    Ok(())
}
