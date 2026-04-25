# Strellerminds Infrastructure as Code

This directory contains Terraform configuration for deploying and managing the Strellerminds infrastructure on AWS.

## Structure

- `main.tf` - Root module configuration that orchestrates all infrastructure
- `variables.tf` - Input variables for the root module
- `modules/` - Reusable Terraform modules
  - `network/` - VPC, subnets, NAT gateways, internet gateways
  - `compute/` - EKS cluster and node groups
  - `storage/` - S3 buckets for backups, logs, and artifacts
  - `database/` - RDS PostgreSQL database
- `environments/` - Environment-specific configurations
  - `dev/` - Development environment
  - `staging/` - Staging environment
  - `prod/` - Production environment

## Prerequisites

1. AWS Account with appropriate permissions
2. Terraform >= 1.5.0
3. AWS CLI configured with credentials
4. S3 bucket for Terraform state backend (must be created manually)
5. DynamoDB table for state locking (must be created manually)

## State Backend Setup

Before running Terraform, create the S3 bucket and DynamoDB table for state management:

```bash
aws s3api create-bucket \
  --bucket strellerminds-terraform-state \
  --region us-east-1

aws dynamodb create-table \
  --table-name terraform-locks \
  --attribute-definitions AttributeName=LockID,AttributeType=S \
  --key-schema AttributeName=LockID,KeyType=HASH \
  --provisioned-throughput ReadCapacityUnits=5,WriteCapacityUnits=5 \
  --region us-east-1
```

## Deployment

### Initialize Terraform

```bash
cd terraform
terraform init
```

### Plan Infrastructure

```bash
terraform plan -var-file="environments/dev/terraform.tfvars"
```

### Apply Configuration

```bash
terraform apply -var-file="environments/dev/terraform.tfvars"
```

### Switch Environments

To deploy to staging or production:

```bash
terraform plan -var-file="environments/staging/terraform.tfvars"
terraform apply -var-file="environments/staging/terraform.tfvars"
```

## Module Descriptions

### Network Module

Provisions VPC infrastructure:
- VPC with configurable CIDR block
- Public and private subnets across multiple AZs
- Internet Gateway for public subnet routing
- NAT Gateways for private subnet egress
- Network ACLs for ingress/egress rules

### Compute Module

Provisions EKS cluster:
- EKS cluster with specified Kubernetes version
- Auto Scaling node groups
- IAM roles and policies
- Cluster autoscaler configuration
- CloudWatch logging for cluster components

### Storage Module

Provisions S3 storage:
- Backup bucket with versioning and lifecycle policies
- Logs bucket with automatic expiration
- Artifacts bucket with encryption
- Public access blocking on all buckets

### Database Module

Provisions RDS instance:
- PostgreSQL database with configurable version
- Multi-AZ option for production
- Automated backups with configurable retention
- CloudWatch logs export
- Parameter group for database tuning

## Environment Variables

Set these before running Terraform:

```bash
export AWS_REGION=us-east-1
export TF_VAR_db_username=<admin_username>
```

## Outputs

After successful deployment, Terraform outputs:
- VPC ID
- EKS cluster name and endpoint
- RDS endpoint
- S3 bucket names

## Maintenance

### Updating Infrastructure

Modify the relevant `.tfvars` file and run:

```bash
terraform plan -var-file="environments/dev/terraform.tfvars"
terraform apply -var-file="environments/dev/terraform.tfvars"
```

### Destroying Infrastructure

```bash
terraform destroy -var-file="environments/dev/terraform.tfvars"
```

**Warning:** This will destroy all infrastructure including databases. Use with caution in production.

## Best Practices

1. Always use `terraform plan` before applying changes
2. Keep state files secure (already encrypted in S3)
3. Use separate state files per environment
4. Review all changes before applying to production
5. Tag all resources appropriately
6. Use remote state backend for team collaboration
7. Implement cost controls via AWS budgets
8. Regularly backup Terraform state

## Troubleshooting

### State Lock

If Terraform gets stuck with a lock:

```bash
terraform force-unlock <LOCK_ID>
```

### Missing Permissions

Ensure AWS credentials have these permissions:
- EC2: CreateVpc, CreateSubnet, CreateRoute, etc.
- EKS: CreateCluster, CreateNodegroup
- RDS: CreateDBInstance
- S3: CreateBucket, PutBucketPolicy
- IAM: CreateRole, AttachRolePolicy

## Support

For issues or questions, contact the DevOps team or check the [Terraform AWS Provider Documentation](https://registry.terraform.io/providers/hashicorp/aws/latest).
