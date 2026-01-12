@echo off
REM Quick Test Script for ZK Causal Fingerprint Experiment
echo ============================================
echo ZK Causal Fingerprint - Quick Test Suite
echo ============================================
echo.

REM Test 1: Default configuration
echo [Test 1] Running default configuration...
cargo run --example zk_fingerprint_experiment
echo.
echo Press any key to continue to next test...
pause > nul

REM Test 2: Conservative agents
echo [Test 2] Testing conservative agents...
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_conservative.json
echo.
echo Press any key to continue to next test...
pause > nul

REM Test 3: Aggressive agents with multiple runs
echo [Test 3] Testing aggressive agents (3 runs)...
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_aggressive.json --runs 3
echo.
echo Press any key to continue to next test...
pause > nul

REM Test 4: Mixed agents via command line
echo [Test 4] Testing mixed agents via command line...
cargo run --example zk_fingerprint_experiment -- --agents analytical=3,cautious=2,aggressive=2,neutral=2,suspicious=1
echo.
echo Press any key to continue to next test...
pause > nul

REM Test 5: Statistical analysis
echo [Test 5] Running statistical analysis (5 runs)...
cargo run --example zk_fingerprint_experiment -- --config examples/configs/test_mixed.json --runs 5
echo.

echo ============================================
echo All tests completed!
echo ============================================
echo.
echo Check TESTING_GUIDE.md for detailed explanation of results.
pause
