# Security Group for RDS
resource "aws_security_group" "rds" {
  name        = "strellerminds-rds-${var.environment}"
  description = "Security group for RDS database"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 5432
    to_port     = 5432
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/8"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "strellerminds-rds-sg-${var.environment}"
  }
}

# DB Subnet Group
resource "aws_db_subnet_group" "main" {
  name       = "strellerminds-db-subnet-${var.environment}"
  subnet_ids = var.subnet_ids

  tags = {
    Name = "strellerminds-db-subnet-group-${var.environment}"
  }
}

# RDS Instance
resource "aws_db_instance" "main" {
  identifier     = "strellerminds-db-${var.environment}"
  engine         = var.db_engine
  engine_version = var.db_engine_version
  instance_class = var.db_instance_class

  allocated_storage     = var.db_allocated_storage
  storage_type          = "gp2"
  storage_encrypted     = true
  iops                  = 1000

  db_name  = var.db_name
  username = var.db_username

  db_subnet_group_name            = aws_db_subnet_group.main.name
  vpc_security_group_ids          = [aws_security_group.rds.id]
  publicly_accessible             = false
  multi_az                        = var.multi_az
  backup_retention_period         = var.backup_retention_period
  backup_window                   = "03:00-04:00"
  maintenance_window              = "mon:04:00-mon:05:00"
  enabled_cloudwatch_logs_exports = ["postgresql"]

  auto_minor_version_upgrade = true
  skip_final_snapshot        = false
  final_snapshot_identifier  = "strellerminds-db-final-snapshot-${formatdate("YYYY-MM-DD-hhmm", timestamp())}"

  performance_insights_enabled = var.environment != "dev" ? true : false

  tags = {
    Name = "strellerminds-db-${var.environment}"
  }
}

# Parameter Group for RDS
resource "aws_db_parameter_group" "main" {
  name   = "strellerminds-pg-${var.environment}"
  family = "${var.db_engine}${split(".", var.db_engine_version)[0]}"

  parameter {
    name  = "log_statement"
    value = "all"
  }

  parameter {
    name  = "log_duration"
    value = "1"
  }

  tags = {
    Name = "strellerminds-param-group-${var.environment}"
  }
}
