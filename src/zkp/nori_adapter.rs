//! Nori Circuit Adapter
//!
//! This module provides interface between Rust codebase and Nori ZK circuit.
//! It handles proof generation, verification, and key management.

use super::{types::*, ZkpConfig, Result};
use std::path::Path;
use std::hash::{Hash, Hasher};

/// Nori circuit adapter for ZK proof operations
pub struct NoriAdapter {
    config: ZkpConfig,
    proving_key: ProvingKey,
    pub verification_key: VerificationKey,
    circuit_metadata: CircuitMetadata,
}

impl NoriAdapter {
    /// Create a new Nori adapter with the given configuration
    pub fn new(config: &ZkpConfig) -> Result<Self> {
        // Load circuit metadata
        let circuit_metadata = Self::load_circuit_metadata(&config.circuit_path)?;

        // Load proving key
        let proving_key = Self::load_proving_key(&config.proving_key_path)?;

        // Load verification key
        let verification_key = Self::load_verification_key(&config.verification_key_path)?;

        Ok(Self {
            config: config.clone(),
            proving_key,
            verification_key,
            circuit_metadata,
        })
    }

    /// Generate a ZK proof from circuit inputs
    ///
    /// # Arguments
    ///
    /// * `inputs` - Circuit inputs (public and private)
    ///
    /// # Returns
    ///
    /// Returns a ZK proof that can be verified
    pub async fn generate_proof(&self, inputs: &CircuitInputs) -> Result<ZkProof> {
        let start_time = std::time::Instant::now();

        // In a real implementation, this would:
        // 1. Call compiled WASM circuit
        // 2. Use proving key to generate a Groth16 proof
        // 3. Serialize the proof

        // For now, simulate proof generation
        if self.config.debug {
            println!("🔧 Generating proof using Nori circuit...");
            println!("   Circuit: {}", self.circuit_metadata.name);
            println!("   Constraints: {}", self.circuit_metadata.num_constraints);
        }

        // Simulate WASM execution (would use node/snarkjs in production)
        let proof_bytes = self.simulate_proof_generation(inputs)?;

        // Build proof metadata
        let generation_time = start_time.elapsed().as_millis() as u64;
        let metadata = ProofMetadata {
            agent_id: "agent_unknown".to_string(), // Would be set by caller
            generation_time_ms: generation_time,
            memory_usage_bytes: 1024 * 1024, // 1MB estimate
            num_constraints: self.circuit_metadata.num_constraints,
        };

        // Create proof
        let proof = ZkProof {
            proof_bytes,
            public_inputs: inputs.public_inputs.clone(),
            circuit_hash: [0u8; 32], // Would compute actual hash
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        };

        if self.config.debug {
            println!("   ✅ Proof generated in {}ms", generation_time);
            println!("   Proof size: {} bytes", proof.proof_bytes.len());
        }

        Ok(proof)
    }

    /// Verify a ZK proof
    ///
    /// # Arguments
    ///
    /// * `proof` - The ZK proof to verify
    /// * `public_inputs` - The public inputs used in proof generation
    ///
    /// # Returns
    ///
    /// Returns true if the proof is valid
    pub async fn verify_proof(&self, proof: &ZkProof, public_inputs: &PublicInputs) -> Result<bool> {
        if self.config.debug {
            println!("🔍 Verifying proof...");
        }

        // In a real implementation, this would:
        // 1. Use the verification key
        // 2. Verify the proof against the public inputs
        // 3. Return the result

        // For now, simulate verification (always true for demonstration)
        let is_valid = self.simulate_verification(proof, public_inputs)?;

        if self.config.debug {
            if is_valid {
                println!("   ✅ Proof is valid");
            } else {
                println!("   ❌ Proof is invalid");
            }
        }

        Ok(is_valid)
    }

    /// Load circuit metadata from file
    fn load_circuit_metadata(_circuit_path: &str) -> Result<CircuitMetadata> {
        // In a real implementation, this would read from a compiled circuit file
        // For now, return default metadata
        Ok(CircuitMetadata::default())
    }

    /// Load proving key from file
    fn load_proving_key(path: &str) -> Result<ProvingKey> {
        let path = Path::new(path);

        if !path.exists() {
            // For development, return a mock key
            return Ok(ProvingKey {
                key_bytes: vec![0u8; 1024],
                circuit_id: "causal_fingerprint".to_string(),
                key_hash: [0u8; 32],
            });
        }

        // In a real implementation, this would load from file
        let key_bytes = std::fs::read(path)?;
        let key_hash = Self::compute_key_hash(&key_bytes);

        Ok(ProvingKey {
            key_bytes,
            circuit_id: "causal_fingerprint".to_string(),
            key_hash,
        })
    }

    /// Load verification key from file
    fn load_verification_key(path: &str) -> Result<VerificationKey> {
        let path = Path::new(path);

        if !path.exists() {
            // For development, return a mock key
            return Ok(VerificationKey {
                key_bytes: vec![0u8; 512],
                circuit_id: "causal_fingerprint".to_string(),
                version: "0.1.0".to_string(),
                key_hash: [0u8; 32],
            });
        }

        // In a real implementation, this would load from file
        let key_bytes = std::fs::read(path)?;
        let key_hash = Self::compute_key_hash(&key_bytes);

        Ok(VerificationKey {
            key_bytes,
            circuit_id: "causal_fingerprint".to_string(),
            version: "0.1.0".to_string(),
            key_hash,
        })
    }

    /// Compute hash of key bytes
    fn compute_key_hash(bytes: &[u8]) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();
        let mut bytes_out = [0u8; 32];
        bytes_out.copy_from_slice(&hash.to_be_bytes()[..32]);
        bytes_out
    }

    /// Simulate proof generation (for development)
    fn simulate_proof_generation(&self, inputs: &CircuitInputs) -> Result<Vec<u8>> {
        // In production, this would:
        // 1. Serialize inputs to WASM format
        // 2. Execute WASM circuit
        // 3. Call snarkjs groth16 fullProve
        // 4. Serialize the proof

        // For simulation, generate a mock proof
        let proof_size = 1024; // Typical Groth16 proof size
        let mut proof_bytes = vec![0u8; proof_size];

        // Include some data from inputs for realism
        let input_hash = inputs.public_inputs.hash();
        proof_bytes[0..32].copy_from_slice(&input_hash);

        Ok(proof_bytes)
    }

    /// Simulate proof verification (for development)
    fn simulate_verification(&self, proof: &ZkProof, public_inputs: &PublicInputs) -> Result<bool> {
        // In production, this would:
        // 1. Call snarkjs groth16 verify
        // 2. Verify the proof against the public inputs and verification key
        // For simulation, verify that input hash matches
        let expected_hash = public_inputs.hash();
        let actual_hash = &proof.proof_bytes[0..32];

        Ok(expected_hash == actual_hash.try_into().unwrap_or([0u8; 32]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nori_adapter_creation() {
        let config = ZkpConfig::default();
        let adapter = NoriAdapter::new(&config);
        assert!(adapter.is_ok());
    }

    #[tokio::test]
    async fn test_proof_generation() {
        let config = ZkpConfig::default();
        let adapter = NoriAdapter::new(&config).unwrap();

        let inputs = create_test_inputs();

        let proof = adapter.generate_proof(&inputs).await;
        assert!(proof.is_ok());

        let proof = proof.unwrap();
        assert_eq!(proof.public_inputs.intervention_vector.len(), 5);
        assert_eq!(proof.public_inputs.delta_response.len(), 5);
        assert_eq!(proof.public_inputs.expected_eigenvalues.len(), 3);
    }

    #[tokio::test]
    async fn test_proof_verification() {
        let config = ZkpConfig::default();
        let adapter = NoriAdapter::new(&config).unwrap();

        let inputs = create_test_inputs();
        let proof = adapter.generate_proof(&inputs).await.unwrap();

        let result = adapter.verify_proof(&proof, &inputs.public_inputs).await;
        assert!(result.is_ok());

        let is_valid = result.unwrap();
        // Simulation should always validate matching hashes
        assert!(is_valid);
    }

    fn create_test_inputs() -> CircuitInputs {
        let _spectral_features = crate::consensus::SpectralFeatures {
            eigenvalues: vec![5.0, 3.0, 1.0, 0.5, 0.2, 0.1, 0.05, 0.02],
            spectral_radius: 5.0,
            trace: 9.87,
            rank: 4,
            entropy: 0.75,
            timestamp: 0,
        };

        let public_inputs = PublicInputs {
            intervention_vector: vec![1.0, -1.0, 0.5, -0.5, 0.0],
            delta_response: vec![1.2, 0.8, 1.5, -0.3, 0.9],
            expected_eigenvalues: vec![5.0, 3.0, 1.0],
            spectral_radius: 5.0,
            spectral_entropy: 0.75,
            cosine_similarity: 0.9,
        };

        let private_inputs = PrivateInputs {
            response_history: vec![1.0; 50],
            covariance_matrix: vec![0.5; 25],
            eigenvectors: vec![0.1; 15],
        };

        CircuitInputs {
            public_inputs,
            private_inputs,
        }
    }
}
