#!/usr/bin/env bash
set -e

SOROBAN_CLI_VERSION="22.0.0"

echo "Installing Soroban CLI v${SOROBAN_CLI_VERSION}..."
cargo install --locked soroban-cli --version "${SOROBAN_CLI_VERSION}"

echo "Verifying version..."
soroban --version
