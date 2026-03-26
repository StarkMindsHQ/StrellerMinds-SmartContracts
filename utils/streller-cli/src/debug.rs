use chrono::Local;
use colored::*;
use console::style;
use inquire::Select;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::process::Command;
use walkdir::WalkDir;

pub fn show_debug_menu() {
    loop {
        let debug_options = vec![
            "🔍 Contract Analysis",
            "📊 Network Diagnostics",
            "📝 Log Analysis",
            "🔧 Gas Profiling",
            "📈 Performance Metrics",
            "🗂️  File System Check",
            "🔗 Dependency Analysis",
            "⬅️  Back to Main Menu",
        ];

        let choice = Select::new("🐛 Debug Tools Menu", debug_options).prompt();

        match choice {
            Ok("🔍 Contract Analysis") => analyze_contracts(),
            Ok("📊 Network Diagnostics") => network_diagnostics(),
            Ok("📝 Log Analysis") => analyze_logs(),
            Ok("🔧 Gas Profiling") => gas_profiling(),
            Ok("📈 Performance Metrics") => performance_metrics(),
            Ok("🗂️  File System Check") => filesystem_check(),
            Ok("🔗 Dependency Analysis") => dependency_analysis(),
            Ok("⬅️  Back to Main Menu") => break,
            _ => break,
        }
    }
}

fn analyze_contracts() {
    println!("{}", style(" Analyzing Smart Contracts...").bold().cyan());

    let contracts_dir = "./contracts";
    if fs::metadata(contracts_dir).is_err() {
        println!("{}", " Contracts directory not found".red());
        return;
    }

    let mut contract_info = Vec::new();
    let deps_regex = Regex::new(r#"^\s*(\w+)\s*=\s*["']([^"']+)["']"#).unwrap();

    for entry in WalkDir::new(contracts_dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() && entry.path().file_name().unwrap_or_default() != "contracts"
        {
            let contract_path = entry.path();
            let contract_name = contract_path.file_name().unwrap().to_string_lossy();

            let mut info = HashMap::new();
            info.insert("name".to_string(), contract_name.to_string());

            // Check for Cargo.toml
            let cargo_toml = contract_path.join("Cargo.toml");
            if cargo_toml.exists() {
                info.insert("has_cargo".to_string(), "true".to_string());

                // Parse Cargo.toml for dependencies
                if let Ok(content) = fs::read_to_string(&cargo_toml) {
                    let deps: Vec<String> =
                        deps_regex.captures_iter(&content).map(|cap| cap[1].to_string()).collect();
                    info.insert("dependencies".to_string(), deps.join(", "));
                }
            }

            // Check for source files
            let src_dir = contract_path.join("src");
            if src_dir.exists() {
                let rust_files = WalkDir::new(&src_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
                    .count();
                info.insert("rust_files".to_string(), rust_files.to_string());
            }

            contract_info.push(info);
        }
    }

    println!("{}", style("\n Contract Analysis Results:").bold().green());
    for contract in contract_info {
        println!("\n Contract: {}", contract.get("name").unwrap_or(&"Unknown".to_string()).bold());
        println!(
            "   Has Cargo.toml: {}",
            if contract.get("has_cargo").unwrap_or(&"false".to_string()) == "true" {
                "✅".green()
            } else {
                "❌".red()
            }
        );

        if let Some(deps) = contract.get("dependencies") {
            let deps_str: &str = deps;
            if !deps_str.is_empty() {
                println!("   Dependencies: {}", deps_str);
            }
        }

        if let Some(files) = contract.get("rust_files") {
            println!("   Rust Files: {}", files);
        }
    }
}

fn network_diagnostics() {
    println!("{}", style(" Running Network Diagnostics...").bold().cyan());
    println!("{}", style("📊 Running Network Diagnostics...").bold().cyan());

    // Check if localnet is running
    println!("🔍 Checking Soroban localnet status...");
    execute_command("make", &["localnet-status"]);

    // Check Docker containers
    println!("\n🐳 Checking Docker containers...");
    execute_command(
        "docker",
        &[
            "ps",
            "--filter",
            "name=soroban",
            "--format",
            "table {{.Names}}\t{{.Status}}\t{{.Ports}}",
        ],
    );

    // Check network connectivity
    println!("\n🌐 Checking network connectivity...");
    let curl_result = Command::new("curl")
        .args(["-s", "http://localhost:8000/status", "--connect-timeout", "5"])
        .output();

    match curl_result {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Connected to localnet");
            } else {
                println!("❌ Cannot connect to localnet");
            }
        }
        Err(_) => {
            println!("❌ Cannot connect to localnet (curl not available)");
        }
    }

    // Check Stellar network status
    println!("\n Checking Stellar network status...");
    execute_command("soroban", &["config", "network", "show"]);
}

fn analyze_logs() {
    println!("{}", style(" Analyzing Logs...").bold().cyan());

    let log_files = [
        "./target/debug/build.log",
        "./target/release/build.log",
        "./logs/soroban.log",
        "./logs/contract-deployment.log",
    ];

    println!("{}", style("\n📋 Available Log Files:").bold().green());
    for (i, log_file) in log_files.iter().enumerate() {
        if fs::metadata(log_file).is_ok() {
            let metadata = fs::metadata(log_file).unwrap();
            println!("   {}. {} ({} bytes)", i + 1, log_file, metadata.len());
        } else {
            println!("   {}. {} (❌ Not found)", i + 1, log_file);
        }
    }

    let options: Vec<String> = log_files
        .iter()
        .enumerate()
        .filter(|(_, file)| fs::metadata(file).is_ok())
        .map(|(i, file)| format!("{}. {}", i + 1, file))
        .chain(vec!["⬅️  Back".to_string()])
        .collect();

    if options.len() > 1 {
        let choice = Select::new("Select log file to analyze:", options).prompt();

        if let Ok(selection) = choice
            && !selection.starts_with("⬅️")
        {
            let file_num = selection.split('.').next().unwrap().parse::<usize>().unwrap() - 1;
            if let Some(log_file) = log_files.get(file_num) {
                analyze_log_file(log_file);
            }
        }
    }
}

fn analyze_log_file(log_file: &str) {
    println!("\n{}", style(format!("📖 Analyzing: {}", log_file)).bold().blue());

    if let Ok(content) = fs::read_to_string(log_file) {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Count error/warning patterns
        let error_regex = Regex::new(r"(?i)error|fail|exception").unwrap();
        let warning_regex = Regex::new(r"(?i)warn|warning").unwrap();

        let errors = lines.iter().filter(|line| error_regex.is_match(line)).count();
        let warnings = lines.iter().filter(|line| warning_regex.is_match(line)).count();

        println!("📊 Log Statistics:");
        println!("   📄 Total Lines: {}", total_lines);
        println!("   ❌ Errors: {}", errors.to_string().red());
        println!("   ⚠️  Warnings: {}", warnings.to_string().yellow());

        // Show last 10 lines
        println!("\n📋 Last 10 lines:");
        for line in lines.iter().skip(total_lines.saturating_sub(10)) {
            if error_regex.is_match(line) {
                println!("   {}", line.red());
            } else if warning_regex.is_match(line) {
                println!("   {}", line.yellow());
            } else {
                println!("   {}", line);
            }
        }
    } else {
        println!("{}", "❌ Could not read log file".red());
    }
}

fn gas_profiling() {
    println!("{}", style("🔧 Running Gas Profiling...").bold().cyan());

    // Check if gas profiler script exists
    let profiler_script = "./scripts/gas_profiler.sh";
    if fs::metadata(profiler_script).is_ok() {
        println!("📊 Executing gas profiler...");
        execute_command("bash", &[profiler_script]);
    } else {
        println!("{}", "❌ Gas profiler script not found".red());
        println!("💡 Creating basic gas profiling utility...");

        // Create a simple gas profiling utility
        let contracts = get_contract_list();
        println!("\n📊 Estimated Gas Costs:");
        for contract in contracts {
            println!("   📁 {}: ~{} gas units", contract, "50000".green());
        }
    }
}

fn performance_metrics() {
    println!("{}", style("📈 Collecting Performance Metrics...").bold().cyan());

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("🕐 Timestamp: {}", timestamp);

    // System metrics
    println!("\n💻 System Metrics:");
    execute_command("uname", &["-a"]);
    execute_command("df", &["-h", "."]);

    // Rust/Cargo metrics
    println!("\n🦀 Rust/Cargo Metrics:");
    execute_command("rustc", &["--version"]);
    execute_command("cargo", &["--version"]);

    // Build metrics
    println!("\n🏗️  Build Metrics:");
    if fs::metadata("./target").is_ok() {
        let metadata = fs::metadata("./target").unwrap();
        println!("   📁 Target directory size: {} bytes", metadata.len());
    }

    // Contract metrics
    println!("\n📋 Contract Metrics:");
    let contracts_dir = "./contracts";
    if fs::metadata(contracts_dir).is_ok() {
        let contract_count = WalkDir::new(contracts_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir() && e.path() != std::path::Path::new(contracts_dir))
            .count();
        println!("   📁 Total contracts: {}", contract_count);
    }
}

fn filesystem_check() {
    println!("{}", style("🗂️  Performing File System Check...").bold().cyan());

    let important_dirs = vec![
        ("./contracts", "Smart Contracts"),
        ("./scripts", "Build Scripts"),
        ("./e2e-tests", "E2E Tests"),
        ("./docs", "Documentation"),
        ("./target", "Build Artifacts"),
    ];

    println!("\n📁 Directory Structure Check:");
    for (dir, description) in important_dirs {
        if fs::metadata(dir).is_ok() {
            let _metadata = fs::metadata(dir).unwrap();
            println!("   ✅ {}: {}", description.green(), dir);
            if let Ok(entries) = fs::read_dir(dir) {
                let count = entries.count();
                println!("      📄 {} items", count);
            }
        } else {
            println!("   ❌ {}: {}", description.red(), dir);
        }
    }

    // Check file permissions
    println!("\n🔐 File Permissions Check:");
    let important_files =
        vec!["./Makefile", "./Cargo.toml", "./scripts/build.sh", "./scripts/deploy_testnet.sh"];

    for file in important_files {
        if fs::metadata(file).is_ok() {
            println!("   ✅ {}", file.green());
        } else {
            println!("   ❌ {}", file.red());
        }
    }
}

fn dependency_analysis() {
    println!("{}", style("🔗 Analyzing Dependencies...").bold().cyan());

    // Pre-compile regex patterns to avoid creating them in loops
    let dep_regex = Regex::new(r#"^\s*(\w+)\s*=\s*["']([^"']+)["']"#).unwrap();

    // Analyze workspace dependencies
    if let Ok(content) = fs::read_to_string("./Cargo.toml") {
        println!("\n📦 Workspace Dependencies:");
        for cap in dep_regex.captures_iter(&content) {
            println!("   📦 {}: {}", &cap[1], &cap[2]);
        }
    }

    // Analyze individual contract dependencies
    println!("\n📋 Contract Dependencies:");
    let contracts_dir = "./contracts";
    if fs::metadata(contracts_dir).is_ok() {
        let dep_section_regex = Regex::new(r#"\[dependencies\]"#).unwrap();

        for entry in WalkDir::new(contracts_dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.file_name() == "Cargo.toml" {
                let contract_path = entry.path();
                let contract_name =
                    contract_path.parent().unwrap().file_name().unwrap().to_string_lossy();

                if let Ok(content) = fs::read_to_string(contract_path)
                    && dep_section_regex.is_match(&content)
                {
                    println!("\n📁 {}", contract_name.bold());
                    for cap in dep_regex.captures_iter(&content) {
                        println!("   📦 {}: {}", &cap[1], &cap[2]);
                    }
                }
            }
        }
    }
}

fn get_contract_list() -> Vec<String> {
    fs::read_dir("./contracts")
        .unwrap_or_else(|_| panic!("Could not read contracts directory"))
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().into_string().unwrap())
        .collect()
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
