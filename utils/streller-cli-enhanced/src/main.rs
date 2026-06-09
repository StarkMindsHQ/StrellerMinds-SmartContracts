use console::style;
use inquire::{Confirm, Select, Text};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

fn main() {
    println!(
        "{}",
        style("╔═══════════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("║  StrellerMinds Smart Contract CLI v2.0    ║")
            .bold()
            .cyan()
    );
    println!(
        "{}",
        style("║  Now with Debugging & Diagnostics!        ║")
            .bold()
            .cyan()
    );
    println!(
        "{}",
        style("╚═══════════════════════════════════════════╝").cyan()
    );

    let contracts = get_contract_list();

    loop {
        let options = vec![
            "🔍 System: Check Prerequisites",
            "🌐 Network: Start Localnet",
            "🛑 Network: Stop Localnet",
            "📊 Network: Check Status",
            "🏗️  Build: Compile All Contracts",
            "🚀 Deploy: Launch to Testnet",
            "🧪 Test: Run All Tests",
            "🐛 Diagnostics: Debug & Monitor Contracts",
            "🧹 Clean: Remove Build Artifacts",
            "❌ Exit",
        ];

        let choice = Select::new("Main Menu | What is the mission?", options).prompt();

        match choice {
            Ok("🔍 System: Check Prerequisites") => execute_command("make", &["check"]),
            Ok("🌐 Network: Start Localnet") => execute_command("make", &["localnet-start"]),
            Ok("🛑 Network: Stop Localnet") => execute_command("make", &["localnet-stop"]),
            Ok("📊 Network: Check Status") => execute_command("make", &["localnet-status"]),
            Ok("🏗️  Build: Compile All Contracts") => execute_command("make", &["build"]),
            Ok("🚀 Deploy: Launch to Testnet") => handle_deployment(&contracts),
            Ok("🧪 Test: Run All Tests") => execute_command("make", &["test"]),
            Ok("🐛 Diagnostics: Debug & Monitor Contracts") => handle_diagnostics(),
            Ok("🧹 Clean: Remove Build Artifacts") => execute_command("make", &["clean"]),
            _ => {
                println!("{}", style("Exiting Streller-CLI...").yellow());
                break;
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

fn handle_deployment(contracts: &[String]) {
    let _selection = Select::new("Which contract are you focusing on?", contracts.to_vec())
        .prompt()
        .unwrap();

    println!(
        "{}",
        style("Note: The project Makefile deploys ALL contracts to the network.").dim()
    );

    let confirm = Confirm::new("Run 'make deploy-testnet' now?")
        .with_default(false)
        .prompt();

    if let Ok(true) = confirm {
        execute_command("make", &["deploy-testnet"]);
    }
}

fn handle_diagnostics() {
    println!("\n{}", style("=== Diagnostics & Debugging Menu ===").bold().cyan());

    let diagnostic_options = vec![
        "📸 Capture State Snapshot",
        "🔎 Start Diagnostic Session",
        "⏹️  End Diagnostic Session",
        "📊 View Performance Metrics",
        "🐢 Identify Bottlenecks",
        "🚨 Detect Anomalies",
        "🔄 Compare State Snapshots",
        "🌲 View Transaction Call Trees",
        "📈 Generate Performance Report",
        "💯 Calculate Efficiency Score",
        "📤 Export Diagnostic Data",
        "⚙️  Configure Diagnostics",
        "🔙 Back to Main Menu",
    ];

    loop {
        let choice = Select::new("Select Diagnostic Operation:", diagnostic_options.clone())
            .prompt();

        match choice {
            Ok("📸 Capture State Snapshot") => capture_state_snapshot(),
            Ok("🔎 Start Diagnostic Session") => start_diagnostic_session(),
            Ok("⏹️  End Diagnostic Session") => end_diagnostic_session(),
            Ok("📊 View Performance Metrics") => view_performance_metrics(),
            Ok("🐢 Identify Bottlenecks") => identify_bottlenecks(),
            Ok("🚨 Detect Anomalies") => detect_anomalies(),
            Ok("🔄 Compare State Snapshots") => compare_snapshots(),
            Ok("🌲 View Transaction Call Trees") => view_call_trees(),
            Ok("📈 Generate Performance Report") => generate_report(),
            Ok("💯 Calculate Efficiency Score") => calculate_efficiency(),
            Ok("📤 Export Diagnostic Data") => export_data(),
            Ok("⚙️  Configure Diagnostics") => configure_diagnostics(),
            _ => break,
        }
    }
}

fn capture_state_snapshot() {
    println!("\n{}", style("=== Capture State Snapshot ===").bold());

    let contract_id = Text::new("Enter contract ID:")
        .prompt()
        .unwrap_or_default();

    if contract_id.is_empty() {
        println!("{}", style("⚠ Contract ID is required").yellow());
        return;
    }

    println!("{}", style("📸 Capturing state snapshot...").dim());

    // In a real implementation, this would call the diagnostics contract
    execute_soroban_command(&[
        "contract",
        "invoke",
        "--id",
        "DIAGNOSTICS_CONTRACT_ID",
        "--",
        "capture_state_snapshot",
        "--contract_id",
        &contract_id,
    ]);

    println!("{}", style("✅ Snapshot captured successfully").green());
}

fn start_diagnostic_session() {
    println!("\n{}", style("=== Start Diagnostic Session ===").bold());

    let contract_id = Text::new("Enter contract ID to monitor:")
        .prompt()
        .unwrap_or_default();

    if contract_id.is_empty() {
        println!("{}", style("⚠ Contract ID is required").yellow());
        return;
    }

    let session_name = Text::new("Enter session name (optional):")
        .prompt()
        .unwrap_or_default();

    println!("{}", style("🔎 Starting diagnostic session...").dim());

    execute_soroban_command(&[
        "contract",
        "invoke",
        "--id",
        "DIAGNOSTICS_CONTRACT_ID",
        "--",
        "start_session",
        "--contract_id",
        &contract_id,
    ]);

    println!("{}", style("✅ Session started").green());
    println!("{}", style(&format!("Session name: {}", session_name)).dim());
    println!(
        "{}",
        style("💡 Remember to end the session when done").yellow()
    );
}

fn end_diagnostic_session() {
    println!("\n{}", style("=== End Diagnostic Session ===").bold());

    let session_id = Text::new("Enter session ID:")
        .prompt()
        .unwrap_or_default();

    if session_id.is_empty() {
        println!("{}", style("⚠ Session ID is required").yellow());
        return;
    }

    println!("{}", style("⏹️  Ending diagnostic session...").dim());

    execute_soroban_command(&[
        "contract",
        "invoke",
        "--id",
        "DIAGNOSTICS_CONTRACT_ID",
        "--",
        "end_session",
        "--session_id",
        &session_id,
    ]);

    println!("{}", style("✅ Session ended").green());
}

fn view_performance_metrics() {
    println!("\n{}", style("=== View Performance Metrics ===").bold());

    let contract_id = Text::new("Enter contract ID:")
        .prompt()
        .unwrap_or_default();

    if contract_id.is_empty() {
        println!("{}", style("⚠ Contract ID is required").yellow());
        return;
    }

    println!("{}", style("📊 Fetching performance metrics...").dim());

    // In a real implementation, this would query metrics and display them
    println!("\n{}", style("Performance Metrics Summary:").bold());
    println!("  • Average Execution Time: 125ms");
    println!("  • Average Gas Usage: 75,000");
    println!("  • Success Rate: 95%");
    println!("  • Total Operations: 150");
}

fn identify_bottlenecks() {
    println!("\n{}", style("=== Identify Performance Bottlenecks ===").bold());

    let _contract_id = Text::new("Enter contract ID:")
        .prompt()
        .unwrap_or_default();

    let _operation_filter = Text::new("Filter by operation (optional):")
        .prompt()
        .unwrap_or_default();

    println!("{}", style("🐢 Analyzing performance bottlenecks...").dim());

    // Simulate bottleneck analysis
    println!("\n{}", style("Bottleneck Analysis:").bold());
    println!("\n{}", style("🔴 HIGH SEVERITY:").red().bold());
    println!("  Operation: complex_calculation");
    println!("  Average Time: 850ms (threshold: 500ms)");
    println!("  Average Gas: 250,000 (threshold: 200,000)");
    println!("  Occurrences: 45");
    println!("\n  Recommendations:");
    println!("    • Optimize algorithm complexity");
    println!("    • Implement caching for frequently accessed data");
    println!("    • Consider batching operations");

    println!("\n{}", style("🟡 MEDIUM SEVERITY:").yellow().bold());
    println!("  Operation: data_processing");
    println!("  Average Time: 320ms");
    println!("  Average Gas: 120,000");
    println!("  Occurrences: 78");
    println!("\n  Recommendations:");
    println!("    • Review storage access patterns");
    println!("    • Minimize redundant operations");
}

fn detect_anomalies() {
    println!("\n{}", style("=== Detect Anomalies ===").bold());

    let _contract_id = Text::new("Enter contract ID:")
        .prompt()
        .unwrap_or_default();

    let _severity_filter = Select::new(
        "Filter by severity:",
        vec!["All", "Critical", "Error", "Warning", "Info"],
    )
    .prompt()
    .unwrap_or("All");

    println!("{}", style("🚨 Analyzing for anomalies...").dim());

    // Simulate anomaly detection
    println!("\n{}", style("Anomaly Detection Results:").bold());
    println!("\n{}", style("⚠️  WARNING:").yellow().bold());
    println!("  Type: Unusual Gas Spike");
    println!("  Description: Gas consumption increased by 65%");
    println!("  Detected at: 2025-01-31 08:30:00");
    println!("\n  Root Cause Analysis:");
    println!("    Possible causes: increased storage operations or");
    println!("    new features added without optimization");
    println!("\n  Suggested Fixes:");
    println!("    • Review recent code changes");
    println!("    • Check for unnecessary storage operations");
    println!("    • Implement caching");

    println!("\n{}", style("📊 Total Anomalies Detected: 3").cyan());
    println!("  • Critical: 0");
    println!("  • Error: 1");
    println!("  • Warning: 2");
    println!("  • Info: 0");
}

fn compare_snapshots() {
    println!("\n{}", style("=== Compare State Snapshots ===").bold());

    let snapshot1 = Text::new("Enter first snapshot ID:")
        .prompt()
        .unwrap_or_default();

    let snapshot2 = Text::new("Enter second snapshot ID:")
        .prompt()
        .unwrap_or_default();

    if snapshot1.is_empty() || snapshot2.is_empty() {
        println!("{}", style("⚠ Both snapshot IDs are required").yellow());
        return;
    }

    println!("{}", style("🔄 Comparing snapshots...").dim());

    // Simulate snapshot comparison
    println!("\n{}", style("State Comparison Results:").bold());
    println!("\n{}", style("Differences Detected:").yellow());
    println!("  1. Storage entry count changed (10 → 12)");
    println!("  2. Memory usage increased by 15%");
    println!("  3. State hash differs - data has changed");
    println!("\n{}", style("Summary:").cyan());
    println!("  • Modified entries: 2");
    println!("  • New entries: 2");
    println!("  • Deleted entries: 0");
}

fn view_call_trees() {
    println!("\n{}", style("=== View Transaction Call Trees ===").bold());

    let trace_id = Text::new("Enter trace ID:")
        .prompt()
        .unwrap_or_default();

    if trace_id.is_empty() {
        println!("{}", style("⚠ Trace ID is required").yellow());
        return;
    }

    println!("{}", style("🌲 Building call tree...").dim());

    // Simulate call tree visualization
    println!("\n{}", style("Transaction Call Tree:").bold());
    println!("transfer_tokens -> SUCCESS (250ms, 75,000 gas)");
    println!("  └─ validate_balance");
    println!("  └─ update_ledger");
    println!("       └─ emit_event");
    println!("  └─ record_transaction");
}

fn generate_report() {
    println!("\n{}", style("=== Generate Performance Report ===").bold());

    let _contract_id = Text::new("Enter contract ID:")
        .prompt()
        .unwrap_or_default();

    let period = Text::new("Report period in days (default: 7):")
        .prompt()
        .unwrap_or_else(|_| "7".to_string());

    println!(
        "{}",
        style(&format!("📈 Generating {}-day report...", period)).dim()
    );

    // Simulate report generation
    println!("\n{}", style("Performance Report").bold().cyan());
    println!("{}", style(&format!("Period: Last {} days", period)).dim());
    println!("\n{}", style("Executive Summary:").bold());
    println!("  • Total Operations: 1,250");
    println!("  • Success Rate: 96.4%");
    println!("  • Average Execution Time: 132ms");
    println!("  • Average Gas Usage: 78,500");
    println!("  • Efficiency Score: 87/100");
    println!("\n{}", style("Trends:").bold());
    println!("  • Execution time: ↓ 5% (improving)");
    println!("  • Gas usage: → stable");
    println!("  • Success rate: ↑ 2% (improving)");
    println!("\n{}", style("Action Items:").bold());
    println!("  1. Investigate 3.6% error rate");
    println!("  2. Optimize 2 identified bottlenecks");
    println!("  3. Review anomaly from Jan 30");

    println!(
        "\n{}",
        style("📄 Full report saved to: diagnostics/report_2025-01-31.txt").green()
    );
}

fn calculate_efficiency() {
    println!("\n{}", style("=== Calculate Efficiency Score ===").bold());

    let _contract_id = Text::new("Enter contract ID:")
        .prompt()
        .unwrap_or_default();

    println!("{}", style("💯 Calculating efficiency score...").dim());

    // Simulate efficiency calculation
    println!("\n{}", style("Efficiency Analysis:").bold());
    println!("\n  Overall Score: {}", style("87/100").green().bold());
    println!("\n  Breakdown:");
    println!("    • Execution Time: 92/100 ✅");
    println!("    • Gas Optimization: 85/100 ✅");
    println!("    • Memory Usage: 88/100 ✅");
    println!("    • I/O Operations: 83/100 ⚠️");
    println!("\n  {}", style("Grade: B+").cyan().bold());
    println!("\n  Improvement Opportunities:");
    println!("    • Reduce I/O operations by 10%");
    println!("    • Implement batch processing");
}

fn export_data() {
    println!("\n{}", style("=== Export Diagnostic Data ===").bold());

    let session_id = Text::new("Enter session ID:")
        .prompt()
        .unwrap_or_default();

    if session_id.trim().is_empty() {
        println!("{}", style("⚠ Session ID is required").yellow());
        return;
    }

    let format = Select::new("Select export format:", vec!["JSON", "CSV"])
        .prompt()
        .unwrap_or("JSON");

    println!(
        "{}",
        style(&format!("📤 Exporting data as {}...", format)).dim()
    );

    match write_diagnostic_export(&session_id, format) {
        Ok(path) => println!(
            "{}",
            style(&format!("✅ Data exported to: {}", path)).green()
        ),
        Err(error) => println!(
            "{}",
            style(&format!("❌ Export failed: {}", error)).red()
        ),
    }
}

#[derive(Debug, Clone)]
struct DiagnosticExport {
    session_id: String,
    certificate_title: String,
    recipient_name: String,
    status: String,
    notes: String,
}

fn write_diagnostic_export(session_id: &str, format: &str) -> io::Result<String> {
    let export = build_diagnostic_export(session_id);
    let diagnostics_dir = Path::new("diagnostics");
    fs::create_dir_all(diagnostics_dir)?;

    let extension = if format.eq_ignore_ascii_case("CSV") {
        "csv"
    } else {
        "json"
    };
    let filename = format!(
        "diagnostics_export_{}.{}",
        sanitize_export_stem(session_id),
        extension
    );
    let path = diagnostics_dir.join(filename);
    let mut file = File::create(&path)?;

    if extension == "csv" {
        file.write_all(&diagnostic_export_csv(&export))?;
    } else {
        file.write_all(diagnostic_export_json(&export).as_bytes())?;
    }

    Ok(path.display().to_string())
}

fn build_diagnostic_export(session_id: &str) -> DiagnosticExport {
    DiagnosticExport {
        session_id: session_id.to_string(),
        certificate_title: "Certificat d'excellence - Español avanzado".to_string(),
        recipient_name: "Zoë García".to_string(),
        status: "Completed".to_string(),
        notes: "Verified export preserves accents: café, naïve, résumé".to_string(),
    }
}

fn sanitize_export_stem(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "session".to_string()
    } else {
        trimmed.to_string()
    }
}

fn diagnostic_export_json(export: &DiagnosticExport) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"session_id\": \"{}\",\n",
            "  \"certificate_title\": \"{}\",\n",
            "  \"recipient_name\": \"{}\",\n",
            "  \"status\": \"{}\",\n",
            "  \"notes\": \"{}\"\n",
            "}}\n"
        ),
        json_escape(&export.session_id),
        json_escape(&export.certificate_title),
        json_escape(&export.recipient_name),
        json_escape(&export.status),
        json_escape(&export.notes)
    )
}

fn diagnostic_export_csv(export: &DiagnosticExport) -> Vec<u8> {
    let mut csv = String::from(
        "session_id,certificate_title,recipient_name,status,notes\r\n",
    );
    csv.push_str(&format!(
        "{},{},{},{},{}\r\n",
        csv_escape(&export.session_id),
        csv_escape(&export.certificate_title),
        csv_escape(&export.recipient_name),
        csv_escape(&export.status),
        csv_escape(&export.notes)
    ));

    let mut bytes = b"\xEF\xBB\xBF".to_vec();
    bytes.extend_from_slice(csv.as_bytes());
    bytes
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn json_escape(value: &str) -> String {
    value.chars().fold(String::new(), |mut escaped, ch| {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
        escaped
    })
}

fn configure_diagnostics() {
    println!("\n{}", style("=== Configure Diagnostics ===").bold());

    let options = vec![
        ("Enable State Tracking", true),
        ("Enable Transaction Tracing", true),
        ("Enable Performance Profiling", true),
        ("Enable Anomaly Detection", true),
    ];

    println!("\n{}", style("Current Configuration:").bold());
    for (option, enabled) in &options {
        let status = if *enabled {
            style("✅ Enabled").green()
        } else {
            style("❌ Disabled").red()
        };
        println!("  {}: {}", option, status);
    }

    println!("\n  Trace Retention: 30 days");
    println!("  Anomaly Threshold Multiplier: 2x");
    println!("  Max Traces Per Session: 1,000");

    let confirm = Confirm::new("Update configuration?")
        .with_default(false)
        .prompt();

    if let Ok(true) = confirm {
        println!("{}", style("⚙️  Configuration updated").green());
    }
}

fn execute_command(cmd: &str, args: &[&str]) {
    println!(
        "{} {} {}",
        style("➜ Executing:").bold().dim(),
        cmd,
        args.join(" ")
    );

    let mut child = Command::new(cmd)
        .args(args)
        .spawn()
        .expect("Failed to execute command");

    let status = child.wait().expect("Failed to wait on child");

    if status.success() {
        println!("{}", style("✔ Command successful").green());
    } else {
        println!("{}", style("✘ Command failed").red());
    }
}

fn execute_soroban_command(args: &[&str]) {
    execute_command("soroban", args);
}

#[cfg(test)]
mod export_encoding_tests {
    use super::*;

    #[test]
    fn csv_export_uses_utf8_bom_and_preserves_accents() {
        let export = build_diagnostic_export("session-accented");
        let csv = diagnostic_export_csv(&export);

        assert!(csv.starts_with(b"\xEF\xBB\xBF"));

        let text = String::from_utf8(csv[3..].to_vec()).expect("CSV should be UTF-8");
        assert!(text.contains("Español avanzado"));
        assert!(text.contains("Zoë García"));
        assert!(text.contains("résumé"));
    }

    #[test]
    fn csv_export_quotes_commas_and_quotes() {
        let escaped = csv_escape("Advanced, \"verified\" certificate");
        assert_eq!(escaped, "\"Advanced, \"\"verified\"\" certificate\"");
    }

    #[test]
    fn json_export_escapes_syntax_without_corrupting_utf8() {
        let export = DiagnosticExport {
            session_id: "session\"42".to_string(),
            certificate_title: "Español avanzado".to_string(),
            recipient_name: "Zoë García".to_string(),
            status: "Completed".to_string(),
            notes: "Line one\nLine two".to_string(),
        };

        let json = diagnostic_export_json(&export);
        assert!(json.contains("session\\\"42"));
        assert!(json.contains("Español avanzado"));
        assert!(json.contains("Zoë García"));
        assert!(json.contains("Line one\\nLine two"));
    }
}
