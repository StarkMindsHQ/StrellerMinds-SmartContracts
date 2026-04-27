output "rds_endpoint" {
  value       = aws_db_instance.main.endpoint
  sensitive   = true
  description = "RDS database endpoint"
}

output "rds_address" {
  value       = aws_db_instance.main.address
  sensitive   = true
  description = "RDS database address"
}

output "rds_identifier" {
  value       = aws_db_instance.main.identifier
  description = "RDS database identifier"
}

output "rds_resource_id" {
  value       = aws_db_instance.main.resource_id
  description = "RDS resource ID"
}
