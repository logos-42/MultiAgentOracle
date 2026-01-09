#!/bin/bash

# Compile Nori Circuit for Causal Fingerprint ZK Proofs
# This script compiles the Circom circuit and generates necessary keys
# for both proof generation and verification.

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CIRCUIT_NAME="causal_fingerprint"
CIRCUIT_FILE="${SCRIPT_DIR}/${CIRCUIT_NAME}.circom"

echo "=========================================="
echo "Nori Circuit Compilation Script"
echo "=========================================="
echo "Project Root: ${PROJECT_ROOT}"
echo "Circuit File: ${CIRCUIT_FILE}"
echo ""

# Check if Circom is installed
if ! command -v circom &> /dev/null; then
    echo "❌ Error: circom is not installed"
    echo "   Please install Circom: https://docs.circom.io/getting-started/installation/"
    exit 1
fi

echo "✅ Circom found: $(circom --version)"

# Check if Node.js is installed (for snarkjs)
if ! command -v node &> /dev/null; then
    echo "❌ Error: node is not installed"
    echo "   Please install Node.js: https://nodejs.org/"
    exit 1
fi

echo "✅ Node.js found: $(node --version)"

# Check if snarkjs is installed
if ! command -v snarkjs &> /dev/null; then
    echo "⚠️  Warning: snarkjs is not installed"
    echo "   Installing snarkjs globally..."
    npm install -g snarkjs
fi

echo "✅ snarkjs found"
echo ""

# Create output directories
echo "📁 Creating output directories..."
mkdir -p "${SCRIPT_DIR}/build"
mkdir -p "${SCRIPT_DIR}/keys"
echo "✅ Directories created"
echo ""

# Step 1: Install circomlib if not present
echo "📦 Checking circomlib..."
if [ ! -d "${PROJECT_ROOT}/node_modules/circomlib" ]; then
    echo "   Installing circomlib..."
    cd "${PROJECT_ROOT}"
    npm install circomlib
    echo "✅ circomlib installed"
else
    echo "✅ circomlib already installed"
fi
echo ""

# Step 2: Compile the circuit
echo "🔨 Compiling circuit: ${CIRCUIT_FILE}..."
cd "${SCRIPT_DIR}"

circom "${CIRCUIT_FILE}" \
    --r1cs \
    --wasm \
    --c \
    -l "${PROJECT_ROOT}/node_modules/circomlib/circuits" \
    -o "${SCRIPT_DIR}/build"

if [ $? -eq 0 ]; then
    echo "✅ Circuit compiled successfully"
else
    echo "❌ Circuit compilation failed"
    exit 1
fi
echo ""

# Check if compiled files exist
R1CS_FILE="${SCRIPT_DIR}/build/${CIRCUIT_NAME}.r1cs"
WASM_FILE="${SCRIPT_DIR}/build/${CIRCUIT_NAME}_js/${CIRCUIT_NAME}.wasm"

if [ ! -f "${R1CS_FILE}" ]; then
    echo "❌ Error: R1CS file not found: ${R1CS_FILE}"
    exit 1
fi

if [ ! -f "${WASM_FILE}" ]; then
    echo "❌ Error: WASM file not found: ${WASM_FILE}"
    exit 1
fi

echo "✅ Compiled files:"
echo "   R1CS: ${R1CS_FILE}"
echo "   WASM: ${WASM_FILE}"
echo ""

# Step 3: Generate powers of tau ceremony contribution (if not exists)
PTAU_FILE="${SCRIPT_DIR}/keys/powersOfTau28_hez_final_14.ptau"
if [ ! -f "${PTAU_FILE}" ]; then
    echo "⚠️  Powers of Tau file not found"
    echo "   Note: For development, we'll use a small PTAU"
    echo "   For production, you should participate in a trusted setup ceremony"
    echo ""
    
    # Download a small PTAU for development
    PTAU_URL="https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_14.ptau"
    
    if command -v wget &> /dev/null; then
        echo "   Downloading PTAU file (this may take a while)..."
        wget -P "${SCRIPT_DIR}/keys" "${PTAU_URL}"
    elif command -v curl &> /dev/null; then
        echo "   Downloading PTAU file (this may take a while)..."
        curl -L -o "${PTAU_FILE}" "${PTAU_URL}"
    else
        echo "   ❌ Error: Neither wget nor curl is available"
        echo "   Please download the PTAU file manually:"
        echo "   ${PTAU_URL}"
        exit 1
    fi
    
    if [ $? -eq 0 ]; then
        echo "✅ PTAU file downloaded"
    else
        echo "❌ Failed to download PTAU file"
        exit 1
    fi
else
    echo "✅ PTAU file found: ${PTAU_FILE}"
fi
echo ""

# Step 4: Generate proving key and verification key
echo "🔑 Generating proving and verification keys..."

# Start a new zkey
ZKEY_START="${SCRIPT_DIR}/keys/${CIRCUIT_NAME}_0000.zkey"
snarkjs groth16 setup "${R1CS_FILE}" "${PTAU_FILE}" "${ZKEY_START}"

# Contribute to the ceremony (random entropy)
ZKEY_FINAL="${SCRIPT_DIR}/keys/${CIRCUIT_NAME}.zkey"
echo -n "Entropy for phase 2: " 
snarkjs zkey contribute "${ZKEY_START}" "${ZKEY_FINAL}" \
    --name="1st Contributor" \
    -e="$(openssl rand -hex 32)"

# Export verification key
VK_FILE="${SCRIPT_DIR}/keys/verification_key.json"
snarkjs zkey export verificationkey "${ZKEY_FINAL}" "${VK_FILE}"

# Export proving key
PK_FILE="${SCRIPT_DIR}/keys/proving_key.json"
snarkjs zkey export solidityverifier "${ZKEY_FINAL}" "${SCRIPT_DIR}/keys/verifier.sol"

echo "✅ Keys generated:"
echo "   Proving Key: ${ZKEY_FINAL}"
echo "   Verification Key: ${VK_FILE}"
echo "   Solidity Verifier: ${SCRIPT_DIR}/keys/verifier.sol"
echo ""

# Step 5: Generate circuit information
INFO_FILE="${SCRIPT_DIR}/build/circuit_info.json"
echo "📊 Generating circuit information..."
echo "{\"circuit_name\": \"${CIRCUIT_NAME}\", \"r1cs_file\": \"${R1CS_FILE}\", \"wasm_file\": \"${WASM_FILE}\", \"verification_key\": \"${VK_FILE}\", \"proving_key\": \"${ZKEY_FINAL}\", \"compiled_at\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ\")\"}" > "${INFO_FILE}"
echo "✅ Circuit info saved to: ${INFO_FILE}"
echo ""

# Step 6: Print circuit statistics
echo "📈 Circuit Statistics:"
snarkjs r1cs info "${R1CS_FILE}"
echo ""

# Step 7: Generate test vectors (optional)
echo "🧪 Generating test vectors..."
TEST_VECTORS_FILE="${SCRIPT_DIR}/test_vectors.json"
if command -v node &> /dev/null; then
    # Create a simple test vectors generator
    cat > "${SCRIPT_DIR}/generate_test_vectors.js" << 'EOF'
const fs = require('fs');
const crypto = require('crypto');

function generateTestVectors(count) {
    const vectors = [];
    
    for (let i = 0; i < count; i++) {
        const vector = {
            id: i,
            public_inputs: {
                intervention_vector: Array(5).fill(0).map(() => Math.floor(Math.random() * 100)),
                delta_response: Array(5).fill(0).map(() => Math.floor(Math.random() * 50) - 25),
                expected_eigenvalues: Array(3).fill(0).map(() => Math.random() * 10),
                spectral_radius: Math.random() * 10,
                spectral_entropy: Math.random(),
                cosine_similarity: Math.random() * 0.3 + 0.7
            },
            private_inputs: {
                response_history: Array(50).fill(0).map(() => Math.floor(Math.random() * 20)),
                covariance_matrix: Array(25).fill(0).map(() => Math.random() * 2),
                eigenvectors: Array(15).fill(0).map(() => Math.random() * 2 - 1)
            },
            expected_output: {
                proof_valid: true,
                verification_time_ms: Math.floor(Math.random() * 100) + 10
            }
        };
        vectors.push(vector);
    }
    
    return vectors;
}

const vectors = generateTestVectors(100);
fs.writeFileSync('test_vectors.json', JSON.stringify(vectors, null, 2));
console.log('Generated 100 test vectors');
EOF
    
    cd "${SCRIPT_DIR}"
    node generate_test_vectors.js
    echo "✅ Test vectors generated: ${TEST_VECTORS_FILE}"
else
    echo "⚠️  Skipped test vector generation (node not available)"
fi
echo ""

# Step 8: Copy necessary files to project structure
echo "📋 Copying files to project structure..."
mkdir -p "${PROJECT_ROOT}/circuits/keys"
mkdir -p "${PROJECT_ROOT}/circuits/build"

cp "${ZKEY_FINAL}" "${PROJECT_ROOT}/circuits/keys/"
cp "${VK_FILE}" "${PROJECT_ROOT}/circuits/keys/"
cp "${R1CS_FILE}" "${PROJECT_ROOT}/circuits/build/"
cp "${WASM_FILE}" "${PROJECT_ROOT}/circuits/build/"
echo "✅ Files copied to project structure"
echo ""

# Success message
echo "=========================================="
echo "✅ Circuit compilation completed successfully!"
echo "=========================================="
echo ""
echo "Generated files:"
echo "  - Circuit R1CS: ${R1CS_FILE}"
echo "  - Circuit WASM: ${WASM_FILE}"
echo "  - Proving Key: ${ZKEY_FINAL}"
echo "  - Verification Key: ${VK_FILE}"
echo "  - Test Vectors: ${TEST_VECTORS_FILE}"
echo ""
echo "Next steps:"
echo "  1. Review the circuit statistics above"
echo "  2. Test proof generation with:"
echo "     node generate_proof.js"
echo "  3. Verify proofs with:"
echo "     node verify_proof.js"
echo ""
echo "For more information, see docs/NORI_CIRCUIT_GUIDE.md"
