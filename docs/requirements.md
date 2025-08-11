# Requirements for BLC PostgreSQL HA - Add Replica Functionality

## Overview
This document outlines all the requirements needed for the `add-replica` command to function properly in a production environment.

## Prerequisites

### 1. SSH Access and Authentication

#### SSH Key Setup
- **SSH keys must be configured** between all nodes in the cluster
- **Key-based authentication** (no password prompts)
- **SSH user** must have sudo privileges on target nodes

```bash
# Generate SSH key on source node
ssh-keygen -t rsa -b 4096 -C "blcpg-ha@cluster"

# Copy public key to target nodes
ssh-copy-id -i ~/.ssh/id_rsa.pub user@target-node

# Test SSH access
ssh user@target-node 'echo "SSH access confirmed"'
```

#### SSH Configuration
```bash
# /etc/ssh/sshd_config on all nodes
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
AuthorizedKeysFile .ssh/authorized_keys
```

### 2. PostgreSQL Configuration

#### User and Permissions
```sql
-- Create replication user on primary
CREATE ROLE replicator WITH REPLICATION LOGIN PASSWORD 'replicator123';

-- Grant necessary permissions
GRANT CONNECT ON DATABASE postgres TO replicator;
GRANT USAGE ON SCHEMA public TO replicator;
```

#### PostgreSQL Configuration Files
```bash
# postgresql.conf on primary
wal_level = replica
max_wal_senders = 10
max_replication_slots = 10
hot_standby = on

# pg_hba.conf on primary
host replication replicator 0.0.0.0/0 md5
```

### 3. Network Requirements

#### Firewall Configuration
```bash
# Allow PostgreSQL port (5432) between nodes
sudo ufw allow from 10.0.0.0/8 to any port 5432
sudo ufw allow from 172.16.0.0/12 to any port 5432
sudo ufw allow from 192.168.0.0/16 to any port 5432

# Allow SSH port (22) between nodes
sudo ufw allow from 10.0.0.0/8 to any port 22
sudo ufw allow from 172.16.0.0/12 to any port 22
sudo ufw allow from 192.168.0.0/16 to any port 22
```

#### Network Connectivity
- **All nodes must be reachable** via IP address
- **DNS resolution** must work between nodes
- **No network latency** > 100ms between nodes
- **Bandwidth** > 100Mbps for backup operations

### 4. System Requirements

#### Operating System
- **Ubuntu 20.04+** or **CentOS 8+** or **RHEL 8+**
- **64-bit architecture** (amd64, arm64)
- **Minimum 4GB RAM** per node
- **Minimum 20GB free disk space** per node

#### System Users
```bash
# Create postgres user if not exists
sudo useradd -r -s /bin/bash -d /var/lib/postgresql -m postgres

# Set proper permissions
sudo chown -R postgres:postgres /var/lib/postgresql
sudo chmod 700 /var/lib/postgresql
```

#### Package Dependencies
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y postgresql-15 postgresql-client-15 openssh-server

# CentOS/RHEL
sudo yum install -y postgresql15-server postgresql15 openssh-server
```

### 5. Storage Requirements

#### Disk Space
- **Data directory**: Minimum 20GB free
- **WAL directory**: Minimum 5GB free
- **Backup temporary space**: Minimum 10GB free

#### File System
- **Ext4** or **XFS** recommended
- **No NFS** for data directories
- **Local storage** only for PostgreSQL data

### 6. Security Requirements

#### SSL/TLS Configuration
```bash
# Generate SSL certificates
sudo -u postgres openssl req -new -x509 -days 365 -nodes -text -out server.crt -keyout server.key -subj "/CN=postgresql-cluster"

# Configure SSL in postgresql.conf
ssl = on
ssl_cert_file = 'server.crt'
ssl_key_file = 'server.key'
```

#### Access Control
```bash
# Restrict access to PostgreSQL
sudo ufw deny 5432/tcp
sudo ufw allow from 10.0.0.0/8 to any port 5432
```

### 7. Monitoring and Logging

#### Log Configuration
```bash
# postgresql.conf
logging_collector = on
log_directory = 'log'
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'
log_rotation_age = 1d
log_rotation_size = 100MB
```

#### Monitoring Tools
- **Prometheus** for metrics collection
- **Grafana** for visualization
- **pg_stat_statements** for query monitoring

## Installation Checklist

### Pre-Installation
- [ ] SSH keys configured between all nodes
- [ ] Network connectivity verified
- [ ] Firewall rules configured
- [ ] PostgreSQL packages installed
- [ ] System users created
- [ ] Disk space verified
- [ ] SSL certificates generated

### Installation Steps
1. **Verify SSH access**
   ```bash
   ssh user@target-node 'echo "SSH OK"'
   ```

2. **Check PostgreSQL installation**
   ```bash
   ssh user@target-node 'systemctl status postgresql'
   ```

3. **Verify replication user**
   ```bash
   ssh user@source-node 'sudo -u postgres psql -c "SELECT rolname FROM pg_roles WHERE rolname = '\''replicator'\'';"'
   ```

4. **Test network connectivity**
   ```bash
   ssh user@target-node 'telnet source-node 5432'
   ```

## Troubleshooting

### Common Issues

#### SSH Connection Failed
```bash
# Check SSH service
sudo systemctl status sshd

# Check SSH keys
ls -la ~/.ssh/
ssh-add -l

# Test SSH connection
ssh -v user@target-node
```

#### PostgreSQL Connection Failed
```bash
# Check PostgreSQL service
sudo systemctl status postgresql

# Check PostgreSQL logs
sudo tail -f /var/log/postgresql/postgresql-15-main.log

# Test connection
psql -h source-node -U replicator -d postgres
```

#### Insufficient Disk Space
```bash
# Check disk space
df -h

# Clean up old logs
sudo find /var/log -name "*.log" -mtime +7 -delete

# Clean up old backups
sudo find /var/backups -name "*.sql" -mtime +30 -delete
```

### Error Codes and Solutions

| Error Code | Description | Solution |
|------------|-------------|----------|
| SSH_001 | SSH connection failed | Verify SSH keys and network connectivity |
| PG_001 | PostgreSQL not installed | Install PostgreSQL packages |
| PG_002 | Replication user not found | Create replicator user on primary |
| PG_003 | Insufficient disk space | Free up disk space or expand storage |
| NET_001 | Network connectivity failed | Check firewall rules and network configuration |

## Performance Considerations

### Backup Performance
- **Use SSD storage** for better I/O performance
- **Increase shared_buffers** for larger datasets
- **Tune checkpoint_segments** for write-heavy workloads
- **Monitor network bandwidth** during backup operations

### Replication Performance
- **Use synchronous replication** for data consistency
- **Monitor replication lag** regularly
- **Tune wal_buffers** for better WAL performance
- **Use connection pooling** for application connections

## Security Best Practices

### Network Security
- **Use VPN** for inter-node communication
- **Implement network segmentation**
- **Regular security audits**
- **Monitor access logs**

### Data Security
- **Encrypt data at rest**
- **Use SSL/TLS for connections**
- **Regular backups**
- **Access control lists**

## Maintenance Procedures

### Regular Maintenance
- **Weekly**: Check disk space and logs
- **Monthly**: Review security settings
- **Quarterly**: Performance tuning
- **Annually**: Security audit

### Backup Procedures
- **Daily**: Automated backups
- **Weekly**: Full backups
- **Monthly**: Archive backups
- **Quarterly**: Disaster recovery tests

## Support and Documentation

### Documentation
- **Installation guides**
- **Configuration examples**
- **Troubleshooting guides**
- **Performance tuning guides**

### Support Channels
- **GitHub Issues**: Bug reports and feature requests
- **Documentation**: Online documentation
- **Community**: User forums and discussions
- **Enterprise**: Commercial support options 