#!/usr/bin/env python3
"""
Script to update all contract Cargo.toml files to inherit workspace linting configuration
"""

import os
import toml
from pathlib import Path

def update_cargo_toml(file_path: Path) -> bool:
    """Update a single Cargo.toml file to inherit workspace linting"""
    try:
        with open(file_path, 'r') as f:
            data = toml.load(f)
        
        # Check if lints section already exists
        if 'lints' in data:
            if data.get('lints', {}).get('workspace') is True:
                print(f"✓ {file_path.name} already inherits workspace linting")
                return False
            else:
                print(f"! {file_path.name} has custom lints, skipping")
                return False
        
        # Add lints section
        data['lints'] = {'workspace': True}
        
        with open(file_path, 'w') as f:
            toml.dump(data, f)
        
        print(f"✓ Updated {file_path.name}")
        return True
        
    except Exception as e:
        print(f"✗ Error updating {file_path}: {e}")
        return False

def main():
    """Update all contract Cargo.toml files"""
    contracts_dir = Path("contracts")
    
    if not contracts_dir.exists():
        print("Contracts directory not found")
        return
    
    updated_count = 0
    total_count = 0
    
    for cargo_toml in contracts_dir.glob("*/Cargo.toml"):
        total_count += 1
        if update_cargo_toml(cargo_toml):
            updated_count += 1
    
    print(f"\nSummary: Updated {updated_count}/{total_count} Cargo.toml files")

if __name__ == "__main__":
    main()
