output "eks_cluster_name" {
  value       = aws_eks_cluster.main.name
  description = "EKS cluster name"
}

output "eks_cluster_endpoint" {
  value       = aws_eks_cluster.main.endpoint
  description = "EKS cluster API endpoint"
}

output "eks_cluster_arn" {
  value       = aws_eks_cluster.main.arn
  description = "EKS cluster ARN"
}

output "eks_node_group_id" {
  value       = aws_eks_node_group.main.id
  description = "EKS node group ID"
}
