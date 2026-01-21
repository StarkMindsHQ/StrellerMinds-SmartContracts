# Security Guidelines

## Security-First Mindset

All development for StrellerMinds smart contracts must follow these security principles:

1. **Principle of Least Privilege**: Contracts should request only the permissions they need.
2. **Input Validation**: All inputs must be validated before processing.
3. **Error Handling**: Proper error handling must be implemented for all operations.
4. **Access Control**: Clear access control mechanisms must be in place.
5. **Audit Readiness**: Code should be written with clarity for future audits.

## Security Review Process

1. All PRs must undergo security review before merging.
2. Static analysis tools must be run on all code.
3. Test coverage must include security-focused test cases.

## Release Verification

All releases include Software Bill of Materials (SBOM) and cryptographic signatures for verification.

### SBOM Verification

Each release includes:
- `sbom.spdx.json` - Complete repository SBOM in SPDX format
- `sbom-wasm.spdx.json` - WASM artifacts specific SBOM
- Corresponding `.sig` files for each SBOM

### Artifact Verification

All release artifacts are signed using Cosign when signing keys are available:

#### Prerequisites for Verification

```bash
# Install Cosign
curl -O -L "https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64"
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
sudo chmod +x /usr/local/bin/cosign

# Install Syft for SBOM validation
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin
```

#### Verify Release Artifacts

1. **Download release assets** from the GitHub release page
2. **Verify WASM artifacts:**
   ```bash
   # Verify each WASM file
   cosign verify-blob --key cosign.pub --signature <filename>.wasm.sig <filename>.wasm
   ```

3. **Verify SBOM files:**
   ```bash
   # Verify main SBOM
   cosign verify-blob --key cosign.pub --signature sbom.spdx.json.sig sbom.spdx.json
   
   # Verify WASM-specific SBOM
   cosign verify-blob --key cosign.pub --signature sbom-wasm.spdx.json.sig sbom-wasm.spdx.json
   ```

4. **Validate SBOM content:**
   ```bash
   # Validate SPDX JSON format
   syft validate sbom.spdx.json
   
   # Convert to human-readable format
   syft convert sbom.spdx.json -o table
   ```

5. **Verify checksums:**
   ```bash
   # Verify SHA256 checksums
   sha256sum -c SHA256SUMS.txt
   ```

#### Public Key

The public key for verification is available at: `https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/releases`

### Security Best Practices

- Always verify signatures before using release artifacts
- Check SBOM for known vulnerabilities
- Use only officially signed releases in production
- Report any signature verification failures immediately

## Vulnerability Reporting

If you discover a security vulnerability, please do NOT open an issue. Email security@strellerminds.com instead.
