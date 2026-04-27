environment = "prod"
aws_region  = "us-east-1"

# Networking
vpc_cidr = "10.2.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b", "us-east-1c"]
public_subnet_cidrs  = ["10.2.1.0/24", "10.2.2.0/24", "10.2.3.0/24"]
private_subnet_cidrs = ["10.2.11.0/24", "10.2.12.0/24", "10.2.13.0/24"]
enable_nat_gateway = true
enable_vpn_gateway = true

# EKS
eks_cluster_name = "strellerminds-prod"
eks_version = "1.28"
eks_node_count = 5
eks_instance_types = ["t3.medium", "t3.large"]
enable_cluster_autoscaler = true

# Storage
backup_bucket_name    = "strellerminds-backups-prod"
logs_bucket_name      = "strellerminds-logs-prod"
artifacts_bucket_name = "strellerminds-artifacts-prod"
log_retention_days = 90

# Database
db_name = "strellerminds_prod"
db_engine = "postgres"
db_engine_version = "15.3"
db_instance_class = "db.t3.large"
db_allocated_storage = 200
