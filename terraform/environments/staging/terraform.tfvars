environment = "staging"
aws_region  = "us-east-1"

# Networking
vpc_cidr = "10.1.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]
public_subnet_cidrs  = ["10.1.1.0/24", "10.1.2.0/24"]
private_subnet_cidrs = ["10.1.11.0/24", "10.1.12.0/24"]
enable_nat_gateway = true
enable_vpn_gateway = true

# EKS
eks_cluster_name = "strellerminds-staging"
eks_version = "1.28"
eks_node_count = 3
eks_instance_types = ["t3.small"]
enable_cluster_autoscaler = true

# Storage
backup_bucket_name    = "strellerminds-backups-staging"
logs_bucket_name      = "strellerminds-logs-staging"
artifacts_bucket_name = "strellerminds-artifacts-staging"
log_retention_days = 30

# Database
db_name = "strellerminds_staging"
db_engine = "postgres"
db_engine_version = "15.3"
db_instance_class = "db.t3.small"
db_allocated_storage = 50
