//! Comprehensive Test Runner for Caxton
//! 
//! Orchestrates all test suites and provides coverage reporting:
//! - Property-based tests with quickcheck
//! - Integration tests with testcontainers
//! - Performance benchmarks with criterion
//! - Coverage analysis and reporting

use std::env;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Caxton Comprehensive Test Suite");
    println!("====================================\n");
    
    let start_time = Instant::now();
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let test_type = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let coverage = args.contains(&"--coverage".to_string());
    
    match test_type {
        "unit" => {
            println!("🔬 Running Unit Tests (Property-based)");
            let result = run_unit_tests().await?;
            total_tests += result.total;
            passed_tests += result.passed;
            failed_tests += result.failed;
        }
        "integration" => {
            println!("🔗 Running Integration Tests");
            let result = run_integration_tests().await?;
            total_tests += result.total;
            passed_tests += result.passed;
            failed_tests += result.failed;
        }
        "benchmarks" => {
            println!("⚡ Running Performance Benchmarks");
            let result = run_benchmarks().await?;
            total_tests += result.total;
            passed_tests += result.passed;
            failed_tests += result.failed;
        }
        "all" | _ => {
            // Run all test suites
            println!("🔬 Running Unit Tests (Property-based)");
            let unit_result = run_unit_tests().await?;
            total_tests += unit_result.total;
            passed_tests += unit_result.passed;
            failed_tests += unit_result.failed;
            
            println!("\n🔗 Running Integration Tests");
            let integration_result = run_integration_tests().await?;
            total_tests += integration_result.total;
            passed_tests += integration_result.passed;
            failed_tests += integration_result.failed;
            
            println!("\n⚡ Running Performance Benchmarks");
            let benchmark_result = run_benchmarks().await?;
            total_tests += benchmark_result.total;
            passed_tests += benchmark_result.passed;
            failed_tests += benchmark_result.failed;
        }
    }
    
    // Generate coverage report if requested
    if coverage {
        println!("\n📊 Generating Coverage Report");
        generate_coverage_report().await?;
    }
    
    let total_time = start_time.elapsed();
    
    // Print summary
    println!("\n📋 Test Summary");
    println!("===============");
    println!("Total Tests: {}", total_tests);
    println!("✅ Passed: {}", passed_tests);
    println!("❌ Failed: {}", failed_tests);
    println!("⏱️  Total Time: {:?}", total_time);
    println!("📊 Success Rate: {:.1}%", (passed_tests as f64 / total_tests as f64) * 100.0);
    
    if failed_tests > 0 {
        println!("\n⚠️  Some tests failed. Check output above for details.");
        std::process::exit(1);
    } else {
        println!("\n🎉 All tests passed!");
    }
    
    Ok(())
}

#[derive(Debug)]
struct TestResult {
    total: usize,
    passed: usize,
    failed: usize,
}

async fn run_unit_tests() -> Result<TestResult, Box<dyn std::error::Error>> {
    println!("  🧮 Property-based tests for core types");
    
    let output = Command::new("cargo")
        .args(&["test", "--lib", "--", "--test-threads=1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Parse test results
    let result = parse_test_output(&stdout);
    
    if !output.status.success() {
        println!("❌ Unit tests failed:");
        println!("{}", stderr);
    } else {
        println!("✅ Unit tests passed: {}/{}", result.passed, result.total);
    }
    
    Ok(result)
}

async fn run_integration_tests() -> Result<TestResult, Box<dyn std::error::Error>> {
    let mut total_result = TestResult { total: 0, passed: 0, failed: 0 };
    
    // Agent coordination tests
    println!("  🤝 Agent coordination and lifecycle");
    let coord_result = run_test_suite("agent_coordination").await?;
    total_result.total += coord_result.total;
    total_result.passed += coord_result.passed;
    total_result.failed += coord_result.failed;
    
    // FIPA messaging tests
    println!("  📨 FIPA messaging protocol compliance");
    let fipa_result = run_test_suite("fipa_messaging").await?;
    total_result.total += fipa_result.total;
    total_result.passed += fipa_result.passed;
    total_result.failed += fipa_result.failed;
    
    // WASM isolation tests
    println!("  🔒 WebAssembly isolation boundaries");
    let wasm_result = run_test_suite("wasm_isolation").await?;
    total_result.total += wasm_result.total;
    total_result.passed += wasm_result.passed;
    total_result.failed += wasm_result.failed;
    
    // Observability tests
    println!("  📊 Observability and telemetry");
    let obs_result = run_test_suite("observability").await?;
    total_result.total += obs_result.total;
    total_result.passed += obs_result.passed;
    total_result.failed += obs_result.failed;
    
    println!("✅ Integration tests: {}/{}", total_result.passed, total_result.total);
    
    Ok(total_result)
}

async fn run_benchmarks() -> Result<TestResult, Box<dyn std::error::Error>> {
    println!("  ⚡ Message throughput and latency");
    println!("  🚀 Agent spawning performance");
    println!("  🧠 Memory usage patterns");
    println!("  ⚖️  Resource contention handling");
    println!("  📈 Scaling characteristics");
    
    let output = Command::new("cargo")
        .args(&["test", "benchmark", "--release", "--", "--test-threads=1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let result = parse_test_output(&stdout);
    
    if !output.status.success() {
        println!("❌ Benchmarks failed:");
        println!("{}", stderr);
    } else {
        println!("✅ Benchmarks completed: {}/{}", result.passed, result.total);
    }
    
    Ok(result)
}

async fn run_test_suite(suite_name: &str) -> Result<TestResult, Box<dyn std::error::Error>> {
    let output = Command::new("cargo")
        .args(&["test", &format!("integration::{}", suite_name), "--", "--test-threads=1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let result = parse_test_output(&stdout);
    
    if !output.status.success() {
        println!("    ❌ {} tests failed", suite_name);
        if !stderr.trim().is_empty() {
            println!("       {}", stderr);
        }
    } else {
        println!("    ✅ {} tests: {}/{}", suite_name, result.passed, result.total);
    }
    
    Ok(result)
}

async fn generate_coverage_report() -> Result<(), Box<dyn std::error::Error>> {
    // Check if tarpaulin is installed
    let tarpaulin_check = Command::new("cargo")
        .args(&["tarpaulin", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    
    if !tarpaulin_check.success() {
        println!("  ⚠️  cargo-tarpaulin not installed. Installing...");
        let install_output = Command::new("cargo")
            .args(&["install", "cargo-tarpaulin"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        
        if !install_output.status.success() {
            println!("  ❌ Failed to install cargo-tarpaulin");
            return Ok(());
        }
    }
    
    // Generate coverage report
    println!("  📊 Generating coverage report with tarpaulin...");
    let coverage_output = Command::new("cargo")
        .args(&[
            "tarpaulin",
            "--out", "Html",
            "--output-dir", "target/coverage",
            "--exclude-files", "target/*",
            "--exclude-files", "tests/*",
            "--ignore-panics",
            "--timeout", "300"
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    let coverage_stdout = String::from_utf8_lossy(&coverage_output.stdout);
    let coverage_stderr = String::from_utf8_lossy(&coverage_output.stderr);
    
    if coverage_output.status.success() {
        // Extract coverage percentage
        if let Some(coverage_line) = coverage_stdout.lines().find(|line| line.contains("coverage:")) {
            println!("  ✅ {}", coverage_line);
        } else {
            println!("  ✅ Coverage report generated");
        }
        println!("  📄 Report saved to: target/coverage/tarpaulin-report.html");
    } else {
        println!("  ❌ Coverage generation failed:");
        println!("     {}", coverage_stderr);
    }
    
    Ok(())
}

fn parse_test_output(output: &str) -> TestResult {
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    
    // Parse cargo test output
    for line in output.lines() {
        if line.contains("test result:") {
            // Extract test counts from line like "test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
            let parts: Vec<&str> = line.split_whitespace().collect();
            
            for (i, part) in parts.iter().enumerate() {
                if *part == "passed;" && i > 0 {
                    if let Ok(count) = parts[i - 1].parse::<usize>() {
                        passed = count;
                        total += count;
                    }
                }
                if *part == "failed;" && i > 0 {
                    if let Ok(count) = parts[i - 1].parse::<usize>() {
                        failed = count;
                        total += count;
                    }
                }
            }
        }
    }
    
    TestResult { total, passed, failed }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_test_output() {
        let output = "test result: ok. 15 passed; 2 failed; 0 ignored; 0 measured; 5 filtered out; finished in 1.23s";
        let result = parse_test_output(output);
        
        assert_eq!(result.total, 17);
        assert_eq!(result.passed, 15);
        assert_eq!(result.failed, 2);
    }
    
    #[test]
    fn test_parse_empty_output() {
        let output = "";
        let result = parse_test_output(output);
        
        assert_eq!(result.total, 0);
        assert_eq!(result.passed, 0);
        assert_eq!(result.failed, 0);
    }
}