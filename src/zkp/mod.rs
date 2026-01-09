//! Zero-Knowledge Proof Module for Causal Fingerprint Verification
//!
//! This module provides ZKP generation and verification capabilities
//! using Nori circuits for the MultiAgentOracle system.
//!
//! # Architecture
//!
//! ```
//! ┌─────────────────────────────────────────────────────────┐
//! │ Agent Side (Proof Generation)                          │
//! │ ┌─────────────────────────────────────────────────┐    │
//! │ │ Spectral Analysis → Circuit Inputs → ZKP        │    │
//! │ └─────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────┘
//!                         ↓ ZKP
//! ┌─────────────────────────────────────────────────────────┐
//! │ Solana Side (Proof Verification)                       │
//! │ ┌─────────────────────────────────────────────────┐    │
//! │ │ ZKP + Public Inputs → Verification → Result     │    │
//! │ └─────────────────────────────────────────────────┘    │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod types;
pub mod nori_adapter;

pub use types::*;
pub use nori_adapter::NoriAdapter;

/// Result type for ZKP operations
pub type Result<T> = std::result::Result<T, ZkpError>;

/// Error types for ZKP operations
#[derive(Debug, thiserror::Error)]
pub enum ZkpError {
    #[error("Circuit compilation failed: {0}")]
    CircuitCompilationFailed(String),

    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),

    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),

    #[error("Invalid circuit inputs: {0}")]
    InvalidInputs(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Key loading failed: {0}")]
    KeyLoadingError(String),
}

/// ZKP Generator configuration
#[derive(Debug, Clone)]
pub struct ZkpConfig {
    /// Path to Nori circuit files
    pub circuit_path: String,

    /// Path to proving key
    pub proving_key_path: String,

    /// Path to verification key
    pub verification_key_path: String,

    /// Number of eigenvalues to include in proof
    pub num_eigenvalues: usize,

    /// Fixed-point scale factor for arithmetic
    pub scale_factor: u64,

    /// Enable debug logging
    pub debug: bool,
}

impl Default for ZkpConfig {
    fn default() -> Self {
        Self {
            circuit_path: "circuits/causal_fingerprint.circom".to_string(),
            proving_key_path: "circuits/keys/causal_fingerprint.zkey".to_string(),
            verification_key_path: "circuits/keys/verification_key.json".to_string(),
            num_eigenvalues: 3,
            scale_factor: 1_000_000,
            debug: false,
        }
    }
}

/// Zero-Knowledge Proof Generator
pub struct ZkpGenerator {
    config: ZkpConfig,
    nori_adapter: NoriAdapter,
}

impl ZkpGenerator {
    /// Create a new ZKP generator with default configuration
    pub fn new() -> Result<Self> {
        let config = ZkpConfig::default();
        Self::with_config(config)
    }

    /// Create a new ZKP generator with custom configuration
    pub fn with_config(config: ZkpConfig) -> Result<Self> {
        let nori_adapter = NoriAdapter::new(&config)?;
        Ok(Self { config, nori_adapter })
    }

    /// Generate a ZK proof for causal fingerprint verification
    ///
    /// # Arguments
    ///
    /// * `spectral_features` - Spectral features from analysis
    /// * `response_history` - Historical response matrix
    /// * `intervention_vector` - Random intervention vector
    /// * `delta_response` - Agent's causal response
    ///
    /// # Returns
    ///
    /// Returns a ZK proof that can be verified on-chain
    pub async fn generate_fingerprint_proof(
        &self,
        spectral_features: &crate::consensus::SpectralFeatures,
        response_history: &[Vec<f64>],
        intervention_vector: &[f64],
        delta_response: &[f64],
    ) -> Result<ZkProof> {
        // Validate inputs
        if spectral_features.eigenvalues.len() < self.config.num_eigenvalues {
            return Err(ZkpError::InvalidInputs(format!(
                "Insufficient eigenvalues: expected {}, got {}",
                self.config.num_eigenvalues,
                spectral_features.eigenvalues.len()
            )));
        }

        // Build circuit inputs
        let circuit_inputs = self.build_circuit_inputs(
            spectral_features,
            response_history,
            intervention_vector,
            delta_response,
        )?;

        // Generate proof using Nori adapter
        if self.config.debug {
            println!("🔒 Generating ZK proof...");
            println!("   Public inputs: {}", circuit_inputs.public_inputs.len());
            println!("   Private inputs: {}", circuit_inputs.private_inputs.len());
        }

        let proof = self.nori_adapter.generate_proof(&circuit_inputs).await?;

        if self.config.debug {
            println!("   ✅ Proof generated successfully");
            println!("   Proof size: {} bytes", proof.proof_bytes.len());
        }

        Ok(proof)
    }

    /// Build circuit inputs from spectral analysis results
    fn build_circuit_inputs(
        &self,
        spectral_features: &crate::consensus::SpectralFeatures,
        response_history: &[Vec<f64>],
        intervention_vector: &[f64],
        delta_response: &[f64],
    ) -> Result<CircuitInputs> {
        // Flatten response history (10 agents × 5 dimensions = 50 values)
        let mut flat_history = Vec::with_capacity(50);
        for response in response_history.iter().take(10) {
            for val in response.iter().take(5) {
                flat_history.push(*val);
            }
        }
        // Pad if needed
        while flat_history.len() < 50 {
            flat_history.push(0.0);
        }

        // Build public inputs
        let public_inputs = PublicInputs {
            intervention_vector: intervention_vector.to_vec(),
            delta_response: delta_response.to_vec(),
            expected_eigenvalues: spectral_features.eigenvalues[..self.config.num_eigenvalues].to_vec(),
            spectral_radius: spectral_features.spectral_radius,
            spectral_entropy: spectral_features.entropy,
            cosine_similarity: 0.9, // TODO: Calculate from global fingerprint
        };

        // Build private inputs (simplified - in real implementation, compute covariance)
        let private_inputs = PrivateInputs {
            response_history: flat_history,
            covariance_matrix: vec![0.0; 25], // Placeholder - needs computation
            eigenvectors: vec![0.0; 15],     // Placeholder - needs computation
        };

        Ok(CircuitInputs {
            public_inputs,
            private_inputs,
        })
    }

    /// Verify a ZK proof (for local testing)
    pub async fn verify_proof(&self, proof: &ZkProof, public_inputs: &PublicInputs) -> Result<bool> {
        self.nori_adapter.verify_proof(proof, public_inputs).await
    }

    /// Get the verification key for on-chain deployment
    pub fn verification_key(&self) -> Result<VerificationKey> {
        Ok(self.nori_adapter.verification_key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zkp_config_default() {
        let config = ZkpConfig::default();
        assert_eq!(config.num_eigenvalues, 3);
        assert_eq!(config.scale_factor, 1_000_000);
    }

    #[test]
    fn test_build_circuit_inputs() {
        let generator = ZkpGenerator::new().unwrap();
        let spectral_features = crate::consensus::SpectralFeatures {
            eigenvalues: vec![5.0, 3.0, 1.0, 0.5, 0.2, 0.1, 0.05, 0.02],
            spectral_radius: 5.0,
            trace: 9.87,
            rank: 4,
            entropy: 0.75,
            timestamp: 0,
        };

        let response_history = vec![
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![1.1, 2.1, 3.1, 4.1, 5.1],
        ];

        let intervention_vector = vec![1.0, -1.0, 0.5, -0.5, 0.0];
        let delta_response = vec![1.2, 0.8, 1.5, -0.3, 0.9];

        let inputs = generator
            .build_circuit_inputs(
                &spectral_features,
                &response_history,
                &intervention_vector,
                &delta_response,
            )
            .unwrap();

        assert_eq!(inputs.public_inputs.intervention_vector.len(), 5);
        assert_eq!(inputs.public_inputs.delta_response.len(), 5);
        assert_eq!(inputs.public_inputs.expected_eigenvalues.len(), 3);
        assert_eq!(inputs.private_inputs.response_history.len(), 50);
    }
}
