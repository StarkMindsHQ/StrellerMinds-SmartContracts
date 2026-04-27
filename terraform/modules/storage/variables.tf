variable "environment" {
  type        = string
  description = "Environment name"
}

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

variable "enable_versioning" {
  type        = bool
  default     = true
  description = "Enable S3 versioning"
}

variable "enable_encryption" {
  type        = bool
  default     = true
  description = "Enable S3 encryption"
}

variable "log_retention_days" {
  type        = number
  description = "Log retention period in days"
}
