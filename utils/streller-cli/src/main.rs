use console::style;
use inquire::{Confirm, Select};
use std::fs;
use std::process::Command;

mod debug;
mod testing;

use debug::*;
use testing::*;

fn main() {
    println!("{}", style("╔════════════════════════════════════════╗").cyan());
    println!("{}", style("║    StrellerMinds Smart Contract CLI    ║").bold().cyan());
    println!("{}", style("╚════════════════════════════════════════╝").cyan());

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
            "🐛 Debug: Debug Tools",
            "🧪 Test Utils: Testing Utilities",
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
            Ok("🐛 Debug: Debug Tools") => show_debug_menu(),
            Ok("🧪 Test Utils: Testing Utilities") => show_testing_menu(),
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
    // Prefixing with underscore (_selection) silences the 'unused variable' warning
    let _selection =
        Select::new("Which contract are you focusing on?", contracts.to_vec()).prompt().unwrap();

    println!("{}", style("Note: The project Makefile deploys ALL contracts to the network.").dim());
    let confirm = Confirm::new("Run 'make deploy-testnet' now?").with_default(false).prompt();

    if let Ok(true) = confirm {
        execute_command("make", &["deploy-testnet"]);
    }
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
