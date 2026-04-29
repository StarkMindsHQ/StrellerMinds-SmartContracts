variable "environment" {
  type        = string
  description = "Environment name"
}

variable "vpc_id" {
  type        = string
  description = "VPC ID"
}

variable "subnet_ids" {
  type        = list(string)
  description = "Subnet IDs for EKS cluster"
}

variable "eks_cluster_name" {
  type        = string
  description = "EKS cluster name"
}

variable "eks_version" {
  type        = string
  description = "Kubernetes version"
}

variable "eks_node_count" {
  type        = number
  description = "Number of nodes"
}

variable "eks_instance_types" {
  type        = list(string)
  description = "EC2 instance types for nodes"
}

variable "enable_cluster_autoscaler" {
  type        = bool
  description = "Enable cluster autoscaler"
}
