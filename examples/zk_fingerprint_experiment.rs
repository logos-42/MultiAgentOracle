//! ZK Causal Fingerprint Experiment
//!
//! This example demonstrates the complete workflow of:
//! 1. Creating multiple oracle agents with different prompt identities
//! 2. Running causal fingerprint detection with ZK proofs
//! 3. Generating fingerprint creation table
//! 4. Calculating pass rate
//!
//! Usage:
//!   cargo run --example zk_fingerprint_experiment
//!   cargo run --example zk_fingerprint_experiment -- --config configs/test_aggressive.json
//!   cargo run --example zk_fingerprint_experiment -- --agents analytical=3 cautious=3 aggressive=2 neutral=2

use multi_agent_oracle::{
    OracleAgent, OracleAgentConfig, OracleDataType,
    consensus::{CausalFingerprint, extract_spectral_features},
    zkp::{ZkpGenerator, ZkpConfig, ZkProof, PublicInputs},
    diap::{AgentIdentity, IdentityStatus},
};
use std::collections::HashMap;
use std::time::SystemTime;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use std::env;

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
}

fn default_sensitivity() -> f64 { 1.0 }
fn default_noise_level() -> f64 { 0.1 }

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
}

fn default_intervention_dimensions() -> usize { 5 }
fn default_consensus_threshold() -> f64 { 0.85 }
fn default_global_fingerprint() -> Vec<f64> { vec![5.0, 3.0, 1.0] }

/// Command Line Arguments
struct CliArgs {
    config_path: Option<String>,
    agent_counts: Option<HashMap<String, usize>>,
    test_runs: usize,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            agents: create_default_agent_prompt_identities(),
            intervention_dimensions: 5,
            consensus_threshold: 0.85,
            global_fingerprint: vec![5.0, 3.0, 1.0],
            test_runs: 1,
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
    pub proof_valid: bool,          // ZK proof verification result
    pub is_outlier: bool,           // Outlier detection result
}

/// Main Experiment Function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        run_multiple_experiments(&config).await?;
    } else {
        // Run single experiment
        run_single_experiment(&config).await?;
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

    Ok(config)
}

/// Generate agents from count specification
fn generate_agents_from_counts(counts: &HashMap<String, usize>) -> Vec<AgentPromptIdentity> {
    let mut agents = Vec::new();
    let mut agent_id = 1;

    for (prompt_type, &count) in counts {
        for i in 0..count {
            let characteristics = get_characteristics_for_prompt(prompt_type);
            agents.push(AgentPromptIdentity {
                agent_id: format!("agent_{}", agent_id),
                prompt_type: prompt_type.clone(),
                model_characteristics: characteristics,
                sensitivity: get_default_sensitivity(prompt_type),
                noise_level: 0.1,
            });
            agent_id += 1;
        }
    }

    agents
}

/// Run a single experiment
async fn run_single_experiment(config: &ExperimentConfig) -> Result<ExperimentResults, Box<dyn std::error::Error>> {
    println!("🔄 Running single experiment with {} agents...", config.agents.len());
    
    // Generate random intervention vector (δX)
    let intervention_vector = generate_intervention_vector(config.intervention_dimensions);
    println!("✅ Generated intervention vector δX: {:?}", intervention_vector);
    println!();

    // Initialize ZKP generator
    let zkp_generator = ZkpGenerator::new()?;
    println!("✅ Initialized ZKP generator");
    println!();

    // Run experiment
    let results = run_experiment_with_config(config, &intervention_vector, &zkp_generator).await?;

    // Print results
    print_fingerprint_table(&results.fingerprint_table);
    print_experiment_summary(&results);

    println!();
    println!("✅ Experiment completed successfully!");

    Ok(results)
}

/// Run multiple experiments for statistical analysis
async fn run_multiple_experiments(config: &ExperimentConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Running {} experiments for statistical analysis...", config.test_runs);
    
    let mut all_results = Vec::new();
    let mut all_pass_rates = Vec::new();
    let mut all_similarities = Vec::new();
    let mut all_entropies = Vec::new();

    for run in 0..config.test_runs {
        println!("\n========== Run {}/{} ==========", run + 1, config.test_runs);
        
        let results = run_single_experiment(config).await?;
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
        },
        AgentPromptIdentity {
            agent_id: "agent_5".to_string(),
            prompt_type: "analytical".to_string(),
            model_characteristics: vec![
                "统计方法".to_string(),
                "量化分析".to_string(),
                "数据驱动".to_string(),
            ],
            sensitivity: 1.0,
            noise_level: 0.1,
        },
        AgentPromptIdentity {
            agent_id: "agent_6".to_string(),
            prompt_type: "cautious".to_string(),
            model_characteristics: vec![
                "风险厌恶".to_string(),
                "保守策略".to_string(),
                "安全第一".to_string(),
            ],
            sensitivity: 0.5,
            noise_level: 0.05,
        },
        AgentPromptIdentity {
            agent_id: "agent_7".to_string(),
            prompt_type: "neutral".to_string(),
            model_characteristics: vec![
                "平衡观点".to_string(),
                "多方考虑".to_string(),
                "折中方案".to_string(),
            ],
            sensitivity: 1.0,
            noise_level: 0.1,
        },
        AgentPromptIdentity {
            agent_id: "agent_8".to_string(),
            prompt_type: "aggressive".to_string(),
            model_characteristics: vec![
                "积极进取".to_string(),
                "高回报导向".to_string(),
                "风险承担".to_string(),
            ],
            sensitivity: 1.5,
            noise_level: 0.15,
        },
        AgentPromptIdentity {
            agent_id: "agent_9".to_string(),
            prompt_type: "analytical".to_string(),
            model_characteristics: vec![
                "理性分析".to_string(),
                "逻辑严密".to_string(),
                "证据驱动".to_string(),
            ],
            sensitivity: 1.0,
            noise_level: 0.1,
        },
        AgentPromptIdentity {
            agent_id: "agent_10".to_string(),
            prompt_type: "suspicious".to_string(),
            model_characteristics: vec![
                "异常行为".to_string(),
                "逻辑不一致".to_string(),
                "可能的攻击者".to_string(),
            ],
            sensitivity: -1.0,
            noise_level: 0.2,
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
) -> Result<ExperimentResults, Box<dyn std::error::Error>> {
    let mut fingerprint_entries = Vec::new();
    let mut response_history = Vec::new();

    for identity in &config.agents {
        println!("🔄 Processing agent {} ({})...", identity.agent_id, identity.prompt_type);

        // Compute causal response (Δy)
        let delta_response = compute_causal_response(identity, intervention_vector);
        println!("   ✓ Causal response Δy: {:?}", delta_response);

        // Add to response history
        response_history.push(delta_response.clone());

        // Extract spectral features
        let spectral_features = extract_spectral_features(&response_history);
        println!(
            "   ✓ Eigenvalues: {:?}",
            &spectral_features.eigenvalues[..3.min(spectral_features.eigenvalues.len())]
        );
        println!(
            "   ✓ Spectral radius: {:.4}, Entropy: {:.4}",
            spectral_features.spectral_radius, spectral_features.entropy
        );

        // Generate ZK proof
        let proof = zkp_generator
            .generate_fingerprint_proof(
                &spectral_features,
                &response_history,
                intervention_vector,
                &delta_response,
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
            &spectral_features.eigenvalues,
            &config.global_fingerprint
        );

        fingerprint_entries.push(FingerprintEntry {
            agent_id: identity.agent_id.clone(),
            prompt_type: identity.prompt_type.clone(),
            delta_response,
            eigenvalues: spectral_features.eigenvalues.clone(),
            spectral_radius: spectral_features.spectral_radius,
            spectral_entropy: spectral_features.entropy,
            cosine_similarity,
            proof_valid,
            is_outlier: false,
        });

        println!();
    }

    // Detect outliers
    detect_outliers_with_threshold(&mut fingerprint_entries, config.consensus_threshold);

    // Build results
    Ok(build_experiment_results(&fingerprint_entries))
}

/// Create agent prompt identities (legacy function for backward compatibility)
fn create_agent_prompt_identities() -> Vec<AgentPromptIdentity> {
    create_default_agent_prompt_identities()
}

/// Generate random intervention vector (δX) with specified dimensions
fn generate_intervention_vector(dimensions: usize) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Generate n-dimensional intervention vector
    (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect()
}

/// Compute causal response (Δy) based on prompt identity
fn compute_causal_response(identity: &AgentPromptIdentity, intervention_vector: &[f64]) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Compute Δy = sensitivity * δX + noise
    intervention_vector
        .iter()
        .map(|x| identity.sensitivity * x + rng.gen_range(-identity.noise_level..identity.noise_level))
        .collect()
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

/// Calculate consensus similarity (simplified cosine similarity) - legacy function
fn calculate_consensus_similarity(eigenvalues: &[f64]) -> f64 {
    let global_fingerprint = vec![5.0, 3.0, 1.0];
    calculate_consensus_similarity_with_global(eigenvalues, &global_fingerprint)
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

/// Detect outliers using cosine similarity threshold - legacy function
fn detect_outliers(entries: &mut [FingerprintEntry]) {
    detect_outliers_with_threshold(entries, 0.85);
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
    println!("║                    Fingerpring Creation Table                                    ║");
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
            "❌ Invalid Proof"
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
    println!("║                    Experiment Results Summary                                    ║");
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
        println!("   ✅ System is very healthy - high pass rate (>85%)");
    } else if results.pass_rate >= 0.70 {
        println!("   ⚠️  System is healthy - moderate pass rate (70-85%)");
    } else if results.pass_rate >= 0.60 {
        println!("   ⚠️  System needs attention - low pass rate (60-70%)");
    } else {
        println!("   ❌ System is unhealthy - very low pass rate (<60%)");
    }

    if results.average_spectral_entropy >= 0.6 && results.average_spectral_entropy <= 0.9 {
        println!("   ✅ Good model diversity - entropy in healthy range");
    } else if results.average_spectral_entropy < 0.6 {
        println!("   ⚠️  Potential homogeneity - entropy too low (<0.6)");
    } else {
        println!("   ⚠️  Unusual entropy - too high (>0.9)");
    }

    if results.average_consensus_similarity >= 0.85 {
        println!("   ✅ Strong consensus - high similarity");
    } else if results.average_consensus_similarity >= 0.70 {
        println!("   ⚠️  Moderate consensus - acceptable similarity");
    } else {
        println!("   ❌ Weak consensus - low similarity");
    }
}

/// Print statistical summary for multiple runs
fn print_statistical_summary(
    pass_rates: &[f64],
    similarities: &[f64],
    entropies: &[f64],
    all_results: &[ExperimentResults],
) {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                    Statistical Analysis Summary                              ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    
    // Pass rate statistics
    let avg_pass_rate = pass_rates.iter().sum::<f64>() / pass_rates.len() as f64;
    let min_pass_rate = pass_rates.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_pass_rate = pass_rates.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let std_pass_rate = (pass_rates.iter()
        .map(|x| (x - avg_pass_rate).powi(2))
        .sum::<f64>() / pass_rates.len() as f64)
        .sqrt();
    
    println!("║  Pass Rate - Avg: {:.1}%, Min: {:.1}%, Max: {:.1}%, Std: {:.1}%            ║", 
             avg_pass_rate * 100.0, min_pass_rate * 100.0, max_pass_rate * 100.0, std_pass_rate * 100.0);
    
    // Consensus similarity statistics
    let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
    let min_similarity = similarities.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_similarity = similarities.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    println!("║  Consensus Sim - Avg: {:.3}, Min: {:.3}, Max: {:.3}                         ║", 
             avg_similarity, min_similarity, max_similarity);
    
    // Entropy statistics
    let avg_entropy = entropies.iter().sum::<f64>() / entropies.len() as f64;
    let min_entropy = entropies.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_entropy = entropies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    println!("║  Spectral Entropy - Avg: {:.3}, Min: {:.3}, Max: {:.3}                      ║", 
             avg_entropy, min_entropy, max_entropy);
    
    println!("╚════════════════════════════════════════════════════════════════════════════╝");
    println!();
    
    // Overall assessment
    println!("📈 Overall Assessment:");
    if avg_pass_rate >= 0.85 && std_pass_rate < 0.1 {
        println!("   ✅ Excellent: High average pass rate with low variance");
    } else if avg_pass_rate >= 0.70 {
        println!("   ⚠️  Good: Acceptable pass rate, watch for consistency");
    } else {
        println!("   ❌ Poor: Low pass rate, system needs improvement");
    }
    
    if avg_similarity >= 0.85 {
        println!("   ✅ Strong consensus across all runs");
    } else if avg_similarity >= 0.70 {
        println!("   ⚠️  Moderate consensus, some variability");
    } else {
        println!("   ❌ Weak consensus, high system variability");
    }
    
    // Prompt type analysis
    let mut prompt_stats: HashMap<String, (usize, f64)> = HashMap::new();
    for results in all_results {
        for entry in &results.fingerprint_table {
            let stats = prompt_stats.entry(entry.prompt_type.clone()).or_insert((0, 0.0));
            stats.0 += 1;
            stats.1 += entry.cosine_similarity;
        }
    }
    
    println!("\n📊 Prompt Type Analysis:");
    for (prompt_type, (count, total_sim)) in prompt_stats {
        let avg_sim = total_sim / count as f64;
        let performance = if avg_sim >= 0.85 { "✅" } else if avg_sim >= 0.70 { "⚠️" } else { "❌" };
        println!("   {} {}: avg similarity {:.3} ({} samples)", performance, prompt_type, avg_sim, count);
    }
}
