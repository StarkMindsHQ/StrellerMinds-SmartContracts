variable "environment" {
  type        = string
  description = "Environment name (dev, staging, prod)"
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

variable "aws_region" {
  type        = string
  default     = "us-east-1"
  description = "AWS region for resource deployment"
}

# Networking variables
variable "vpc_cidr" {
  type        = string
  description = "CIDR block for the VPC"
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  type        = list(string)
  description = "List of availability zones"
}

variable "public_subnet_cidrs" {
  type        = list(string)
  description = "CIDR blocks for public subnets"
}

variable "private_subnet_cidrs" {
  type        = list(string)
  description = "CIDR blocks for private subnets"
}

variable "enable_nat_gateway" {
  type        = bool
  default     = true
  description = "Enable NAT Gateway for private subnets"
}

variable "enable_vpn_gateway" {
  type        = bool
  default     = false
  description = "Enable VPN Gateway"
}

# EKS variables
variable "eks_cluster_name" {
  type        = string
  description = "Name of the EKS cluster"
  default     = "strellerminds"
}

variable "eks_version" {
  type        = string
  description = "Kubernetes version for EKS"
  default     = "1.28"
}

variable "eks_node_count" {
  type        = number
  description = "Number of nodes in EKS cluster"
  default     = 3
}

variable "eks_instance_types" {
  type        = list(string)
  description = "Instance types for EKS nodes"
  default     = ["t3.medium"]
}

variable "enable_cluster_autoscaler" {
  type        = bool
  default     = true
  description = "Enable Kubernetes cluster autoscaler"
}

# Storage variables
variable "backup_bucket_name" {
  type        = string
  description = "S3 bucket name for backups"
}

variable "logs_bucket_name" {
  type        = string
  description = "S3 bucket name for logs"
}

variable "artifacts_bucket_name" {
  type        = string
  description = "S3 bucket name for artifacts"
}

variable "log_retention_days" {
  type        = number
  default     = 30
  description = "Number of days to retain logs"
}

# Database variables
variable "db_name" {
  type        = string
  description = "Name of the initial database"
}

variable "db_username" {
  type        = string
  description = "Master username for the database"
  sensitive   = true
}

variable "db_engine" {
  type        = string
  default     = "postgres"
  description = "Database engine type"
}

variable "db_engine_version" {
  type        = string
  default     = "15.3"
  description = "Database engine version"
}

variable "db_instance_class" {
  type        = string
  default     = "db.t3.micro"
  description = "RDS instance class"
}

variable "db_allocated_storage" {
  type        = number
  default     = 20
  description = "Allocated storage in GB"
}
