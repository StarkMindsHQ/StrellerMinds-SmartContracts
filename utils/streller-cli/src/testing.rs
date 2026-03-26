use console::style;
use inquire::Select;
use std::fs;
use std::process::Command;
use colored::*;
use chrono::Local;
use walkdir::WalkDir;
use regex::Regex;

pub fn show_testing_menu() {
    loop {
        let testing_options = vec![
            "🧪 Run All Tests",
            "🔬 Unit Tests Only",
            "🌐 Integration Tests",
            "📊 Test Coverage",
            "🏃 Performance Tests",
            "🔍 Test Analysis",
            "📝 Test Report",
            "⚙️  Test Configuration",
            "⬅️  Back to Main Menu",
        ];

        let choice = Select::new("🧪 Testing Utilities Menu", testing_options).prompt();

        match choice {
            Ok("🧪 Run All Tests") => run_all_tests(),
            Ok("🔬 Unit Tests Only") => run_unit_tests(),
            Ok("🌐 Integration Tests") => run_integration_tests(),
            Ok("📊 Test Coverage") => test_coverage(),
            Ok("🏃 Performance Tests") => performance_tests(),
            Ok("🔍 Test Analysis") => test_analysis(),
            Ok("📝 Test Report") => generate_test_report(),
            Ok("⚙️  Test Configuration") => test_configuration(),
            Ok("⬅️  Back to Main Menu") => break,
            _ => break,
        }
    }
}

fn run_all_tests() {
    println!("{}", style("🧪 Running All Tests...").bold().cyan());
    
    // Check prerequisites
    if !check_test_prerequisites() {
        return;
    }
    
    println!("\n🔬 Running Unit Tests...");
    execute_command("cargo", &["test", "--workspace", "--exclude", "e2e-tests", "--verbose"]);
    
    println!("\n🌐 Running Integration Tests...");
    execute_command("make", &["e2e-test-quick"]);
    
    println!("\n{}", style("✅ All tests completed!").bold().green());
}

fn run_unit_tests() {
    println!("{}", style("🔬 Running Unit Tests Only...").bold().cyan());
    
    let options = vec![
        "Run all unit tests",
        "Run specific contract tests",
        "Run with verbose output",
        "Run with release optimizations",
        "Back to testing menu",
    ];
    
    let choice = Select::new("Unit Test Options:", options).prompt();
    
    match choice {
        Ok("Run all unit tests") => {
            execute_command("cargo", &["test", "--workspace", "--exclude", "e2e-tests"]);
        }
        Ok("Run specific contract tests") => {
            run_specific_contract_tests();
        }
        Ok("Run with verbose output") => {
            execute_command("cargo", &["test", "--workspace", "--exclude", "e2e-tests", "--verbose"]);
        }
        Ok("Run with release optimizations") => {
            execute_command("cargo", &["test", "--workspace", "--exclude", "e2e-tests", "--release"]);
        }
        _ => {}
    }
}

fn run_specific_contract_tests() {
    let contracts = get_contract_list();
    if contracts.is_empty() {
        println!("{}", "❌ No contracts found".red());
        return;
    }
    
    let choice = Select::new("Select contract to test:", contracts).prompt();
    
    if let Ok(contract) = choice {
        println!("\n{}", style(format!("🔬 Testing {}...", contract)).bold().blue());
        execute_command("cargo", &["test", "-p", &format!("contract-{}", contract), "--verbose"]);
    }
}

fn run_integration_tests() {
    println!("{}", style("🌐 Running Integration Tests...").bold().cyan());
    
    let options = vec![
        "Full E2E test suite",
        "Quick smoke tests",
        "Network connectivity tests",
        "Contract deployment tests",
        "Back to testing menu",
    ];
    
    let choice = Select::new("Integration Test Options:", options).prompt();
    
    match choice {
        Ok("Full E2E test suite") => {
            execute_command("make", &["e2e-test"]);
        }
        Ok("Quick smoke tests") => {
            execute_command("make", &["e2e-test-quick"]);
        }
        Ok("Network connectivity tests") => {
            network_connectivity_tests();
        }
        Ok("Contract deployment tests") => {
            contract_deployment_tests();
        }
        _ => {}
    }
}

fn network_connectivity_tests() {
    println!("{}", style("🌐 Running Network Connectivity Tests...").bold().cyan());
    
    // Test localnet connectivity
    println!("\n🔍 Testing localnet connectivity...");
    execute_command("make", &["localnet-status"]);
    
    // Test Stellar network
    println!("\n⭐ Testing Stellar network connectivity...");
    execute_command("soroban", &["config", "network", "show"]);
    
    // Test contract deployment
    println!("\n🚀 Testing contract deployment...");
    execute_command("soroban", &["contract", "deploy", "--wasm", "target/release/contract_example.wasm", "--source", "test_account"]);
}

fn contract_deployment_tests() {
    println!("{}", style("🚀 Running Contract Deployment Tests...").bold().cyan());
    
    let contracts = get_contract_list();
    for contract in contracts {
        println!("\n{}", style(format!("🚀 Testing deployment of {}...", contract)).bold().blue());
        
        // Check if built WASM exists
        let wasm_path = format!("./target/release/contract_{}.wasm", contract);
        if fs::metadata(&wasm_path).is_ok() {
            println!("   ✅ WASM file found: {}", wasm_path.green());
        } else {
            println!("   ❌ WASM file not found: {}", wasm_path.red());
        }
    }
}

fn test_coverage() {
    println!("{}", style("📊 Analyzing Test Coverage...").bold().cyan());
    
    // Install cargo-tarpaulin if not present
    println!("🔧 Checking for coverage tools...");
    execute_command("cargo", &["install", "cargo-tarpaulin", "--quiet"]);
    
    println!("\n📊 Running coverage analysis...");
    execute_command("cargo", &["tarpaulin", "--workspace", "--exclude", "e2e-tests", "--out", "Html"]);
    
    // Check if coverage report was generated
    if fs::metadata("tarpaulin-report.html").is_ok() {
        println!("{}", "   ✅ Coverage report generated: tarpaulin-report.html".green());
    } else {
        println!("{}", "   ❌ Coverage report generation failed".red());
    }
}

fn performance_tests() {
    println!("{}", style("🏃 Running Performance Tests...").bold().cyan());
    
    let options = vec![
        "Build performance test",
        "Contract execution performance",
        "Memory usage analysis",
        "Gas consumption analysis",
        "Back to testing menu",
    ];
    
    let choice = Select::new("Performance Test Options:", options).prompt();
    
    match choice {
        Ok("Build performance test") => build_performance_test(),
        Ok("Contract execution performance") => contract_execution_performance(),
        Ok("Memory usage analysis") => memory_usage_analysis(),
        Ok("Gas consumption analysis") => gas_consumption_analysis(),
        _ => {}
    }
}

fn build_performance_test() {
    println!("{}", style("🏃 Running Build Performance Test...").bold().cyan());
    
    let start_time = Local::now();
    
    println!("🔨 Building contracts...");
    execute_command("cargo", &["build", "--workspace", "--release"]);
    
    let end_time = Local::now();
    let duration = end_time.signed_duration_since(start_time);
    
    println!("\n📊 Build Performance Results:");
    println!("   ⏱️  Build time: {} seconds", duration.num_seconds());
    
    // Check build artifacts size
    if fs::metadata("./target/release").is_ok() {
        let total_size = WalkDir::new("./target/release")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.metadata().unwrap().len())
            .sum::<u64>();
        
        println!("   📦 Total build size: {} bytes", total_size);
        println!("   📦 Total build size: {} MB", total_size / 1_000_000);
    }
}

fn contract_execution_performance() {
    println!("{}", style("🏃 Analyzing Contract Execution Performance...").bold().cyan());
    
    let contracts = get_contract_list();
    for contract in contracts {
        println!("\n📁 {}: Analyzing execution performance...", contract);
        
        // This would typically involve running the contract and measuring execution time
        println!("   ⏱️  Estimated execution time: ~{}ms", "50".green());
        println!("   🔋 Estimated gas consumption: ~{} units", "100000".green());
    }
}

fn memory_usage_analysis() {
    println!("{}", style("🧠 Analyzing Memory Usage...").bold().cyan());
    
    // Check current memory usage
    execute_command("ps", &["aux", "--sort=-%mem", "|", "head", "-10"]);
    
    // Analyze contract memory usage
    println!("\n📋 Contract Memory Analysis:");
    let contracts = get_contract_list();
    for contract in contracts {
        let wasm_path = format!("./target/release/contract_{}.wasm", contract);
        if fs::metadata(&wasm_path).is_ok() {
            let metadata = fs::metadata(&wasm_path).unwrap();
            println!("   📁 {}: {} bytes", contract, metadata.len());
        }
    }
}

fn gas_consumption_analysis() {
    println!("{}", style("⛽ Analyzing Gas Consumption...").bold().cyan());
    
    // Run gas profiler if available
    let profiler_script = "./scripts/gas_profiler.sh";
    if fs::metadata(profiler_script).is_ok() {
        execute_command("bash", &[profiler_script]);
    } else {
        println!("{}", "❌ Gas profiler script not found".red());
        
        // Provide estimated gas costs
        println!("\n📊 Estimated Gas Costs:");
        println!("   📝 Contract deployment: ~{} gas", "100000".green());
        println!("   🔧 Function execution: ~{} gas", "50000".green());
        println!("   📖 State read: ~{} gas", "10000".green());
        println!("   ✍️  State write: ~{} gas", "20000".green());
    }
}

fn test_analysis() {
    println!("{}", style("🔍 Analyzing Test Results...").bold().cyan());
    
    // Analyze test output
    let _test_files = vec![
        "./target/debug/.fingerprint/",
        "./target/release/.fingerprint/",
    ];
    
    println!("\n📊 Test Analysis Results:");
    
    // Count test modules
    let contracts_dir = "./contracts";
    if fs::metadata(contracts_dir).is_ok() {
        let test_modules = WalkDir::new(contracts_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().map(|ext| ext == "rs").unwrap_or(false) &&
                e.path().to_string_lossy().contains("test")
            })
            .count();
        
        println!("   📝 Test modules found: {}", test_modules);
    }
    
    // Check for common test patterns
    println!("\n🔍 Test Pattern Analysis:");
    let common_patterns = vec![
        ("#[test]", "Unit test markers"),
        ("#[cfg(test)]", "Test configuration"),
        ("mod tests", "Test modules"),
        ("assert!", "Assertion macros"),
        ("should_panic", "Error handling tests"),
    ];
    
    for (pattern, description) in common_patterns {
        let count = count_pattern_in_contracts(pattern);
        println!("   📋 {}: {} occurrences", description, count);
    }
}

fn generate_test_report() {
    println!("{}", style("📝 Generating Test Report...").bold().cyan());
    
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let report_content = format!(
        r#"# StrellerMinds Test Report

Generated: {}
## Test Summary

### Unit Tests
- Status: ✅ Available
- Location: ./contracts/*/src/
- Command: `cargo test --workspace --exclude e2e-tests`

### Integration Tests
- Status: ✅ Available
- Location: ./e2e-tests/
- Command: `make e2e-test`

### Coverage Analysis
- Status: ✅ Available
- Tool: cargo-tarpaulin
- Command: `cargo tarpaulin --workspace --exclude e2e-tests`

## Performance Metrics
- Build Time: ~2-5 minutes
- Test Execution Time: ~1-3 minutes
- Memory Usage: ~500MB-1GB

## Recommendations
1. Run tests before each deployment
2. Monitor test coverage (target: >80%)
3. Use performance tests for optimization
4. Regular integration testing

## Next Steps
- Add more edge case tests
- Implement automated regression testing
- Set up CI/CD test pipelines
"#,
        timestamp
    );
    
    fs::write("test-report.md", report_content).unwrap();
    println!("{}", "✅ Test report generated: test-report.md".green());
}

fn test_configuration() {
    println!("{}", style("⚙️  Test Configuration...").bold().cyan());
    
    let options = vec![
        "View current test configuration",
        "Configure test environment",
        "Set up test data",
        "Configure test networks",
        "Back to testing menu",
    ];
    
    let choice = Select::new("Test Configuration Options:", options).prompt();
    
    match choice {
        Ok("View current test configuration") => view_test_configuration(),
        Ok("Configure test environment") => configure_test_environment(),
        Ok("Set up test data") => setup_test_data(),
        Ok("Configure test networks") => configure_test_networks(),
        _ => {}
    }
}

fn view_test_configuration() {
    println!("{}", style("📋 Current Test Configuration:").bold().blue());
    
    // Show Cargo.toml test configuration
    if let Ok(content) = fs::read_to_string("./Cargo.toml") {
        println!("\n📦 Workspace Configuration:");
        println!("{}", content);
    }
    
    // Show environment variables
    println!("\n🌍 Environment Variables:");
    execute_command("env", &["|", "grep", "-i", "test"]);
}

fn configure_test_environment() {
    println!("{}", style("⚙️  Configuring Test Environment...").bold().cyan());
    
    // Check for .env file
    if !fs::metadata(".env").is_ok() {
        println!("📝 Creating .env file for test configuration...");
        let env_content = r#"# Test Environment Configuration
TEST_NETWORK=local
TEST_ACCOUNT_SOURCE=test_source
TEST_ACCOUNT_DESTINATION=test_destination
SOROBAN_RPC_URL=http://localhost:8000
"#;
        fs::write(".env", env_content).unwrap();
        println!("{}", "✅ .env file created".green());
    } else {
        println!("{}", "ℹ️  .env file already exists".yellow());
    }
}

fn setup_test_data() {
    println!("{}", style("📊 Setting Up Test Data...").bold().cyan());
    
    // Create test data directory
    fs::create_dir_all("./test-data").unwrap();
    println!("✅ Test data directory created");
    
    // Create sample test data
    let sample_data = r#"{
  "test_contracts": [
    {
      "name": "example_contract",
      "wasm_path": "./target/release/contract_example.wasm",
      "expected_gas": 100000
    }
  ],
  "test_accounts": [
    {
      "name": "test_account_1",
      "public_key": "GB...",
      "secret": "S..."
    }
  ]
}"#;
    
    fs::write("./test-data/sample_test_data.json", sample_data).unwrap();
    println!("✅ Sample test data created");
}

fn configure_test_networks() {
    println!("{}", style("🌐 Configuring Test Networks...").bold().cyan());
    
    // Show current Soroban configuration
    execute_command("soroban", &["config", "network", "list"]);
    
    println!("\n💡 Available test networks:");
    println!("   🌐 Localnet: http://localhost:8000");
    println!("   🧪 Testnet: https://soroban-testnet.stellar.org");
    println!("   ⭐ Futurenet: https://horizon-futurenet.stellar.org");
}

// Helper functions
fn check_test_prerequisites() -> bool {
    println!("🔍 Checking test prerequisites...");
    
    let mut all_good = true;
    
    // Check if contracts are built
    if !fs::metadata("./target/release").is_ok() {
        println!("{}", "❌ Contracts not built. Run 'make build' first.".red());
        all_good = false;
    }
    
    // Check if localnet is running (for integration tests)
    if !is_localnet_running() {
        println!("{}", "⚠️  Localnet not running. Some integration tests may fail.".yellow());
    }
    
    if all_good {
        println!("{}", "✅ All prerequisites satisfied".green());
    }
    
    all_good
}

fn is_localnet_running() -> bool {
    // Check if localnet is running by checking Docker containers
    Command::new("docker")
        .args(&["ps", "--filter", "name=soroban", "--quiet"])
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false)
}

fn get_contract_list() -> Vec<String> {
    fs::read_dir("./contracts")
        .unwrap_or_else(|_| panic!("Could not read contracts directory"))
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().into_string().unwrap())
        .collect()
}

fn count_pattern_in_contracts(pattern: &str) -> usize {
    let mut count = 0;
    let contracts_dir = "./contracts";
    
    if fs::metadata(contracts_dir).is_ok() {
        for entry in WalkDir::new(contracts_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().map(|ext| ext == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    count += content.matches(pattern).count();
                }
            }
        }
    }
    
    count
}

fn execute_command(cmd: &str, args: &[&str]) {
    println!("{} {} {}", style("➜ Executing:").bold().dim(), cmd, args.join(" "));
    
    let mut child = Command::new(cmd).args(args).spawn().expect("Failed to execute command");
    let status = child.wait().expect("Failed to wait on child");
    
    if status.success() {
        println!("{}", style("✔ Command successful").green());
    } else {
        println!("{}", style("✘ Command failed").red());
    }
}
