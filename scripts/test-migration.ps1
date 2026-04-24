# StrellerMinds Smart Contracts - Migration Test Script (PowerShell)
# Tests the migration process on Windows

param(
    [string]$Network = "testnet",
    [switch]$DryRun = $false,
    [switch]$SkipBuild = $false
)

# Colors for output
$Red = "Red"
$Green = "Green"
$Yellow = "Yellow"
$Blue = "Blue"

function Write-Log {
    param([string]$Message)
    Write-Host "[$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')] $Message" -ForegroundColor $Blue
}

function Write-Error-Log {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Red
}

function Write-Success-Log {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Green
}

function Write-Warning-Log {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Yellow
}

function Test-Prerequisites {
    Write-Log "Testing prerequisites..."
    
    # Check if Rust is installed
    try {
        $rustVersion = rustc --version 2>$null
        if ($rustVersion) {
            Write-Success-Log "Rust is installed: $rustVersion"
        } else {
            Write-Error-Log "Rust is not installed"
            return $false
        }
    } catch {
        Write-Error-Log "Rust is not installed"
        return $false
    }
    
    # Check if Soroban CLI is installed
    try {
        $sorobanVersion = soroban --version 2>$null
        if ($sorobanVersion) {
            Write-Success-Log "Soroban CLI is installed: $sorobanVersion"
        } else {
            Write-Error-Log "Soroban CLI is not installed"
            return $false
        }
    } catch {
        Write-Error-Log "Soroban CLI is not installed"
        return $false
    }
    
    # Check if Stellar CLI is installed
    try {
        $stellarVersion = stellar --version 2>$null
        if ($stellarVersion) {
            Write-Success-Log "Stellar CLI is installed: $stellarVersion"
        } else {
            Write-Error-Log "Stellar CLI is not installed"
            return $false
        }
    } catch {
        Write-Error-Log "Stellar CLI is not installed"
        return $false
    }
    
    return $true
}

function Build-Contracts {
    if ($SkipBuild) {
        Write-Log "Skipping build as requested"
        return $true
    }
    
    Write-Log "Building contracts..."
    
    try {
        # Build all contracts
        $buildResult = cargo build --release --target wasm32-unknown-unknown 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success-Log "Contracts built successfully"
            return $true
        } else {
            Write-Error-Log "Build failed: $buildResult"
            return $false
        }
    } catch {
        Write-Error-Log "Build failed with exception: $_"
        return $false
    }
}

function Test-Contract-Functionality {
    Write-Log "Testing contract functionality..."
    
    $contracts = @("analytics", "token", "shared")
    $testResults = @()
    
    foreach ($contract in $contracts) {
        Write-Log "Testing $contract contract..."
        
        # Check if WASM file exists
        $wasmPath = "target/wasm32-unknown-unknown/release/$contract.wasm"
        if (Test-Path $wasmPath) {
            Write-Success-Log "$contract WASM file exists"
            $testResults += @{ Contract = $contract; Status = "PASS"; Details = "WASM file found" }
        } else {
            Write-Error-Log "$contract WASM file not found at $wasmPath"
            $testResults += @{ Contract = $contract; Status = "FAIL"; Details = "WASM file missing" }
        }
        
        # Test contract compilation
        try {
            $testResult = cargo test --package $contract 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success-Log "$contract tests passed"
                # Update the last result
                $testResults[-1].Details += ", Tests passed"
            } else {
                Write-Warning-Log "$contract tests failed: $testResult"
                if ($testResults[-1].Status -eq "PASS") {
                    $testResults[-1].Status = "WARN"
                    $testResults[-1].Details += ", Tests failed"
                }
            }
        } catch {
            Write-Warning-Log "$contract test exception: $_"
        }
    }
    
    return $testResults
}

function Test-Migration-Scripts {
    Write-Log "Testing migration scripts..."
    
    $scriptTests = @()
    
    $scripts = @(
        @{ Name = "migrate-data.sh"; Path = "scripts/migrate-data.sh" },
        @{ Name = "verify-migration.sh"; Path = "scripts/verify-migration.sh" },
        @{ Name = "rollback-to-v1.sh"; Path = "scripts/rollback-to-v1.sh" },
        @{ Name = "export-contract-state.sh"; Path = "scripts/export-contract-state.sh" },
        @{ Name = "verify-backup.sh"; Path = "scripts/verify-backup.sh" }
    )
    
    foreach ($script in $scripts) {
        Write-Log "Testing $($script.Name)..."
        
        if (Test-Path $script.Path) {
            Write-Success-Log "$($script.Name) exists"
            
            # Test script syntax by checking for required functions
            $scriptContent = Get-Content $script.Path -Raw
            $requiredFunctions = @("show_help", "main")
            $missingFunctions = @()
            
            foreach ($func in $requiredFunctions) {
                if ($scriptContent -notmatch "function $func\(\)" -and $scriptContent -notmatch "$func\(\)") {
                    $missingFunctions += $func
                }
            }
            
            if ($missingFunctions.Count -eq 0) {
                Write-Success-Log "$($script.Name) has required functions"
                $scriptTests += @{ Script = $script.Name; Status = "PASS"; Details = "All required functions present" }
            } else {
                Write-Warning-Log "$($script.Name) missing functions: $($missingFunctions -join ', ')"
                $scriptTests += @{ Script = $script.Name; Status = "WARN"; Details = "Missing functions: $($missingFunctions -join ', ')" }
            }
        } else {
            Write-Error-Log "$($script.Name) not found"
            $scriptTests += @{ Script = $script.Name; Status = "FAIL"; Details = "Script file missing" }
        }
    }
    
    return $scriptTests
}

function Test-Documentation {
    Write-Log "Testing migration documentation..."
    
    $docPath = "docs/MIGRATION_GUIDE_V1_TO_V2.md"
    
    if (Test-Path $docPath) {
        Write-Success-Log "Migration guide exists"
        
        # Check for required sections
        $docContent = Get-Content $docPath -Raw
        $requiredSections = @(
            "Breaking Changes",
            "Data Migration Steps", 
            "API Changes",
            "Configuration Updates",
            "Rollback Procedures",
            "Migration Checklist",
            "Troubleshooting"
        )
        
        $missingSections = @()
        foreach ($section in $requiredSections) {
            if ($docContent -notmatch "## $section") {
                $missingSections += $section
            }
        }
        
        if ($missingSections.Count -eq 0) {
            Write-Success-Log "All required sections present in migration guide"
            return @{ Status = "PASS"; Details = "Complete documentation" }
        } else {
            Write-Warning-Log "Missing sections: $($missingSections -join ', ')"
            return @{ Status = "WARN"; Details = "Missing sections: $($missingSections -join ', ')" }
        }
    } else {
        Write-Error-Log "Migration guide not found"
        return @{ Status = "FAIL"; Details = "Documentation missing" }
    }
}

function New-Test-Report {
    param(
        [array]$ContractTests,
        [array]$ScriptTests,
        [hashtable]$DocTest
    )
    
    $reportPath = "migration_test_report_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
    
    $report = @{
        test = @{
            timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
            network = $Network
            dry_run = $DryRun
            skip_build = $SkipBuild
        }
        results = @{
            contracts = $ContractTests
            scripts = $ScriptTests
            documentation = $DocTest
        }
        summary = @{
            total_tests = $ContractTests.Count + $ScriptTests.Count + 1
            passed_tests = ($ContractTests.Where({$_.Status -eq "PASS"}).Count) + ($ScriptTests.Where({$_.Status -eq "PASS"}).Count) + $(if ($DocTest.Status -eq "PASS") { 1 } else { 0 })
            success_rate = 0
        }
    }
    
    # Calculate success rate
    if ($report.summary.total_tests -gt 0) {
        $report.summary.success_rate = [math]::Round($report.summary.passed_tests * 100 / $report.summary.total_tests, 2)
    }
    
    # Save report
    $report | ConvertTo-Json -Depth 10 | Out-File -FilePath $reportPath
    
    Write-Success-Log "Test report saved to: $reportPath"
    
    # Display summary
    Write-Log "Test Summary:"
    Write-Log "Total Tests: $($report.summary.total_tests)"
    Write-Log "Passed Tests: $($report.summary.passed_tests)"
    Write-Log "Success Rate: $($report.summary.success_rate)%"
    
    return $report
}

# Main test function
function Main {
    Write-Log "Starting StrellerMinds migration test"
    Write-Log "Network: $Network"
    Write-Log "Dry Run: $DryRun"
    Write-Log "Skip Build: $SkipBuild"
    
    # Test prerequisites
    if (-not (Test-Prerequisites)) {
        Write-Error-Log "Prerequisites test failed"
        exit 1
    }
    
    # Build contracts
    if (-not (Build-Contracts)) {
        Write-Error-Log "Contract build failed"
        exit 1
    }
    
    # Test contract functionality
    $contractTests = Test-Contract-Functionality
    
    # Test migration scripts
    $scriptTests = Test-Migration-Scripts
    
    # Test documentation
    $docTest = Test-Documentation
    
    # Generate report
    $report = New-Test-Report -ContractTests $contractTests -ScriptTests $scriptTests -DocTest $docTest
    
    if ($report.summary.success_rate -ge 80) {
        Write-Success-Log "Migration test completed successfully!"
    } else {
        Write-Warning-Log "Migration test completed with issues (success rate: $($report.summary.success_rate)%)"
    }
    
    exit 0
}

# Execute main function
Main
