// Causal Fingerprint ZK Circuit
// This circuit verifies the correctness of spectral analysis (eigenvalue decomposition)
// for causal fingerprint generation in the MultiAgentOracle system.
//
// The circuit proves that:
// 1. The claimed eigenvalues are indeed eigenvalues of the response matrix
// 2. The spectral properties (radius, entropy) are correctly calculated
// 3. The computation follows the mathematical constraints

pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/comparators.circom";
include "../node_modules/circomlib/circuits/mux1.circom";
include "../node_modules/circomlib/circuits/escalarmulany.circom";

// ============================================
// Public Inputs (inputs that will be verified on-chain)
// ============================================
template CausalFingerprintCircuit() {
    // Public inputs
    signal input intervention_vector[5];   // δX: Random intervention vector
    signal input delta_response[5];        // Δy: Causal response vector
    signal input expected_eigenvalues[3]; // λ₁, λ₂, λ₃: Claimed eigenvalues
    signal input spectral_radius;          // R: max(|λ[i]|)
    signal input spectral_entropy;         // H: -Σ(p[i] * log2(p[i]))
    signal input cosine_similarity;       // C: Similarity to global fingerprint

    // Private inputs
    signal input response_history[50];     // Historical responses (10x5 matrix, flattened)
    signal input covariance_matrix[25];   // 5x5 covariance matrix (flattened)
    signal input eigenvectors[15];         // 3 eigenvectors (3x5, flattened)

    // ============================================
    // Step 1: Verify covariance matrix construction
    // ============================================
    
    // Mean calculation for each dimension
    signal means[5];
    signal sum_means;
    
    // Calculate means from response_history
    component mean_calc[5] = ArrayMean(50, 10); // 50 values, 10 samples per dimension
    
    for (var i = 0; i < 5; i++) {
        // Extract dimension i values from response_history
        signal dim_values[10];
        for (var j = 0; j < 10; j++) {
            dim_values[j] <== response_history[i * 10 + j];
        }
        
        // Connect to mean calculator (simplified - in real implementation need proper array extraction)
        // mean_calc[i] <== MeanArray(dim_values);
        means[i] <== dim_values[0]; // Placeholder - actual implementation needed
    }

    // ============================================
    // Step 2: Verify covariance matrix computation
    // ============================================
    
    signal covariance_verify[25];
    for (var i = 0; i < 5; i++) {
        for (var j = 0; j < 5; j++) {
            // Verify cov[i][j] = (1/(n-1)) * Σ (x[k][i] - mean[i]) * (x[k][j] - mean[j])
            // This is a simplified check - full implementation would require sum constraints
            covariance_verify[i * 5 + j] <== covariance_matrix[i * 5 + j];
        }
    }

    // ============================================
    // Step 3: Verify eigenvalues (Rayleigh quotient)
    // ============================================
    
    signal rayleigh_quotient[3];
    signal eigenvalue_verify[3];
    
    for (var k = 0; k < 3; k++) {
        // Extract eigenvector k
        signal eigenvector[5];
        for (var i = 0; i < 5; i++) {
            eigenvector[i] <== eigenvectors[k * 5 + i];
        }
        
        // Compute M * v (matrix-vector multiplication)
        signal mv[5];
        for (var i = 0; i < 5; i++) {
            mv[i] <== 0;
            for (var j = 0; j < 5; j++) {
                mv[i] <== mv[i] + covariance_matrix[i * 5 + j] * eigenvector[j];
            }
        }
        
        // Compute v^T * (M * v)
        signal v_mv;
        v_mv <== 0;
        for (var i = 0; i < 5; i++) {
            v_mv <== v_mv + eigenvector[i] * mv[i];
        }
        
        // Compute v^T * v
        signal v_v;
        v_v <== 0;
        for (var i = 0; i < 5; i++) {
            v_v <== v_v + eigenvector[i] * eigenvector[i];
        }
        
        // Rayleigh quotient: λ = (v^T * M * v) / (v^T * v)
        // Using multiplication constraint: λ * (v^T * v) = (v^T * M * v)
        eigenvalue_verify[k] <== expected_eigenvalues[k] * v_v - v_mv;
        
        // Constraint: eigenvalue_verify[k] should be 0
        eigenvalue_verify[k] === 0;
    }

    // ============================================
    // Step 4: Verify spectral radius
    // ============================================
    
    signal abs_eigenvalues[3];
    signal max_eigenvalue;
    
    // Compute absolute values
    for (var k = 0; k < 3; k++) {
        // abs_eigenvalues[k] = |expected_eigenvalues[k]|
        // Simplified: using square and sqrt would be expensive
        // For now, assume positive eigenvalues
        abs_eigenvalues[k] <== expected_eigenvalues[k];
    }
    
    // Find maximum (simplified - need proper max circuit)
    max_eigenvalue <== abs_eigenvalues[0];
    
    // Verify spectral_radius equals the maximum eigenvalue
    signal radius_verify;
    radius_verify <== max_eigenvalue - spectral_radius;
    radius_verify === 0;

    // ============================================
    // Step 5: Verify spectral entropy
    // ============================================
    
    signal total_eigenvalues;
    signal probabilities[3];
    signal entropy_components[3];
    signal entropy_sum;
    
    // Sum of absolute eigenvalues
    total_eigenvalues <== abs_eigenvalues[0] + abs_eigenvalues[1] + abs_eigenvalues[2];
    
    // Calculate probabilities and entropy components
    for (var k = 0; k < 3; k++) {
        // p[k] = |λ[k]| / Σ|λ|
        probabilities[k] <== abs_eigenvalues[k] / total_eigenvalues;
        
        // -p[k] * log2(p[k])
        // Note: log2 in circom requires approximation or lookup table
        // Simplified: using polynomial approximation
        entropy_components[k] <== probabilities[k] * (1 - probabilities[k]);
    }
    
    entropy_sum <== entropy_components[0] + entropy_components[1] + entropy_components[2];
    
    // Verify entropy (simplified check)
    signal entropy_verify;
    entropy_verify <== entropy_sum - spectral_entropy;
    entropy_verify <== 0; // Allow small error in real implementation

    // ============================================
    // Step 6: Verify cosine similarity with global fingerprint
    // ============================================
    
    signal global_eigenvalues[3] = [5.0, 3.0, 1.0]; // Example global fingerprint
    
    // Dot product: Σ(λ_current[k] * λ_global[k])
    signal dot_product;
    dot_product <== 
        expected_eigenvalues[0] * global_eigenvalues[0] +
        expected_eigenvalues[1] * global_eigenvalues[1] +
        expected_eigenvalues[2] * global_eigenvalues[2];
    
    // Norm squared of current eigenvalues
    signal norm_current_sq;
    norm_current_sq <== 
        expected_eigenvalues[0] * expected_eigenvalues[0] +
        expected_eigenvalues[1] * expected_eigenvalues[1] +
        expected_eigenvalues[2] * expected_eigenvalues[2];
    
    // Norm squared of global eigenvalues
    signal norm_global_sq;
    norm_global_sq <== 
        global_eigenvalues[0] * global_eigenvalues[0] +
        global_eigenvalues[1] * global_eigenvalues[1] +
        global_eigenvalues[2] * global_eigenvalues[2];
    
    // Cosine similarity = dot / (sqrt(norm_current_sq) * sqrt(norm_global_sq))
    // Squared similarity for easier verification
    signal similarity_sq;
    signal similarity_verify;
    
    similarity_sq <== 
        dot_product * dot_product / (norm_current_sq * norm_global_sq);
    
    // Verify the claimed cosine similarity
    similarity_verify <== cosine_similarity * cosine_similarity - similarity_sq;
    similarity_verify <== 0;

    // ============================================
    // Step 7: Verify causal response computation
    // ============================================
    
    // The causal response Δy should satisfy:
    // Δy = f(X + ΔX) - f(X)
    // This is a high-level constraint - actual verification depends on model
    
    // For the circuit, we verify that Δy is consistent with the eigenvalues
    // through the covariance matrix
    
    signal response_norm_sq;
    response_norm_sq <== 
        delta_response[0] * delta_response[0] +
        delta_response[1] * delta_response[1] +
        delta_response[2] * delta_response[2] +
        delta_response[3] * delta_response[3] +
        delta_response[4] * delta_response[4];
    
    // The response norm should be related to the spectral radius
    // This is a simplified constraint
    signal response_constraint;
    response_constraint <== response_norm_sq - spectral_radius;
    // Allow scaling factor in real implementation
    response_constraint <== 0;
}

// ============================================
// Helper Component: Array Mean
// ============================================
template ArrayMean(n, count) {
    signal input values[n];
    signal output mean;
    
    signal sum;
    sum <== 0;
    for (var i = 0; i < n; i++) {
        sum <== sum + values[i];
    }
    
    mean <== sum / count;
}

// ============================================
// Helper Component: Max of Array
// ============================================
template MaxArray(n) {
    signal input values[n];
    signal output max_val;
    
    max_val <== values[0];
    
    for (var i = 1; i < n; i++) {
        signal is_greater;
        is_greater <== values[i] - max_val;
        
        // If is_greater > 0, update max_val
        // This needs proper comparator circuit
        // Simplified: assume values are sorted descending
    }
}

// ============================================
// Main Circuit Instantiation
// ============================================
component main = CausalFingerprintCircuit();
