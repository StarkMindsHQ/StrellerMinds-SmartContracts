# Adding repository secrets for CI

Use repository secrets to store AWS credentials and cluster name for GitHub Actions. Do NOT commit any keys or kubeconfig files.

Recommended secrets to add (names used by the workflow):

- `AWS_ACCESS_KEY_ID`
- `AWS_SECRET_ACCESS_KEY`
- `AWS_REGION`
- `EKS_CLUSTER_NAME`

Add via GitHub UI:

1. Go to your repository on GitHub.
2. Settings → Secrets and variables → Actions → New repository secret.
3. Add each secret with the names above.

Add via GitHub CLI (`gh`) if you have it authenticated:

```bash
gh secret set AWS_ACCESS_KEY_ID -b "<your-access-key-id>"
gh secret set AWS_SECRET_ACCESS_KEY -b "<your-secret-access-key>"
gh secret set AWS_REGION -b "us-east-1"
gh secret set EKS_CLUSTER_NAME -b "strellerminds-staging"
```

Local testing (temporary environment variables):

For Bash/WSL:
```bash
export AWS_ACCESS_KEY_ID="<your-access-key-id>"
export AWS_SECRET_ACCESS_KEY="<your-secret-access-key>"
export AWS_REGION="us-east-1"
export EKS_CLUSTER_NAME="strellerminds-staging"
```

For PowerShell (temporary for session):
```powershell
$env:AWS_ACCESS_KEY_ID="<your-access-key-id>"
$env:AWS_SECRET_ACCESS_KEY="<your-secret-access-key>"
$env:AWS_REGION="us-east-1"
$env:EKS_CLUSTER_NAME="strellerminds-staging"
```

After setting secrets or environment variables, the workflow can be triggered manually from Actions (select the `Blue-Green Deploy to EKS` workflow and click "Run workflow") or by pushing to `main`.

PR link (branch pushed by me):

https://github.com/oscarj007/StrellerMinds-SmartContracts/pull/new/add/blue-green-deploy-workflow
