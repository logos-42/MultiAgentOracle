//! Type definitions for ZKP circuit inputs and outputs

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Zero-Knowledge Proof for causal fingerprint verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    /// Serialized proof bytes (Groth16 format)
    pub proof_bytes: Vec<u8>,

    /// Public inputs used for proof generation
    pub public_inputs: PublicInputs,

    /// Circuit hash (for verification)
    pub circuit_hash: [u8; 32],

    /// Proof generation timestamp
    pub timestamp: u64,

    /// Proof metadata
    pub metadata: ProofMetadata,
}

/// Proof metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// Agent ID that generated proof
    pub agent_id: String,

    /// Proof generation time in milliseconds
    pub generation_time_ms: u64,

    /// Memory usage in bytes
    pub memory_usage_bytes: u64,

    /// Number of circuit constraints
    pub num_constraints: u64,
}

/// Circuit inputs for ZK proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitInputs {
    /// Public inputs (will be verified on-chain)
    pub public_inputs: PublicInputs,

    /// Private inputs (known only to the agent)
    pub private_inputs: PrivateInputs,
}

/// Public inputs (visible to all and verified on-chain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    /// Random intervention vector δX (from Solana blockhash)
    pub intervention_vector: Vec<f64>,

    /// Agent's causal response Δy
    pub delta_response: Vec<f64>,

    /// Claimed eigenvalues λ = [λ₁, λ₂, λ₃]
    pub expected_eigenvalues: Vec<f64>,

    /// Spectral radius R = max(|λ[i]|)
    pub spectral_radius: f64,

    /// Spectral entropy H = -Σ(p[i] * log2(p[i]))
    pub spectral_entropy: f64,

    /// Cosine similarity to global fingerprint
    pub cosine_similarity: f64,
}

impl PublicInputs {
    /// Calculate total number of input values
    pub fn len(&self) -> usize {
        self.intervention_vector.len()
            + self.delta_response.len()
            + self.expected_eigenvalues.len()
            + 3 // spectral_radius, spectral_entropy, cosine_similarity
    }

    /// Convert public inputs to i64 array for on-chain storage
    pub fn to_i64_array(&self) -> [i64; 16] {
        let scale = 1_000_000.0;
        let mut result = [0i64; 16];

        // intervention_vector (5 values)
        for (i, val) in self.intervention_vector.iter().take(5).enumerate() {
            result[i] = (*val * scale) as i64;
        }

        // delta_response (5 values)
        for (i, val) in self.delta_response.iter().take(5).enumerate() {
            result[i + 5] = (*val * scale) as i64;
        }

        // expected_eigenvalues (3 values) - store first 3
        for (i, val) in self.expected_eigenvalues.iter().take(3).enumerate() {
            result[i + 10] = (*val * scale) as i64;
        }

        // spectral_radius (1 value)
        result[13] = (self.spectral_radius * scale) as i64;

        // spectral_entropy (scaled to 0-100)
        result[14] = ((self.spectral_entropy * 100.0) as i64).clamp(0, 100);

        // cosine_similarity (scaled to 0-100)
        result[15] = ((self.cosine_similarity * 100.0) as i64).clamp(0, 100);

        result
    }

    /// Create public inputs from i64 array
    pub fn from_i64_array(data: &[i64; 16]) -> Self {
        let scale = 1_000_000.0;

        let intervention_vector = data[0..5].iter().map(|v| *v as f64 / scale).collect();
        let delta_response = data[5..10].iter().map(|v| *v as f64 / scale).collect();
        let expected_eigenvalues = data[10..13].iter().map(|v| *v as f64 / scale).collect();
        let spectral_radius = data[13] as f64 / scale;
        let spectral_entropy = data[14] as f64 / 100.0;
        let cosine_similarity = data[15] as f64 / 100.0;

        Self {
            intervention_vector,
            delta_response,
            expected_eigenvalues,
            spectral_radius,
            spectral_entropy,
            cosine_similarity,
        }
    }

    /// Hash of public inputs (for verification)
    pub fn hash(&self) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();

        // Hash each field manually
        for val in &self.intervention_vector {
            val.to_bits().hash(&mut hasher);
        }
        for val in &self.delta_response {
            val.to_bits().hash(&mut hasher);
        }
        for val in &self.expected_eigenvalues {
            val.to_bits().hash(&mut hasher);
        }
        self.spectral_radius.to_bits().hash(&mut hasher);
        self.spectral_entropy.to_bits().hash(&mut hasher);
        self.cosine_similarity.to_bits().hash(&mut hasher);

        let hash = hasher.finish();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash.to_be_bytes()[..32]);
        bytes
    }
}

/// Private inputs (known only to the agent, used for proof generation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInputs {
    /// Historical response matrix (10 agents × 5 dimensions, flattened)
    pub response_history: Vec<f64>,

    /// Covariance matrix (5x5, flattened)
    pub covariance_matrix: Vec<f64>,

    /// Eigenvectors (3 principal components, flattened 3x5)
    pub eigenvectors: Vec<f64>,
}

impl PrivateInputs {
    /// Calculate total number of input values
    pub fn len(&self) -> usize {
        self.response_history.len() + self.covariance_matrix.len() + self.eigenvectors.len()
    }
}

/// Verification key for on-chain proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationKey {
    /// Verification key bytes
    pub key_bytes: Vec<u8>,

    /// Circuit identifier
    pub circuit_id: String,

    /// Version
    pub version: String,

    /// Key hash
    pub key_hash: [u8; 32],
}

/// Proving key for proof generation (kept private)
#[derive(Debug, Clone)]
pub struct ProvingKey {
    /// Proving key bytes
    pub key_bytes: Vec<u8>,

    /// Circuit identifier
    pub circuit_id: String,

    /// Key hash
    pub key_hash: [u8; 32],
}

/// Circuit metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitMetadata {
    /// Circuit name
    pub name: String,

    /// Circuit version
    pub version: String,

    /// Number of constraints
    pub num_constraints: u64,

    /// Number of public inputs
    pub num_public_inputs: u64,

    /// Number of private inputs
    pub num_private_inputs: u64,

    /// Field size in bits
    pub field_size_bits: u64,

    /// Security level in bits
    pub security_level_bits: u64,
}

impl Default for CircuitMetadata {
    fn default() -> Self {
        Self {
            name: "causal_fingerprint".to_string(),
            version: "0.1.0".to_string(),
            num_constraints: 10_000,
            num_public_inputs: 15,
            num_private_inputs: 90,
            field_size_bits: 254,
            security_level_bits: 128,
        }
    }
}

/// Fingerprint proof submission to Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintSubmission {
    /// Agent ID
    pub agent_id: String,

    /// Task ID
    pub task_id: String,

    /// ZK proof
    pub proof: ZkProof,

    /// Fingerprint data (derived from public inputs)
    pub fingerprint: FingerprintData,

    /// Agent signature
    pub signature: Vec<u8>,
}

/// Fingerprint data extracted from proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintData {
    /// Eigenvalues (i64 format for on-chain)
    pub eigenvalues_i64: [i64; 8],

    /// Spectral radius (i64 format)
    pub spectral_radius_i64: i64,

    /// Spectral entropy (0-100)
    pub spectral_entropy_i64: i64,

    /// Cosine similarity to global (0-100)
    pub cosine_similarity_i64: i64,

    /// Effective rank
    pub rank: usize,

    /// Timestamp
    pub timestamp: u64,
}

/// Proof verification result from Solana contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether to proof is valid
    pub is_valid: bool,

    /// Agent reputation score (updated after verification)
    pub reputation_score: u64,

    /// Reward amount (in smallest unit)
    pub reward_amount: u64,

    /// Error message (if any)
    pub error_message: Option<String>,

    /// Verification timestamp
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_inputs_conversion() {
        let public_inputs = PublicInputs {
            intervention_vector: vec![1.0, -1.0, 0.5, -0.5, 0.0],
            delta_response: vec![1.2, 0.8, 1.5, -0.3, 0.9],
            expected_eigenvalues: vec![5.0, 3.0, 1.0],
            spectral_radius: 5.0,
            spectral_entropy: 0.75,
            cosine_similarity: 0.9,
        };

        let i64_array = public_inputs.to_i64_array();
        let recovered = PublicInputs::from_i64_array(&i64_array);

        // Check values are approximately preserved
        for (orig, rec) in public_inputs
            .intervention_vector
            .iter()
            .zip(recovered.intervention_vector.iter())
        {
            assert!((orig - rec).abs() < 0.001);
        }

        for (orig, rec) in public_inputs
            .delta_response
            .iter()
            .zip(recovered.delta_response.iter())
        {
            assert!((orig - rec).abs() < 0.001);
        }

        assert!((public_inputs.spectral_radius - recovered.spectral_radius).abs() < 0.001);
        assert!((public_inputs.spectral_entropy - recovered.spectral_entropy).abs() < 0.01);
        assert!((public_inputs.cosine_similarity - recovered.cosine_similarity).abs() < 0.01);
    }

    #[test]
    fn test_public_inputs_hash() {
        let public_inputs = PublicInputs {
            intervention_vector: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            delta_response: vec![0.5, 1.0, 1.5, 2.0, 2.5],
            expected_eigenvalues: vec![1.0, 0.5, 0.25],
            spectral_radius: 1.0,
            spectral_entropy: 0.8,
            cosine_similarity: 0.85,
        };

        let hash1 = public_inputs.hash();
        let hash2 = public_inputs.hash();

        assert_eq!(hash1, hash2, "Hash should be deterministic");

        // Modify and verify hash changes
        let mut modified = public_inputs.clone();
        modified.intervention_vector[0] = 2.0;
        let hash3 = modified.hash();

        assert_ne!(hash1, hash3, "Hash should change with input modification");
    }

    #[test]
    fn test_circuit_metadata_default() {
        let metadata = CircuitMetadata::default();
        assert_eq!(metadata.name, "causal_fingerprint");
        assert_eq!(metadata.num_constraints, 10_000);
        assert_eq!(metadata.security_level_bits, 128);
    }
}
