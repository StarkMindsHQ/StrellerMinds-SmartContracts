//! End-to-End Test Utilities for Soroban Contracts
//!
//! This crate provides utilities and test cases for testing the StrellerMinds
//! smart contracts in a realistic Soroban localnet environment.

use anyhow::Result;
use reqwest::Client;
use std::collections::HashMap;
use std::process::Command;
use std::time::Duration;

pub mod test_data;
pub mod test_utils;

/// Configuration for the E2E test environment
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub admin_account: String,
    pub test_accounts: Vec<String>,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8000".to_string(),
            network_passphrase: "Standalone Network ; February 2017".to_string(),
            admin_account: "admin".to_string(),
            test_accounts: vec![
                "alice".to_string(),
                "bob".to_string(),
                "charlie".to_string(),
            ],
        }
    }
}

/// Soroban RPC client for E2E testing
pub struct SorobanClient {
    client: Client,
    pub config: E2ETestConfig,
}

impl SorobanClient {
    pub fn new(config: E2ETestConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Check if the Soroban RPC endpoint is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/health", self.config.rpc_url))
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Deploy a contract using the Soroban CLI
    pub async fn deploy_contract(
        &self,
        contract_name: &str,
        deployer_account: &str,
    ) -> Result<String> {
        let wasm_path = format!(
            "target/wasm32-unknown-unknown/release/{contract_name}.wasm"
        );

        let output = Command::new("soroban")
            .args([
                "contract",
                "deploy",
                "--source",
                deployer_account,
                "--network",
                "local",
                "--wasm",
                &wasm_path,
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to deploy {}: {}",
                contract_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let contract_id = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(contract_id)
    }

    /// Invoke a contract method
    pub async fn invoke_contract(
        &self,
        contract_id: &str,
        method: &str,
        args: &[String],
        source_account: &str,
    ) -> Result<String> {
        let mut cmd_args = vec![
            "contract",
            "invoke",
            "--id",
            contract_id,
            "--source",
            source_account,
            "--network",
            "local",
            "--",
            method,
        ];

        for arg in args {
            cmd_args.push(arg);
        }

        let output = Command::new("soroban").args(cmd_args).output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to invoke {}::{}: {}",
                contract_id,
                method,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Get the address for a named account
    pub fn get_account_address(&self, account_name: &str) -> Result<String> {
        let output = Command::new("soroban")
            .args(["keys", "address", account_name])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get address for account {}: {}",
                account_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }
}

/// Test harness for managing E2E test lifecycle
pub struct E2ETestHarness {
    pub client: SorobanClient,
    pub deployed_contracts: HashMap<String, String>,
}

impl E2ETestHarness {
    pub async fn new() -> Result<Self> {
        let config = E2ETestConfig::default();
        let client = SorobanClient::new(config);

        // Ensure localnet is running
        if !client.health_check().await? {
            anyhow::bail!("Soroban localnet is not running. Please start it first with: ./scripts/start_localnet.sh");
        }

        Ok(Self {
            client,
            deployed_contracts: HashMap::new(),
        })
    }

    /// Setup the test environment by building and deploying contracts
    pub async fn setup(&mut self) -> Result<()> {
        // Build all contracts first
        self.build_contracts().await?;

        // Deploy core contracts
        let contracts_to_deploy = vec!["shared", "certificate", "analytics", "token"];

        for contract_name in contracts_to_deploy {
            let contract_id = self
                .client
                .deploy_contract(contract_name, &self.client.config.admin_account)
                .await?;

            self.deployed_contracts
                .insert(contract_name.to_string(), contract_id.clone());
            println!("✅ Deployed {contract_name}: {contract_id}");
        }

        // Initialize contracts
        self.initialize_contracts().await?;

        Ok(())
    }

    /// Build all contracts
    async fn build_contracts(&self) -> Result<()> {
        println!("🔨 Building contracts...");

        let output = Command::new("./scripts/build.sh").output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to build contracts: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        println!("✅ Contracts built successfully");
        Ok(())
    }

    /// Initialize deployed contracts
    async fn initialize_contracts(&self) -> Result<()> {
        println!("🚀 Initializing contracts...");

        let admin_address = self
            .client
            .get_account_address(&self.client.config.admin_account)?;

        // Initialize shared contract (if needed)
        if let Some(_shared_id) = self.deployed_contracts.get("shared") {
            // Most shared contract initialization happens automatically
            println!("✅ Shared contract initialized");
        }

        // Initialize certificate contract
        if let Some(cert_id) = self.deployed_contracts.get("certificate") {
            self.client
                .invoke_contract(
                    cert_id,
                    "initialize",
                    &[format!("--admin {admin_address}")],
                    &self.client.config.admin_account,
                )
                .await?;
            println!("✅ Certificate contract initialized");
        }

        // Initialize other contracts as needed
        // Analytics, Token contracts may need similar initialization

        Ok(())
    }

    /// Get contract ID by name
    pub fn get_contract_id(&self, contract_name: &str) -> Option<&String> {
        self.deployed_contracts.get(contract_name)
    }

    /// Clean up deployed contracts (optional)
    pub async fn cleanup(&self) -> Result<()> {
        // In a real scenario, we might want to clean up deployed contracts
        // For localnet testing, this is usually not necessary as the network resets
        println!("🧹 Test cleanup completed");
        Ok(())
    }
}

/// Helper macros for E2E tests
#[macro_export]
macro_rules! assert_contract_success {
    ($result:expr) => {
        match $result {
            Ok(output) => {
                assert!(
                    !output.trim().is_empty(),
                    "Contract call returned empty result"
                );
                output
            }
            Err(e) => panic!("Contract call failed: {}", e),
        }
    };
}

#[macro_export]
macro_rules! setup_test_harness {
    () => {{
        let mut harness = E2ETestHarness::new().await?;
        harness.setup().await?;
        harness
    }};
}
