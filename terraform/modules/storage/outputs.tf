output "backup_bucket_name" {
  value       = aws_s3_bucket.backups.id
  description = "Backup bucket name"
}

output "backup_bucket_arn" {
  value       = aws_s3_bucket.backups.arn
  description = "Backup bucket ARN"
}

output "logs_bucket_name" {
  value       = aws_s3_bucket.logs.id
  description = "Logs bucket name"
}

output "logs_bucket_arn" {
  value       = aws_s3_bucket.logs.arn
  description = "Logs bucket ARN"
}

output "artifacts_bucket_name" {
  value       = aws_s3_bucket.artifacts.id
  description = "Artifacts bucket name"
}

output "artifacts_bucket_arn" {
  value       = aws_s3_bucket.artifacts.arn
  description = "Artifacts bucket ARN"
}
