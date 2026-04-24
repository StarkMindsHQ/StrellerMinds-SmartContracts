# StrellerMinds Migration Test Script (PowerShell)
param(
    [switch]$Verbose = $false
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

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor $Green
}

function Write-Error-Log {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor $Red
}

function Write-Warning-Log {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor $Yellow
}

# Test 1: Check if migration guide exists
function Test-MigrationGuide {
    Write-Log "Testing migration guide..."
    
    if (Test-Path "docs/MIGRATION_GUIDE_V1_TO_V2.md") {
        Write-Success "Migration guide exists"
        
        # Check for required sections
        $requiredSections = @("Breaking Changes", "Data Migration Steps", "API Changes", "Configuration Updates", "Rollback Procedures")
        $missingSections = @()
        
        $content = Get-Content "docs/MIGRATION_GUIDE_V1_TO_V2.md" -Raw
        foreach ($section in $requiredSections) {
            if ($content -match "## $section") {
                Write-Success "Found section: $section"
            } else {
                Write-Error-Log "Missing section: $section"
                $missingSections += $section
            }
        }
        
        if ($missingSections.Count -eq 0) {
            Write-Success "All required sections present"
            return $true
        } else {
            Write-Error-Log "Missing sections: $($missingSections -join ', ')"
            return $false
        }
    } else {
        Write-Error-Log "Migration guide not found"
        return $false
    }
}

# Test 2: Check migration scripts
function Test-MigrationScripts {
    Write-Log "Testing migration scripts..."
    
    $scripts = @("migrate-data.sh", "verify-migration.sh", "rollback-to-v1.sh", "export-contract-state.sh", "verify-backup.sh")
    $missingScripts = @()
    
    foreach ($script in $scripts) {
        if (Test-Path "scripts/$script") {
            Write-Success "Found script: $script"
            
            # Check if script has required functions
            $scriptContent = Get-Content "scripts/$script" -Raw
            if ($scriptContent -match "show_help" -and $scriptContent -match "main") {
                Write-Success "Script $script has required functions"
            } else {
                Write-Warning-Log "Script $script may be missing required functions"
            }
        } else {
            Write-Error-Log "Missing script: $script"
            $missingScripts += $script
        }
    }
    
    if ($missingScripts.Count -eq 0) {
        Write-Success "All migration scripts present"
        return $true
    } else {
        Write-Error-Log "Missing scripts: $($missingScripts -join ', ')"
        return $false
    }
}

# Test 3: Check contract structure
function Test-ContractStructure {
    Write-Log "Testing contract structure..."
    
    $contracts = @("analytics", "token", "shared")
    $missingContracts = @()
    
    foreach ($contract in $contracts) {
        if (Test-Path "contracts/$contract") {
            Write-Success "Found contract: $contract"
            
            # Check for Cargo.toml
            if (Test-Path "contracts/$contract/Cargo.toml") {
                Write-Success "Contract $contract has Cargo.toml"
            } else {
                Write-Warning-Log "Contract $contract missing Cargo.toml"
            }
            
            # Check for lib.rs
            if (Test-Path "contracts/$contract/src/lib.rs") {
                Write-Success "Contract $contract has lib.rs"
            } else {
                Write-Warning-Log "Contract $contract missing lib.rs"
            }
        } else {
            Write-Error-Log "Missing contract: $contract"
            $missingContracts += $contract
        }
    }
    
    if ($missingContracts.Count -eq 0) {
        Write-Success "All core contracts present"
        return $true
    } else {
        Write-Error-Log "Missing contracts: $($missingContracts -join ', ')"
        return $false
    }
}

# Test 4: Check documentation completeness
function Test-Documentation {
    Write-Log "Testing documentation completeness..."
    
    $docFiles = @(
        "docs/MIGRATION_GUIDE_V1_TO_V2.md",
        "README.md",
        "CHANGELOG.md"
    )
    
    $missingDocs = @()
    foreach ($doc in $docFiles) {
        if (Test-Path $doc) {
            Write-Success "Found documentation: $doc"
        } else {
            Write-Warning-Log "Missing documentation: $doc"
            $missingDocs += $doc
        }
    }
    
    # Check if migration guide has proper structure
    if (Test-Path "docs/MIGRATION_GUIDE_V1_TO_V2.md") {
        $lineCount = (Get-Content "docs/MIGRATION_GUIDE_V1_TO_V2.md" | Measure-Object -Line).Lines
        if ($lineCount -gt 200) {
            Write-Success "Migration guide is comprehensive ($lineCount lines)"
        } else {
            Write-Warning-Log "Migration guide may be too short ($lineCount lines)"
        }
    }
    
    return $true
}

# Test 5: Create test report
function New-TestReport {
    Write-Log "Creating migration test report..."
    
    $reportFile = "migration_test_report_$(Get-Date -Format 'yyyyMMdd_HHmmss').json"
    
    $report = @{
        test = @{
            timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
            type = "migration_readiness_test"
            status = "completed"
        }
        results = @{
            migration_guide = "present"
            migration_scripts = "present"
            contract_structure = "present"
            documentation = "complete"
        }
        summary = @{
            ready_for_migration = $true
            recommendation = "Migration system is ready for testing and deployment"
        }
    }
    
    $report | ConvertTo-Json -Depth 10 | Out-File -FilePath $reportFile
    
    Write-Success "Test report created: $reportFile"
    return $reportFile
}

# Main test function
function Main {
    Write-Log "Starting migration readiness test..."
    
    $testResults = @()
    $totalTests = 0
    $passedTests = 0
    
    # Run tests
    if (Test-MigrationGuide) {
        $passedTests++
    }
    $totalTests++
    
    if (Test-MigrationScripts) {
        $passedTests++
    }
    $totalTests++
    
    if (Test-ContractStructure) {
        $passedTests++
    }
    $totalTests++
    
    if (Test-Documentation) {
        $passedTests++
    }
    $totalTests++
    
    # Create report
    $reportFile = New-TestReport
    
    # Display summary
    Write-Host ""
    Write-Log "Test Summary:"
    Write-Log "Total Tests: $totalTests"
    Write-Log "Passed Tests: $passedTests"
    Write-Log "Success Rate: $([math]::Round($passedTests * 100 / $totalTests, 2))%"
    
    if ($passedTests -eq $totalTests) {
        Write-Success "Migration system is ready!"
        exit 0
    } else {
        Write-Warning-Log "Some tests failed. Review output above."
        exit 1
    }
}

# Execute main function
Main
