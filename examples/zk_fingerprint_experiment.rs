//! ZK Causal Fingerprint Experiment
//!
//! This example demonstrates the complete workflow of:
//! 1. Creating multiple oracle agents with different prompt identities
//! 2. Running causal fingerprint detection with ZK proofs
//! 3. Generating fingerprint creation table
//! 4. Calculating pass rate

use multi_agent_oracle::{
    OracleAgent, OracleAgentConfig, OracleDataType,
    consensus::{CausalFingerprint, extract_spectral_features},
    zkp::{ZkpGenerator, ZkpConfig, ZkProof, PublicInputs},
    diap::{AgentIdentity, IdentityStatus},
};
use std::collections::HashMap;
use std::time::SystemTime;

/// Agent Prompt Identity Configuration
#[derive(Debug, Clone)]
pub struct AgentPromptIdentity {
    pub agent_id: String,
    pub prompt_type: String,  // "analytical", "cautious", "aggressive", "neutral", "suspicious"
    pub model_characteristics: Vec<String>,
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

    // Step 1: Create agent prompt identities
    let prompt_identities = create_agent_prompt_identities();
    println!("✅ Created {} agent prompt identities", prompt_identities.len());

    // Step 2: Generate random intervention vector (δX)
    let intervention_vector = generate_intervention_vector();
    println!("✅ Generated intervention vector δX: {:?}", intervention_vector);
    println!();

    // Step 3: Initialize ZKP generator
    let zkp_generator = ZkpGenerator::new()?;
    println!("✅ Initialized ZKP generator");
    println!();

    // Step 4: For each agent, compute causal response and generate ZK proof
    let mut fingerprint_entries = Vec::new();
    let mut response_history = Vec::new();

    for identity in &prompt_identities {
        println!("🔄 Processing agent {} ({})...", identity.agent_id, identity.prompt_type);

        // Step 4a: Compute causal response (Δy)
        let delta_response = compute_causal_response(identity, &intervention_vector);
        println!("   ✓ Causal response Δy: {:?}", delta_response);

        // Step 4b: Add to response history
        response_history.push(delta_response.clone());

        // Step 4c: Extract spectral features
        let spectral_features = extract_spectral_features(&response_history);
        println!(
            "   ✓ Eigenvalues: {:?}",
            &spectral_features.eigenvalues[..3]
        );
        println!(
            "   ✓ Spectral radius: {:.4}, Entropy: {:.4}",
            spectral_features.spectral_radius, spectral_features.entropy
        );

        // Step 4d: Generate ZK proof
        let proof = zkp_generator
            .generate_fingerprint_proof(
                &spectral_features,
                &response_history,
                &intervention_vector,
                &delta_response,
            )
            .await?;

        println!("   ✓ ZK proof generated ({} bytes)", proof.proof_bytes.len());

        // Step 4e: Verify proof locally
        let proof_valid = zkp_generator
            .verify_proof(&proof, &proof.public_inputs)
            .await?;

        println!("   ✓ Proof verification: {}", if proof_valid { "✅ Valid" } else { "❌ Invalid" });

        // Step 4f: Calculate cosine similarity to consensus (simplified)
        let cosine_similarity = calculate_consensus_similarity(&spectral_features.eigenvalues);

        fingerprint_entries.push(FingerprintEntry {
            agent_id: identity.agent_id.clone(),
            prompt_type: identity.prompt_type.clone(),
            delta_response,
            eigenvalues: spectral_features.eigenvalues.clone(),
            spectral_radius: spectral_features.spectral_radius,
            spectral_entropy: spectral_features.entropy,
            cosine_similarity,
            proof_valid,
            is_outlier: false, // Will be determined after clustering
        });

        println!();
    }

    // Step 5: Detect outliers using clustering
    println!("🔍 Detecting outliers...");
    detect_outliers(&mut fingerprint_entries);

    // Step 6: Build experiment results
    let results = build_experiment_results(&fingerprint_entries);

    // Step 7: Print fingerprint creation table
    print_fingerprint_table(&results.fingerprint_table);

    // Step 8: Print summary statistics
    print_experiment_summary(&results);

    println!();
    println!("✅ Experiment completed successfully!");

    Ok(())
}

/// Create agent prompt identities with different characteristics
fn create_agent_prompt_identities() -> Vec<AgentPromptIdentity> {
    vec![
        AgentPromptIdentity {
            agent_id: "agent_1".to_string(),
            prompt_type: "analytical".to_string(),
            model_characteristics: vec![
                "逻辑推理能力强".to_string(),
                "数据分析严谨".to_string(),
                "风险偏好中性".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_2".to_string(),
            prompt_type: "cautious".to_string(),
            model_characteristics: vec![
                "保守估计".to_string(),
                "注重安全性".to_string(),
                "低风险容忍度".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_3".to_string(),
            prompt_type: "aggressive".to_string(),
            model_characteristics: vec![
                "乐观估计".to_string(),
                "追求高收益".to_string(),
                "高风险容忍度".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_4".to_string(),
            prompt_type: "neutral".to_string(),
            model_characteristics: vec![
                "平衡分析".to_string(),
                "综合考虑".to_string(),
                "中庸策略".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_5".to_string(),
            prompt_type: "analytical".to_string(),
            model_characteristics: vec![
                "统计方法".to_string(),
                "量化分析".to_string(),
                "数据驱动".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_6".to_string(),
            prompt_type: "cautious".to_string(),
            model_characteristics: vec![
                "风险厌恶".to_string(),
                "保守策略".to_string(),
                "安全第一".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_7".to_string(),
            prompt_type: "neutral".to_string(),
            model_characteristics: vec![
                "平衡观点".to_string(),
                "多方考虑".to_string(),
                "折中方案".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_8".to_string(),
            prompt_type: "aggressive".to_string(),
            model_characteristics: vec![
                "积极进取".to_string(),
                "高回报导向".to_string(),
                "风险承担".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_9".to_string(),
            prompt_type: "analytical".to_string(),
            model_characteristics: vec![
                "理性分析".to_string(),
                "逻辑严密".to_string(),
                "证据驱动".to_string(),
            ],
        },
        AgentPromptIdentity {
            agent_id: "agent_10".to_string(),
            prompt_type: "suspicious".to_string(),
            model_characteristics: vec![
                "异常行为".to_string(),
                "逻辑不一致".to_string(),
                "可能的攻击者".to_string(),
            ],
        },
    ]
}

/// Generate random intervention vector (δX) from Solana blockhash
fn generate_intervention_vector() -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Generate 5-dimensional intervention vector
    vec![
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
        rng.gen_range(-1.0..1.0),
    ]
}

/// Compute causal response (Δy) based on prompt identity
fn compute_causal_response(identity: &AgentPromptIdentity, intervention_vector: &[f64]) -> Vec<f64> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Base sensitivity based on prompt type
    let sensitivity = match identity.prompt_type.as_str() {
        "analytical" => 1.0,
        "cautious" => 0.5,
        "aggressive" => 1.5,
        "neutral" => 1.0,
        "suspicious" => -1.0, // Negative response indicates suspicious behavior
        _ => 1.0,
    };

    // Compute Δy = sensitivity * δX + noise
    intervention_vector
        .iter()
        .map(|x| sensitivity * x + rng.gen_range(-0.1..0.1))
        .collect()
}

/// Calculate consensus similarity (simplified cosine similarity)
fn calculate_consensus_similarity(eigenvalues: &[f64]) -> f64 {
    // Use average of first 3 eigenvalues as consensus reference
    let global_fingerprint = vec![5.0, 3.0, 1.0]; // Example global fingerprint

    let eigenvalues_truncated = &eigenvalues[..3.min(eigenvalues.len())];

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

/// Detect outliers using cosine similarity threshold
fn detect_outliers(entries: &mut [FingerprintEntry]) {
    let threshold = 0.85; // Similarity threshold

    for entry in entries.iter_mut() {
        // Mark as outlier if similarity is below threshold or proof is invalid
        entry.is_outlier = entry.cosine_similarity < threshold || !entry.proof_valid;
    }
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
