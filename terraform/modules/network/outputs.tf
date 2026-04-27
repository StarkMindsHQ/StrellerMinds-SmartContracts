output "vpc_id" {
  value       = aws_vpc.main.id
  description = "ID of the VPC"
}

output "vpc_cidr" {
  value       = aws_vpc.main.cidr_block
  description = "CIDR block of the VPC"
}

output "public_subnet_ids" {
  value       = aws_subnet.public[*].id
  description = "IDs of public subnets"
}

output "private_subnet_ids" {
  value       = aws_subnet.private[*].id
  description = "IDs of private subnets"
}

output "nat_gateway_ids" {
  value       = aws_nat_gateway.main[*].id
  description = "IDs of NAT Gateways"
}

output "internet_gateway_id" {
  value       = aws_internet_gateway.main.id
  description = "ID of the Internet Gateway"
}
