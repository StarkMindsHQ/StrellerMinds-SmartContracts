terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket         = "strellerminds-terraform-state"
    key            = "terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-locks"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Project     = "strellerminds"
      Environment = var.environment
      ManagedBy   = "Terraform"
      CreatedAt   = timestamp()
    }
  }
}

# VPC and Networking
module "network" {
  source = "./modules/network"

  environment = var.environment
  vpc_cidr    = var.vpc_cidr

  availability_zones = var.availability_zones
  public_subnet_cidrs  = var.public_subnet_cidrs
  private_subnet_cidrs = var.private_subnet_cidrs

  enable_nat_gateway = var.enable_nat_gateway
  enable_vpn_gateway = var.enable_vpn_gateway
}

# Compute Resources (EKS, Auto Scaling)
module "compute" {
  source = "./modules/compute"

  environment = var.environment
  vpc_id      = module.network.vpc_id
  subnet_ids  = module.network.private_subnet_ids

  eks_cluster_name    = var.eks_cluster_name
  eks_node_count      = var.eks_node_count
  eks_instance_types  = var.eks_instance_types
  eks_version         = var.eks_version

  enable_cluster_autoscaler = var.enable_cluster_autoscaler
}

# Storage (S3, EBS)
module "storage" {
  source = "./modules/storage"

  environment = var.environment

  backup_bucket_name    = var.backup_bucket_name
  logs_bucket_name      = var.logs_bucket_name
  artifacts_bucket_name = var.artifacts_bucket_name

  enable_versioning       = true
  enable_encryption       = true
  log_retention_days      = var.log_retention_days
}

# Database
module "database" {
  source = "./modules/database"

  environment = var.environment
  vpc_id      = module.network.vpc_id
  subnet_ids  = module.network.private_subnet_ids

  db_name              = var.db_name
  db_username          = var.db_username
  db_engine            = var.db_engine
  db_engine_version    = var.db_engine_version
  db_instance_class    = var.db_instance_class
  db_allocated_storage = var.db_allocated_storage

  multi_az = var.environment != "dev" ? true : false
  backup_retention_period = var.environment != "dev" ? 30 : 7
}

# Outputs
output "vpc_id" {
  value = module.network.vpc_id
}

output "eks_cluster_name" {
  value = module.compute.eks_cluster_name
}

output "eks_cluster_endpoint" {
  value = module.compute.eks_cluster_endpoint
}

output "rds_endpoint" {
  value = module.database.rds_endpoint
  sensitive = true
}

output "backup_bucket_name" {
  value = module.storage.backup_bucket_name
}
