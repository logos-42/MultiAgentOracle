#!/bin/bash
# Quick Test Script for ZK Causal Fingerprint Experiment

echo "============================================"
echo "ZK Causal Fingerprint - Quick Test Suite"
echo "============================================"
echo ""

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}[Test 1]${NC} Running default configuration..."
cargo run --example zk_fingerprint_experiment
echo ""
read -p "Press Enter to continue to next test..."

echo -e "${BLUE}[Test 2]${NC} Testing conservative agents..."
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_conservative.json
echo ""
read -p "Press Enter to continue to next test..."

echo -e "${BLUE}[Test 3]${NC} Testing aggressive agents (3 runs)..."
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_aggressive.json --runs 3
echo ""
read -p "Press Enter to continue to next test..."

echo -e "${BLUE}[Test 4]${NC} Testing mixed agents via command line..."
cargo run --example zk_fingerprint_experiment -- --agents analytical=3,cautious=2,aggressive=2,neutral=2,suspicious=1
echo ""
read -p "Press Enter to continue to next test..."

echo -e "${BLUE}[Test 5]${NC} Running statistical analysis (5 runs)..."
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_mixed.json --runs 5
echo ""

echo "============================================"
echo -e "${GREEN}All tests completed!${NC}"
echo "============================================"
echo ""
echo "Check TESTING_GUIDE.md for detailed explanation of results."
read -p "Press Enter to exit..."
