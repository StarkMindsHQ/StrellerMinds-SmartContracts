environment = "dev"
aws_region  = "us-east-1"

# Networking
vpc_cidr = "10.0.0.0/16"
availability_zones = ["us-east-1a", "us-east-1b"]
public_subnet_cidrs  = ["10.0.1.0/24", "10.0.2.0/24"]
private_subnet_cidrs = ["10.0.11.0/24", "10.0.12.0/24"]
enable_nat_gateway = true
enable_vpn_gateway = false

# EKS
eks_cluster_name = "strellerminds-dev"
eks_version = "1.28"
eks_node_count = 2
eks_instance_types = ["t3.medium"]
enable_cluster_autoscaler = true

# Storage
backup_bucket_name    = "strellerminds-backups-dev"
logs_bucket_name      = "strellerminds-logs-dev"
artifacts_bucket_name = "strellerminds-artifacts-dev"
log_retention_days = 7

# Database
db_name = "strellerminds_dev"
db_engine = "postgres"
db_engine_version = "15.3"
db_instance_class = "db.t3.micro"
db_allocated_storage = 20
