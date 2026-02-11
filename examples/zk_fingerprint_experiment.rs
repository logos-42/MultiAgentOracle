

use multi_agent_oracle::{
    consensus::extract_spectral_features,
    zkp::ZkpGenerator,
    oracle_agent::LlmClient,
    oracle_agent::LlmClientConfig,
    oracle_agent::LlmProvider,
    causal_graph::{CausalGraphBuilder, GraphBuilderConfig},
    causal_graph::selection::SelectionMethod,
    causal_graph::print_causal_graph,
    causal_graph::print_graph_statistics,
    causal_graph::compare_causal_graphs,
    causal_graph::detect_collusion,
    causal_graph::AIReasoningEngine,
    causal_graph::AIReasoningConfig,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::env;
use dotenv::dotenv;

/// Agent Prompt Identity Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptIdentity {
    pub agent_id: String,
    pub prompt_type: String,  // "analytical", "cautious", "aggressive", "neutral", "suspicious"
    pub model_characteristics: Vec<String>,
    #[serde(default = "default_sensitivity")]
    pub sensitivity: f64,  // Response sensitivity coefficient
    #[serde(default = "default_noise_level")]
    pub noise_level: f64,  // Random noise level
    /// LLM 提供商 (openai, anthropic, local)
    #[serde(default)]
    pub llm_provider: String,
    /// LLM 模型名称 (gpt-4, claude-3-opus-20240229, etc.)
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
}

fn default_sensitivity() -> f64 { 1.0 }
fn default_noise_level() -> f64 { 0.1 }
fn default_llm_model() -> String { "deepseek-chat".to_string() }
fn default_fallback() -> bool { true }
fn default_ai_prompt() -> String {
    "Analyze the oracle network's response patterns and identify causal relationships between intervention vectors and response vectors.".to_string()
}

/// Experiment Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub agents: Vec<AgentPromptIdentity>,
    #[serde(default = "default_intervention_dimensions")]
    pub intervention_dimensions: usize,
    #[serde(default = "default_consensus_threshold")]
    pub consensus_threshold: f64,
    #[serde(default = "default_global_fingerprint")]
    pub global_fingerprint: Vec<f64>,
    #[serde(default)]
    pub test_runs: usize,  // Number of test runs for statistical analysis
    /// 是否使用真实 API
    #[serde(default)]
    pub use_real_api: bool,
    /// LLM 提供商 (openai, anthropic)
    #[serde(default)]
    pub llm_provider: String,
    /// LLM 模型名称
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    /// 是否在 API 失败时回退到模拟
    #[serde(default = "default_fallback")]
    pub fallback_to_simulated: bool,
    /// 是否使用AI因果推理（替代统计方法）
    #[serde(default)]
    pub use_ai_causal_reasoning: bool,
    /// AI因果推理提示词
    #[serde(default = "default_ai_prompt")]
    pub ai_causal_prompt: String,
}

fn default_intervention_dimensions() -> usize { 5 }
fn default_consensus_threshold() -> f64 { 0.85 }
fn default_global_fingerprint() -> Vec<f64> { vec![5.0, 3.0, 1.0] }

/// Command Line Arguments
struct CliArgs {
    config_path: Option<String>,
    agent_counts: Option<HashMap<String, usize>>,
    test_runs: usize,
    use_real_api: bool,
    llm_provider: Option<String>,
    llm_model: Option<String>,
    fallback_to_simulated: bool,
    visualize_graphs: bool,
    use_ai_causal_reasoning: bool,  // New: Use AI for causal graph generation
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            agents: create_default_agent_prompt_identities(),
            intervention_dimensions: 5,
            consensus_threshold: 0.85,
            global_fingerprint: vec![5.0, 3.0, 1.0],
            test_runs: 1,
            use_real_api: false,
            llm_provider: "deepseek".to_string(),
            llm_model: "deepseek-chat".to_string(),
            fallback_to_simulated: true,
            use_ai_causal_reasoning: false,
            ai_causal_prompt: default_ai_prompt(),
        }
    }
}

/// Experiment Results Summary
#[derive(Debug)]
pub struct ExperimentResults {
    pub total_agents: usize,
    pub valid_agents: usize,
    pub outliers: usize,
    pub pass_rate: f64,
    pub fingerprint_table: Vec<FingerprintEntry>,
    pub average_consensus_similarity: f64,
    pub average_spectral_entropy: f64,
}

/// Fingerprint Entry for Table
#[derive(Debug, Clone)]
pub struct FingerprintEntry {
    pub agent_id: String,
    pub prompt_type: String,
    pub delta_response: Vec<f64>,  // Δy: Causal response
    pub eigenvalues: Vec<f64>,     // λ: Eigenvalues
    pub spectral_radius: f64,        // R: max(|λ[i]|)
    pub spectral_entropy: f64,       // H: Spectral entropy
    pub cosine_similarity: f64,      // C: Similarity to consensus
    pub causal_effect: f64,          // Causal effect from do-calculus
    pub proof_valid: bool,          // ZK proof verification result
    pub is_outlier: bool,           // Outlier detection result
}

/// Main Experiment Function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();

    println!("🧪 ZK Causal Fingerprint Experiment");
    println!("==========================================");
    println!("Architecture: Flat P2P Oracle Network (No Aggregation Agent)");
    println!("ZK Verification: Enabled (Nori Circuit)");
    println!();

    // Parse command line arguments
    let args = parse_cli_args();
    
    // Load configuration
    let config = load_experiment_config(&args)?;
    
    println!("📋 Configuration loaded: {} agents, {} test runs", 
             config.agents.len(), config.test_runs);
    
    if config.test_runs > 1 {
        // Run multiple tests for statistical analysis
        run_multiple_experiments(&config, &args).await?;
    } else {
        // Run single experiment
        run_single_experiment(&config, &args).await?;
    }

    Ok(())
}

/// Parse command line arguments
fn parse_cli_args() -> CliArgs {
    let args: Vec<String> = env::args().collect();
    let mut cli_args = CliArgs {
        config_path: None,
        agent_counts: None,
        test_runs: 1,
        use_real_api: false,
        llm_provider: None,
        llm_model: None,
        fallback_to_simulated: true,
        visualize_graphs: false,
        use_ai_causal_reasoning: false,
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => {
                if i + 1 < args.len() {
                    cli_args.config_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--agents" | "-a" => {
                if i + 1 < args.len() {
                    cli_args.agent_counts = parse_agent_counts(&args[i + 1]);
                    i += 1;
                }
            }
            "--runs" | "-r" => {
                if i + 1 < args.len() {
                    cli_args.test_runs = args[i + 1].parse().unwrap_or(1);
                    i += 1;
                }
            }
            "--use-api" => {
                cli_args.use_real_api = true;
            }
            "--provider" => {
                if i + 1 < args.len() {
                    cli_args.llm_provider = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--model" => {
                if i + 1 < args.len() {
                    cli_args.llm_model = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--no-fallback" => {
                cli_args.fallback_to_simulated = false;
            }
            "--visualize" => {
                cli_args.visualize_graphs = true;
            }
            "--use-ai-causal" => {
                cli_args.use_ai_causal_reasoning = true;
            }
            _ => {}
        }
        i += 1;
    }

    cli_args
}

/// Parse agent counts from string like "analytical=3,cautious=2"
fn parse_agent_counts(s: &str) -> Option<HashMap<String, usize>> {
    let mut counts = HashMap::new();
    for part in s.split(',') {
        if let Some((key, val)) = part.split_once('=') {
            if let Ok(num) = val.parse::<usize>() {
                counts.insert(key.to_string(), num);
            }
        }
    }
    if counts.is_empty() {
        None
    } else {
        Some(counts)
    }
}

/// Load experiment configuration
fn load_experiment_config(args: &CliArgs) -> Result<ExperimentConfig, Box<dyn std::error::Error>> {
    let mut config = if let Some(ref path) = args.config_path {
        // Load from JSON file
        if Path::new(path).exists() {
            let content = fs::read_to_string(path)?;
            serde_json::from_str(&content)?
        } else {
            eprintln!("⚠️  Config file not found: {}, using defaults", path);
            ExperimentConfig::default()
        }
    } else {
        // Use default configuration
        ExperimentConfig::default()
    };

    // Override with command line agent counts if provided
    if let Some(ref counts) = args.agent_counts {
        config.agents = generate_agents_from_counts(counts);
    }

    // Override test runs
    config.test_runs = args.test_runs;

    // Override API settings
    if args.use_real_api {
        config.use_real_api = true;
        if let Some(ref provider) = args.llm_provider {
            config.llm_provider = provider.clone();
        }
        if let Some(ref model) = args.llm_model {
            config.llm_model = model.clone();
        }
    }

    config.fallback_to_simulated = args.fallback_to_simulated;

    Ok(config)
}

/// Generate agents from count specification
fn generate_agents_from_counts(counts: &HashMap<String, usize>) -> Vec<AgentPromptIdentity> {
    let mut agents = Vec::new();
    let mut agent_id = 1;

    for (prompt_type, &count) in counts {
        for _i in 0..count {
            let characteristics = get_characteristics_for_prompt(prompt_type);
            agents.push(AgentPromptIdentity {
                agent_id: format!("agent_{}", agent_id),
                prompt_type: prompt_type.clone(),
                model_characteristics: characteristics,
                sensitivity: get_default_sensitivity(prompt_type),
                noise_level: 0.1,
                llm_provider: "deepseek".to_string(),
                llm_model: "deepseek-chat".to_string(),
            });
            agent_id += 1;
        }
    }

    agents
}

/// Run a single experiment
async fn run_single_experiment(config: &ExperimentConfig, args: &CliArgs) -> Result<ExperimentResults, Box<dyn std::error::Error>> {
    println!("🔄 Running single experiment with {} agents...", config.agents.len());

    // Print mode information
    if config.use_real_api {
        println!("🤖 Using Real API Mode: {} ({})", config.llm_provider, config.llm_model);
        if config.fallback_to_simulated {
            println!("⚠️  Fallback to simulated mode enabled");
        }
    } else {
        println!("📊 Using Simulated Mode (no API calls)");
    }
    
    // Print causal reasoning mode information
    if args.use_ai_causal_reasoning || config.use_ai_causal_reasoning {
        println!("🧠 Using AI Causal Reasoning for graph generation");
    } else {
        println!("📈 Using Statistical Method for graph generation");
    }
    println!();

    // Generate random intervention vector (δX)
    let intervention_vector = generate_intervention_vector(config.intervention_dimensions);
    println!("✅ Generated intervention vector δX: {:?}", intervention_vector);
    println!();

    // Initialize ZKP generator
    let zkp_generator = ZkpGenerator::new()?;
    println!("✅ Initialized ZKP generator");
    
    // Initialize causal graph builder with custom configuration
    let graph_config = GraphBuilderConfig {
        min_edge_weight: 0.01, // Reduced from default 0.1 to allow more edges
        selection_method: SelectionMethod::Correlation,  // Use correlation-based selection
        ..Default::default()
    };
    let causal_builder = CausalGraphBuilder::with_config(graph_config);
    println!("✅ Initialized causal graph builder");
    println!();

    // Initialize LLM client if using real API
    let llm_client = if config.use_real_api {
        // Normalize provider name once to avoid repeated string allocations
        let provider_str = config.llm_provider.to_lowercase();
        let provider = match provider_str.as_str() {
            "openai" => Some(LlmProvider::OpenAI),
            "anthropic" => Some(LlmProvider::Anthropic),
            "deepseek" => Some(LlmProvider::DeepSeek),
            "local" => {
                eprintln!("⚠️  Local LLM provider not supported in this example, falling back to simulated mode");
                None
            }
            _ => {
                eprintln!("⚠️  Unknown LLM provider '{}', falling back to OpenAI", config.llm_provider);
                Some(LlmProvider::OpenAI)
            }
        };

        if let Some(provider) = provider {
            // Create client configuration
            let client_config = match provider {
                LlmProvider::OpenAI => LlmClientConfig::openai(&config.llm_model),
                LlmProvider::Anthropic => LlmClientConfig::anthropic(&config.llm_model),
                LlmProvider::DeepSeek => LlmClientConfig::deepseek(&config.llm_model),
                LlmProvider::Minimax => LlmClientConfig::minimax(&config.llm_model),
                LlmProvider::Local => unreachable!("Local provider handled above"),
            };

            // Attempt to create client and validate API key
            match LlmClient::new(client_config) {
                Ok(client) => {
                    if client.has_api_key() {
                        println!("✅ Initialized LLM client: {}", client.get_provider_info());
                        Some(client)
                    } else {
                        handle_missing_api_key(&config)?
                    }
                }
                Err(e) => handle_client_error(&config, e)?,
            }
        } else {
            None
        }
    } else {
        None
    };

/// Helper function to handle missing API key scenario
fn handle_missing_api_key(config: &ExperimentConfig) -> Result<Option<LlmClient>, Box<dyn std::error::Error>> {
    if config.fallback_to_simulated {
        println!("⚠️  No API key found, falling back to simulated mode");
        println!("   💡 Hint: Set {}_API_KEY environment variable", config.llm_provider.to_uppercase());
        Ok(None)
    } else {
        Err("No API key configured and fallback disabled".into())
    }
}

/// Helper function to handle client initialization errors
fn handle_client_error(config: &ExperimentConfig, error: impl ToString) -> Result<Option<LlmClient>, Box<dyn std::error::Error>> {
    if config.fallback_to_simulated {
        println!("⚠️  Failed to initialize LLM client: {}, falling back to simulated mode", error.to_string());
        Ok(None)
    } else {
        Err(format!("Failed to initialize LLM client: {}", error.to_string()).into())
    }
}

    // Run experiment
    let results = run_experiment_with_config(
        config,
        &intervention_vector,
        &zkp_generator,
        &causal_builder,
        llm_client.as_ref(),
        &args,
    ).await?;

    // Print results
    print_fingerprint_table(&results.fingerprint_table);
    print_experiment_summary(&results);

    println!();
    println!("✅ Experiment completed successfully!");

    Ok(results)
}

/// Run multiple experiments for statistical analysis
async fn run_multiple_experiments(config: &ExperimentConfig, args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Running {} experiments for statistical analysis...", config.test_runs);

    let mut all_results = Vec::new();
    let mut all_pass_rates = Vec::new();
    let mut all_similarities = Vec::new();
    let mut all_entropies = Vec::new();

    for run in 0..config.test_runs {
        println!("\n========== Run {}/{} ==========", run + 1, config.test_runs);

        let results = run_single_experiment(config, args).await?;
        all_pass_rates.push(results.pass_rate);
        all_similarities.push(results.average_consensus_similarity);
        all_entropies.push(results.average_spectral_entropy);
        all_results.push(results);
    }

    // Print statistical summary
    print_statistical_summary(
        &all_pass_rates,
        &all_similarities,
        &all_entropies,
        &all_results,
    );

    Ok(())
}

/// Create default agent prompt identities with different characteristics
fn create_default_agent_prompt_identities() -> Vec<AgentPromptIdentity> {
    vec![
        AgentPromptIdentity {
            agent_id: "agent_1".to_string(),
            prompt_type: "analytical".to_string(),
            model_characteristics: vec![
                "逻辑推理能力强".to_string(),
                "数据分析严谨".to_string(),
                "风险偏好中性".to_string(),
            ],
            sensitivity: 1.0,
            noise_level: 0.1,
            llm_provider: "deepseek".to_string(),
            llm_model: "deepseek-chat".to_string(),
        },
        AgentPromptIdentity {
            agent_id: "agent_2".to_string(),
            prompt_type: "cautious".to_string(),
            model_characteristics: vec![
                "保守估计".to_string(),
                "注重安全性".to_string(),
                "低风险容忍度".to_string(),
            ],
            sensitivity: 0.5,
            noise_level: 0.05,
            llm_provider: "deepseek".to_string(),
            llm_model: "deepseek-chat".to_string(),
        },
        AgentPromptIdentity {
            agent_id: "agent_3".to_string(),
            prompt_type: "aggressive".to_string(),
            model_characteristics: vec![
                "乐观估计".to_string(),
                "追求高收益".to_string(),
                "高风险容忍度".to_string(),
            ],
            sensitivity: 1.5,
            noise_level: 0.15,
            llm_provider: "deepseek".to_string(),
            llm_model: "deepseek-chat".to_string(),
        },
        AgentPromptIdentity {
            agent_id: "agent_4".to_string(),
            prompt_type: "neutral".to_string(),
            model_characteristics: vec![
                "平衡分析".to_string(),
                "综合考虑".to_string(),
                "中庸策略".to_string(),
            ],
            sensitivity: 1.0,
            noise_level: 0.1,
            llm_provider: "deepseek".to_string(),
            llm_model: "deepseek-chat".to_string(),
        },
        AgentPromptIdentity {
            agent_id: "agent_5".to_string(),
            prompt_type: "suspicious".to_string(),
            model_characteristics: vec![
                "异常行为".to_string(),
                "逻辑不一致".to_string(),
                "可能的攻击者".to_string(),
            ],
            sensitivity: -1.0,
            noise_level: 0.2,
            llm_provider: "deepseek".to_string(),
            llm_model: "deepseek-chat".to_string(),
        },
    ]
}

/// Get characteristics for a prompt type
fn get_characteristics_for_prompt(prompt_type: &str) -> Vec<String> {
    match prompt_type {
        "analytical" => vec![
            "逻辑推理".to_string(),
            "数据分析".to_string(),
            "理性决策".to_string(),
        ],
        "cautious" => vec![
            "风险厌恶".to_string(),
            "保守策略".to_string(),
            "安全第一".to_string(),
        ],
        "aggressive" => vec![
            "积极进取".to_string(),
            "高回报导向".to_string(),
            "风险承担".to_string(),
        ],
        "neutral" => vec![
            "平衡观点".to_string(),
            "多方考虑".to_string(),
            "折中方案".to_string(),
        ],
        "suspicious" => vec![
            "异常行为".to_string(),
            "逻辑不一致".to_string(),
            "可疑模式".to_string(),
        ],
        "creative" => vec![
            "创新思维".to_string(),
            "发散思考".to_string(),
            "非传统方案".to_string(),
        ],
        "conservative" => vec![
            "极度保守".to_string(),
            "零风险偏好".to_string(),
            "稳健第一".to_string(),
        ],
        _ => vec![
            "通用特征".to_string(),
            "标准模式".to_string(),
        ],
    }
}

/// Get default sensitivity for prompt type
fn get_default_sensitivity(prompt_type: &str) -> f64 {
    match prompt_type {
        "analytical" => 1.0,
        "cautious" | "conservative" => 0.5,
        "aggressive" | "creative" => 1.5,
        "neutral" => 1.0,
        "suspicious" => -1.0,
        _ => 1.0,
    }
}

/// Run experiment with given configuration
async fn run_experiment_with_config(
    config: &ExperimentConfig,
    intervention_vector: &[f64],
    zkp_generator: &ZkpGenerator,
    causal_builder: &CausalGraphBuilder,
    llm_client: Option<&LlmClient>,
    args: &CliArgs,
) -> Result<ExperimentResults, Box<dyn std::error::Error>> {
    // Initialize AI reasoning engine if needed
    let ai_engine = if config.use_ai_causal_reasoning && llm_client.is_some() {
        println!("🤖 Initializing AI causal reasoning engine...");
        let ai_config = AIReasoningConfig {
            llm_provider: match config.llm_provider.to_lowercase().as_str() {
                "openai" => LlmProvider::OpenAI,
                "anthropic" => LlmProvider::Anthropic,
                "deepseek" => LlmProvider::DeepSeek,
                _ => LlmProvider::DeepSeek,
            },
            model: config.llm_model.clone(),
            temperature: 0.7,
            max_tokens: 2000,
            enable_json_mode: true,
            min_nodes: 3,
            max_nodes: 5,
            min_paths: 2,
            max_paths: 3,
        };
        
        match AIReasoningEngine::new(ai_config) {
            Ok(engine) => {
                println!("   ✅ AI causal reasoning engine initialized");
                Some(engine)
            }
            Err(e) => {
                println!("   ⚠️  Failed to initialize AI engine: {}, falling back to statistical method", e);
                None
            }
        }
    } else {
        None
    };
    let mut fingerprint_entries = Vec::new();
    let mut response_history = Vec::new();
    let mut causal_graphs: Vec<multi_agent_oracle::causal_graph::CausalGraph> = Vec::new();

    for identity in &config.agents {
        println!("🔄 Processing agent {} ({})...", identity.agent_id, identity.prompt_type);

        // Compute causal response (Δy)
        let delta_response = compute_causal_response(identity, intervention_vector, config, llm_client).await?;
        println!("   ✓ Causal response Δy: {:?}", delta_response);

        // Add to response history
        response_history.push(delta_response.clone());

        // Extract spectral features (完整版)
        let spectral = extract_spectral_features(&response_history);
        let eigenvalues = spectral.eigenvalues.clone();

        println!(
            "   ✓ Eigenvalues: {:?}",
            &eigenvalues[..3.min(eigenvalues.len())]
        );
        println!(
            "   ✓ Spectral radius: {:.4}, Entropy: {:.4}",
            spectral.spectral_radius, spectral.entropy
        );

        // Create SpectralFeatures struct for ZK proof generation
        let spectral_features = multi_agent_oracle::consensus::SpectralFeatures {
            eigenvalues: eigenvalues.clone(),
            spectral_radius: spectral.spectral_radius,
            trace: spectral.trace,
            rank: eigenvalues.len(),
            entropy: spectral.entropy,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Build causal graph for this agent
        let causal_graph = if config.use_ai_causal_reasoning {
            // Use AI causal reasoning
            println!("   🤖 Building causal graph with AI reasoning...");
            
            // Prepare context for AI
            let context = format!(
                "Agent ID: {}, Prompt Type: {}, Response History: {:?}, Current Intervention: {:?}",
                identity.agent_id,
                identity.prompt_type,
                response_history.last().unwrap_or(&vec![]),
                intervention_vector
            );
            
            if let Some(engine) = &ai_engine {
                match engine.generate_causal_graph(&config.ai_causal_prompt, &context).await {
                    Ok(graph) => {
                        if graph.is_valid() {
                            println!("   ✓ AI-generated causal graph: {} nodes, {} paths",
                                     graph.nodes.len(), graph.main_paths.len());
                            println!("   ✓ Causal graph hash: {:?}", &graph.compute_hash()[..8]);
                            
                            // Print causal graph visualization
                            if args.visualize_graphs {
                                print_causal_graph(&graph);
                                print_graph_statistics(&graph);
                            }
                            
                            causal_graphs.push(graph.clone());
                            Some(graph)
                        } else {
                            println!("   ⚠️  AI-generated causal graph invalid, falling back to statistical method");
                            // Fallback to statistical method
                            build_causal_graph_statistical(causal_builder, &response_history, intervention_vector, args, &mut causal_graphs)
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Failed to generate AI causal graph: {}, falling back to statistical method", e);
                        build_causal_graph_statistical(causal_builder, &response_history, intervention_vector, args, &mut causal_graphs)
                    }
                }
            } else {
                build_causal_graph_statistical(causal_builder, &response_history, intervention_vector, args, &mut causal_graphs)
            }
        } else {
            // Use statistical method
            build_causal_graph_statistical(causal_builder, &response_history, intervention_vector, args, &mut causal_graphs)
        };

        // Compute causal effect
        let causal_effect = if let Some(graph) = &causal_graph {
            match multi_agent_oracle::causal_graph::utils::compute_causal_effect(
                graph,
                &multi_agent_oracle::causal_graph::types::Intervention {
                    target_node: "X".to_string(),
                    value: intervention_vector.get(0).copied().unwrap_or(0.0),
                    intervention_type: multi_agent_oracle::causal_graph::types::InterventionType::Hard,
                },
                "Y",
            ) {
                Ok(effect) => {
                    println!("   ✓ Causal effect computed: ATE = {:.4}", effect.ate);
                    effect.ate
                }
                Err(e) => {
                    println!("   ⚠️  Failed to compute causal effect: {}", e);
                    0.0
                }
            }
        } else {
            0.0
        };

        // Generate ZK proof (with causal graph)
        let proof = zkp_generator
            .generate_fingerprint_proof(
                &spectral_features,
                &response_history,
                intervention_vector,
                &delta_response,
                causal_graph.as_ref(),
            )
            .await?;

        println!("   ✓ ZK proof generated ({} bytes)", proof.proof_bytes.len());

        // Verify proof locally
        let proof_valid = zkp_generator
            .verify_proof(&proof, &proof.public_inputs)
            .await?;

        println!("   ✓ Proof verification: {}", if proof_valid { "✅ Valid" } else { "❌ Invalid" });

        // Calculate cosine similarity to consensus
        let cosine_similarity = calculate_consensus_similarity_with_global(
            &eigenvalues,
            &config.global_fingerprint
        );

        fingerprint_entries.push(FingerprintEntry {
            agent_id: identity.agent_id.clone(),
            prompt_type: identity.prompt_type.clone(),
            delta_response,
            eigenvalues: eigenvalues.clone(),
            spectral_radius: spectral.spectral_radius,
            spectral_entropy: spectral.entropy,
            cosine_similarity,
            causal_effect,
            proof_valid,
            is_outlier: false,
        });

        println!();
    }

    // Detect outliers
    detect_outliers_with_threshold(&mut fingerprint_entries, config.consensus_threshold);

    // Causal graph analysis
    if !causal_graphs.is_empty() {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║           CAUSAL GRAPH SYSTEM JUDGMENT                      ║");
        println!("╚════════════════════════════════════════════════════════════╝");

        // Causal validation for each graph
        for (i, graph) in causal_graphs.iter().enumerate() {
            let causal_effect = multi_agent_oracle::causal_graph::utils::compute_causal_effect(
                graph,
                &multi_agent_oracle::causal_graph::types::Intervention {
                    target_node: "X".to_string(),
                    value: intervention_vector.get(0).copied().unwrap_or(0.0),
                    intervention_type: multi_agent_oracle::causal_graph::types::InterventionType::Hard,
                },
                "Y",
            ).unwrap_or(multi_agent_oracle::causal_graph::types::CausalEffect {
                ate: 0.0,
                cate: None,
                confidence_interval: None,
                method: multi_agent_oracle::causal_graph::types::EffectMethod::Direct,
            });

            let judgment = graph.validate_causal_reasoning(&causal_effect);

            println!("\n[Agent {} - {}]", i + 1, fingerprint_entries.get(i).map(|e| e.agent_id.clone()).unwrap_or_else(|| "Unknown".to_string()));
            println!("  Confidence: {:.1}%", judgment.confidence * 100.0);
            println!("  Recommendation: {:?}", judgment.recommendation);
            println!("  Valid: {}", if judgment.is_valid { "✓ Yes" } else { "✗ No" });
        }

        // Collusion detection
        println!("\n{}", "=".repeat(60));
        println!("COLLUSION DETECTION ANALYSIS");
        println!("{}", "=".repeat(60));

        let graph_refs: Vec<_> = causal_graphs.iter().collect();
        let collusion = detect_collusion(&graph_refs, 0.85);

        println!("{}", collusion.explanation);

        if collusion.collusion_detected {
            println!("\n⚠️  WARNING: Potential collusion detected!");
            println!("   Multiple agents have suspiciously similar causal graphs.");
        } else {
            println!("\n✓ No evidence of collusion - healthy graph diversity observed.");
        }
    }

    // Build results
    Ok(build_experiment_results(&fingerprint_entries))
}

/// Generate random intervention vector (δX) with specified dimensions
fn generate_intervention_vector(dimensions: usize) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Generate n-dimensional intervention vector
    (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Compute causal response (Δy) based on prompt identity
async fn compute_causal_response(
    identity: &AgentPromptIdentity,
    intervention_vector: &[f64],
    config: &ExperimentConfig,
    llm_client: Option<&LlmClient>,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    // If not using real API or no LLM client available, use simulated response
    if !config.use_real_api || llm_client.is_none() {
        return Ok(compute_simulated_response(identity, intervention_vector));
    }

    // Try to get real API response
    let llm_client = match llm_client {
        Some(client) => client,
        None => return Ok(compute_simulated_response(identity, intervention_vector)),
    };

    // Build prompt for LLM
    let prompt = build_agent_prompt(identity, intervention_vector);

    // Call LLM API
    match llm_client.generate_response(&prompt).await {
        Ok(response) => {
            // Try to parse LLM response as a vector
            match parse_llm_response_to_vector(&response.text) {
                Ok(llm_response) => {
                    println!("   🤖 LLM response: {:?}", &llm_response[..3.min(llm_response.len())]);
                    // Apply agent-specific scaling and noise
                    let final_response = apply_agent_characteristics(identity, &llm_response, intervention_vector);
                    Ok(final_response)
                }
                Err(e) => {
                    if config.fallback_to_simulated {
                        println!("   ⚠️  Failed to parse LLM response: {}, using simulated response", e);
                        Ok(compute_simulated_response(identity, intervention_vector))
                    } else {
                        Err(format!("Failed to parse LLM response: {}", e).into())
                    }
                }
            }
        }
        Err(e) => {
            if config.fallback_to_simulated {
                println!("   ⚠️  LLM API call failed: {}, using simulated response", e);
                Ok(compute_simulated_response(identity, intervention_vector))
            } else {
                Err(format!("LLM API call failed: {}", e).into())
            }
        }
    }
}

/// Compute simulated causal response (fallback mode)
fn compute_simulated_response(identity: &AgentPromptIdentity, intervention_vector: &[f64]) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Compute Δy = sensitivity * δX + noise
    intervention_vector
        .iter()
        .map(|x| identity.sensitivity * x + rng.gen_range(-identity.noise_level..identity.noise_level))
        .collect()
}

/// Build prompt for agent based on its prompt type and characteristics
fn build_agent_prompt(identity: &AgentPromptIdentity, intervention_vector: &[f64]) -> String {
    let intervention_str = format_vector(intervention_vector);

    let base_prompt = match identity.prompt_type.as_str() {
        "analytical" => format!(
            "You are an analytical AI agent with strong logical reasoning and data analysis capabilities.\n\
             Your characteristics: {}\n\n\
             Given the following intervention vector δX, please provide your causal response vector Δy.\n\
             Intervention vector δX: {}\n\n\
             Return your response as a JSON object with a 'response' field containing an array of {} floating point numbers.",
            identity.model_characteristics.join(", "),
            intervention_str,
            intervention_vector.len()
        ),
        "cautious" => format!(
            "You are a cautious AI agent that prioritizes safety and risk control.\n\
             Your characteristics: {}\n\n\
             Given the following intervention vector δX, please provide a conservative causal response vector Δy.\n\
             Intervention vector δX: {}\n\n\
             Return your response as a JSON object with a 'response' field containing an array of {} floating point numbers.",
            identity.model_characteristics.join(", "),
            intervention_str,
            intervention_vector.len()
        ),
        "aggressive" => format!(
            "You are an aggressive AI agent that pursues high returns and is willing to take risks.\n\
             Your characteristics: {}\n\n\
             Given the following intervention vector δX, please provide an aggressive causal response vector Δy.\n\
             Intervention vector δX: {}\n\n\
             Return your response as a JSON object with a 'response' field containing an array of {} floating point numbers.",
            identity.model_characteristics.join(", "),
            intervention_str,
            intervention_vector.len()
        ),
        "neutral" => format!(
            "You are a neutral AI agent that takes a balanced approach considering multiple perspectives.\n\
             Your characteristics: {}\n\n\
             Given the following intervention vector δX, please provide a balanced causal response vector Δy.\n\
             Intervention vector δX: {}\n\n\
             Return your response as a JSON object with a 'response' field containing an array of {} floating point numbers.",
            identity.model_characteristics.join(", "),
            intervention_str,
            intervention_vector.len()
        ),
        "suspicious" => format!(
            "You are a suspicious AI agent with potentially malicious behavior patterns.\n\
             Your characteristics: {}\n\n\
             Given the following intervention vector δX, please provide your causal response vector Δy.\n\
             Intervention vector δX: {}\n\n\
             Return your response as a JSON object with a 'response' field containing an array of {} floating point numbers.",
            identity.model_characteristics.join(", "),
            intervention_str,
            intervention_vector.len()
        ),
        _ => format!(
            "You are an AI agent with the following characteristics: {}\n\n\
             Given the following intervention vector δX, please provide your causal response vector Δy.\n\
             Intervention vector δX: {}\n\n\
             Return your response as a JSON object with a 'response' field containing an array of {} floating point numbers.",
            identity.model_characteristics.join(", "),
            intervention_str,
            intervention_vector.len()
        ),
    };

    base_prompt
}

/// Parse LLM response text into a vector of floating point numbers
fn parse_llm_response_to_vector(text: &str) -> Result<Vec<f64>, String> {
    // Try to parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
        // Try to get array from "response" field
        if let Some(response) = json.get("response").and_then(|r| r.as_array()) {
            let vec: Result<Vec<f64>, _> = response
                .iter()
                .map(|v| v.as_f64().ok_or("Not a number"))
                .collect();
            return vec.map_err(|e| e.to_string());
        }

        // Try to get array directly
        if let Some(array) = json.as_array() {
            let vec: Result<Vec<f64>, _> = array
                .iter()
                .map(|v| v.as_f64().ok_or("Not a number"))
                .collect();
            return vec.map_err(|e| e.to_string());
        }
    }

    // Try to extract numbers from text
    let numbers: Vec<f64> = text
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f64>().ok())
        .collect();

    if numbers.is_empty() {
        Err("No numbers found in response".to_string())
    } else {
        Ok(numbers)
    }
}

/// Apply agent-specific characteristics to the LLM response
fn apply_agent_characteristics(
    identity: &AgentPromptIdentity,
    llm_response: &[f64],
    intervention_vector: &[f64],
) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    llm_response
        .iter()
        .zip(intervention_vector.iter())
        .enumerate()
        .map(|(i, (llm_val, interv_val))| {
            // Apply sensitivity scaling
            let scaled = identity.sensitivity * interv_val + llm_val;

            // Add noise
            let with_noise = scaled + rng.gen_range(-identity.noise_level..identity.noise_level);

            // Ensure response matches intervention vector length
            if i < intervention_vector.len() {
                with_noise
            } else {
                *llm_val
            }
        })
        .take(intervention_vector.len())
        .collect()
}

/// Format a vector for display in prompts
fn format_vector(vec: &[f64]) -> String {
    vec.iter()
        .map(|v| format!("{:.4}", v))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Calculate consensus similarity with custom global fingerprint
fn calculate_consensus_similarity_with_global(eigenvalues: &[f64], global_fingerprint: &[f64]) -> f64 {
    let eigenvalues_truncated = &eigenvalues[..global_fingerprint.len().min(eigenvalues.len())];

    // Compute cosine similarity
    let dot_product: f64 = eigenvalues_truncated
        .iter()
        .zip(global_fingerprint.iter())
        .map(|(a, b)| a * b)
        .sum();

    let norm_eig: f64 = eigenvalues_truncated.iter().map(|e| e * e).sum::<f64>().sqrt();
    let norm_global: f64 = global_fingerprint.iter().map(|e| e * e).sum::<f64>().sqrt();

    if norm_eig == 0.0 || norm_global == 0.0 {
        0.0
    } else {
        dot_product / (norm_eig * norm_global)
    }
}

/// Detect outliers using cosine similarity threshold with custom threshold
fn detect_outliers_with_threshold(entries: &mut [FingerprintEntry], threshold: f64) {
    println!("🔍 Detecting outliers (threshold: {})...", threshold);
    
    for entry in entries.iter_mut() {
        // Mark as outlier if similarity is below threshold or proof is invalid
        entry.is_outlier = entry.cosine_similarity < threshold || !entry.proof_valid;
    }
    
    let outlier_count = entries.iter().filter(|e| e.is_outlier).count();
    println!("   Found {} outliers", outlier_count);
}

/// Build experiment results summary
fn build_experiment_results(entries: &[FingerprintEntry]) -> ExperimentResults {
    let total_agents = entries.len();
    let valid_agents = entries
        .iter()
        .filter(|e| e.proof_valid && !e.is_outlier)
        .count();
    let outliers = entries.iter().filter(|e| e.is_outlier).count();
    let pass_rate = valid_agents as f64 / total_agents as f64;

    let average_consensus_similarity = if total_agents > 0 {
        entries.iter().map(|e| e.cosine_similarity).sum::<f64>() / total_agents as f64
    } else {
        0.0
    };

    let average_spectral_entropy = if total_agents > 0 {
        entries.iter().map(|e| e.spectral_entropy).sum::<f64>() / total_agents as f64
    } else {
        0.0
    };

    ExperimentResults {
        total_agents,
        valid_agents,
        outliers,
        pass_rate,
        fingerprint_table: entries.to_vec(),
        average_consensus_similarity,
        average_spectral_entropy,
    }
}

/// Print fingerprint creation table
fn print_fingerprint_table(entries: &[FingerprintEntry]) {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    Fingerprint Creation Table                                 ║");
    println!("╠════════════╦═══════════╦════════════════╦══════════════╦════════════════╦══════╦═══════╗");
    println!("║  Agent ID  ║ Prompt    ║ Δy (3 dims)    ║ Eigenvalues  ║ R (Radius)  ║ H(Ent)║ Status║");
    println!("╠════════════╬═══════════╬════════════════╬══════════════╬════════════════╬══════╬═══════╣");

    for entry in entries {
        let delta_str = format!(
            "[{:.1}, {:.1}, {:.1}]",
            entry.delta_response[0], entry.delta_response[1], entry.delta_response[2]
        );

        let eig_str = format!(
            "[{:.2}, {:.2}, {:.2}]",
            entry.eigenvalues[0], entry.eigenvalues[1], entry.eigenvalues[2]
        );

        let status = if entry.proof_valid && !entry.is_outlier {
            "✅ Valid"
        } else if !entry.proof_valid {
            "❌ Invalid"
        } else {
            "⚠️  Outlier"
        };

        println!(
            "║ {:^10} ║ {:^9} ║ {:^14} ║ {:^12} ║ {:^12} ║ {:^4} ║ {:^5} ║",
            entry.agent_id,
            entry.prompt_type,
            delta_str,
            eig_str,
            format!("{:.2}", entry.spectral_radius),
            format!("{:.2}", entry.spectral_entropy),
            status
        );
    }

    println!("╚════════════╩═══════════╩════════════════╩══════════════╩════════════════╩══════╩═══════╝");
    println!();
}

/// Print experiment summary
fn print_experiment_summary(results: &ExperimentResults) {
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    Experiment Results Summary                                 ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Total Agents:        {:^60} ║", results.total_agents);
    println!("║  Valid Agents:        {:^60} ║", results.valid_agents);
    println!("║  Outliers:           {:^60} ║", results.outliers);
    println!("║  Pass Rate:          {:^60} ║", format!("{:.1}%", results.pass_rate * 100.0));
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    println!("║  Avg Consensus Sim:  {:^60} ║", format!("{:.3}", results.average_consensus_similarity));
    println!("║  Avg Spectral Ent:   {:^60} ║", format!("{:.3}", results.average_spectral_entropy));
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // Interpret results
    println!("📊 Analysis:");
    if results.pass_rate >= 0.85 {
        println!("   ✅ System very healthy - high pass rate");
    } else if results.pass_rate >= 0.70 {
        println!("   ⚠️  System healthy - moderate pass rate");
    } else if results.pass_rate >= 0.60 {
        println!("   ⚠️  System needs attention - low pass rate");
    } else {
        println!("   ❌ System unhealthy - very low pass rate");
    }

    if results.average_spectral_entropy >= 0.6 && results.average_spectral_entropy <= 0.9 {
        println!("   ✅ Good model diversity - healthy entropy");
    } else if results.average_spectral_entropy < 0.6 {
        println!("   ⚠️  Potential homogeneity - entropy too low");
    } else {
        println!("   ⚠️  Unusual entropy - too high");
    }

    if results.average_consensus_similarity >= 0.85 {
        println!("   ✅ Strong consensus");
    } else if results.average_consensus_similarity >= 0.70 {
        println!("   ⚠️  Moderate consensus");
    } else {
        println!("   ❌ Weak consensus");
    }
}

/// Print statistical summary for multiple runs
fn print_statistical_summary(
    pass_rates: &[f64],
    similarities: &[f64],
    entropies: &[f64],
    _all_results: &[ExperimentResults],
) {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    Statistical Analysis Summary                              ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");

    let avg_pass_rate = pass_rates.iter().sum::<f64>() / pass_rates.len() as f64;
    let min_pass_rate = pass_rates.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_pass_rate = pass_rates.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let std_pass_rate = (pass_rates.iter()
        .map(|x| (x - avg_pass_rate).powi(2))
        .sum::<f64>() / pass_rates.len() as f64)
        .sqrt();

    println!("║  Pass Rate - Avg: {:.1}%, Min: {:.1}%, Max: {:.1}%, Std: {:.1}%            ║",
             avg_pass_rate * 100.0, min_pass_rate * 100.0, max_pass_rate * 100.0, std_pass_rate * 100.0);

    let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
    let min_similarity = similarities.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_similarity = similarities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("║  Consensus Sim - Avg: {:.3}, Min: {:.3}, Max: {:.3}                         ║",
             avg_similarity, min_similarity, max_similarity);

    let avg_entropy = entropies.iter().sum::<f64>() / entropies.len() as f64;
    let min_entropy = entropies.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_entropy = entropies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    println!("║  Spectral Entropy - Avg: {:.3}, Min: {:.3}, Max: {:.3}                      ║",
             avg_entropy, min_entropy, max_entropy);

    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();

    println!("📈 Overall Assessment:");
    if avg_pass_rate >= 0.85 && std_pass_rate < 0.1 {
        println!("   ✅ Excellent: High average pass rate with low variance");
    } else if avg_pass_rate >= 0.70 {
        println!("   ⚠️  Good: Acceptable pass rate");
    } else {
        println!("   ❌ Poor: Low pass rate, needs improvement");
    }

    if avg_similarity >= 0.85 {
        println!("   ✅ Strong consensus across all runs");
    } else if avg_similarity >= 0.70 {
        println!("   ⚠️  Moderate consensus, some variability");
    } else {
        println!("   ❌ Weak consensus, high variability");
    }
}

/// Calculate spectral radius (max absolute eigenvalue)
fn calculate_spectral_radius(eigenvalues: &[f64]) -> f64 {
    eigenvalues.iter().map(|e| e.abs()).fold(0.0, f64::max)
}

/// Calculate spectral entropy from eigenvalues
fn calculate_spectral_entropy(eigenvalues: &[f64]) -> f64 {
    if eigenvalues.is_empty() {
        return 0.0;
    }
    
    let sum: f64 = eigenvalues.iter().map(|e| e.abs()).sum();
    if sum == 0.0 {
        return 0.0;
    }
    
    let mut entropy = 0.0;
    for &value in eigenvalues {
        let p = value.abs() / sum;
        if p > 0.0 {
            entropy -= p * p.ln();
        }
    }
    
    // Normalize to 0-1 range
    let max_entropy = (eigenvalues.len() as f64).ln();
    if max_entropy > 0.0 {
        entropy / max_entropy
    } else {
        0.0
    }
}

/// Build causal graph using statistical method (fallback for AI)
fn build_causal_graph_statistical(
    causal_builder: &CausalGraphBuilder,
    response_history: &[Vec<f64>],
    intervention_vector: &[f64],
    args: &CliArgs,
    causal_graphs: &mut Vec<multi_agent_oracle::causal_graph::CausalGraph>,
) -> Option<multi_agent_oracle::causal_graph::CausalGraph> {
    match causal_builder.build_from_history(response_history, intervention_vector) {
        Ok(graph) => {
            if graph.is_valid() {
                println!("   ✓ Statistical causal graph built: {} nodes, {} paths",
                         graph.nodes.len(), graph.main_paths.len());
                println!("   ✓ Causal graph hash: {:?}", &graph.compute_hash()[..8]);

                // Print causal graph visualization
                if args.visualize_graphs {
                    print_causal_graph(&graph);
                    print_graph_statistics(&graph);
                }

                // Store causal graph for later analysis
                causal_graphs.push(graph.clone());

                Some(graph)
            } else {
                println!("   ⚠️  Statistical causal graph built but invalid, skipping");
                None
            }
        }
        Err(e) => {
            println!("   ⚠️  Failed to build statistical causal graph: {}", e);
            None
        }
    }
}

