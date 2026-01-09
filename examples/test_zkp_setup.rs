//! Simple test to verify ZKP module setup

use multi_agent_oracle::zkp::{ZkpConfig, ZkpGenerator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing ZKP Module Setup");
    println!("================================");

    // Test 1: Create default configuration
    println!("\nTest 1: Creating ZKP config...");
    let config = ZkpConfig::default();
    println!("✅ Config created:");
    println!("   Circuit path: {}", config.circuit_path);
    println!("   Proving key: {}", config.proving_key_path);
    println!("   Verification key: {}", config.verification_key_path);
    println!("   Num eigenvalues: {}", config.num_eigenvalues);
    println!("   Scale factor: {}", config.scale_factor);

    // Test 2: Try to create ZKP generator (may fail if keys don't exist)
    println!("\nTest 2: Creating ZKP generator...");
    match ZkpGenerator::new() {
        Ok(generator) => {
            println!("✅ ZKP generator created successfully");
            println!("   Generator is ready for proof generation");
        }
        Err(e) => {
            println!("⚠️  ZKP generator creation failed (expected if keys not compiled):");
            println!("   Error: {}", e);
            println!("\n   This is normal if you haven't compiled the circuit yet.");
            println!("   To compile: cd circuits && bash compile_nori.sh");
        }
    }

    // Test 3: Verify configuration values
    println!("\nTest 3: Verifying configuration...");
    assert_eq!(config.num_eigenvalues, 3);
    assert_eq!(config.scale_factor, 1_000_000);
    println!("✅ Configuration values verified");

    println!("\n================================");
    println!("✅ Basic ZKP module test completed!");
    println!();
    println!("Next steps:");
    println!("  1. Compile Nori circuit: cd circuits && bash compile_nori.sh");
    println!("  2. Run full experiment: cargo run --example zk_fingerprint_experiment");

    Ok(())
}
