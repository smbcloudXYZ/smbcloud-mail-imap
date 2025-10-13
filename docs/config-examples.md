# Mail Server Configuration Examples

This document provides example configurations for the mail server, including SMTP, POP3, and IMAP settings.

## Table of Contents

1. [Basic Configuration](#basic-configuration)
2. [POP3 Server Configuration](#pop3-server-configuration)
3. [SMTP Server Configuration](#smtp-server-configuration)
4. [Integrated Mail Server Configuration](#integrated-mail-server-configuration)
5. [Security Configuration](#security-configuration)
6. [Production Configuration](#production-configuration)

## Basic Configuration

### Simple POP3 Server (Development)

```toml
# pop3-dev.toml
[server]
host = "127.0.0.1"
port = 110
timeout = 600  # 10 minutes

[authentication]
# Simple file-based authentication for development
backend = "file"
users_file = "users.txt"

[storage]
# Maildir storage backend
backend = "maildir"
maildir_path = "./mailboxes"
```

**users.txt** format:
```
testuser:testpass
alice:password123
bob:secret
```

### Simple SMTP Server (Development)

```toml
# smtp-dev.toml
[server]
host = "127.0.0.1"
port = 2525
timeout = 300

[storage]
backend = "maildir"
maildir_path = "./mailboxes"

[tls]
enabled = false
```

## POP3 Server Configuration

### Minimal POP3 Configuration

```toml
[server]
host = "0.0.0.0"
port = 110

[authentication]
backend = "file"
users_file = "/etc/pop3/users.txt"

[storage]
backend = "maildir"
maildir_path = "/var/mail"
```

### POP3 with TLS/SSL (POP3S)

```toml
[server]
host = "0.0.0.0"
port = 995  # POP3S standard port
timeout = 600

[tls]
enabled = true
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"
min_tls_version = "1.2"

[authentication]
backend = "database"
db_url = "postgresql://popuser:password@localhost/maildb"

[storage]
backend = "maildir"
maildir_path = "/var/mail"

[limits]
max_message_size = 52428800  # 50MB
max_connections = 500
max_messages_per_session = 1000

[logging]
level = "info"
file = "/var/log/pop3/server.log"
```

### POP3 with Database Authentication

```toml
[server]
host = "0.0.0.0"
port = 110

[authentication]
backend = "database"
db_type = "postgresql"
db_url = "postgresql://user:password@localhost:5432/maildb"
# SQL query to authenticate user
auth_query = "SELECT password_hash FROM users WHERE email = $1 AND active = true"
# Password hashing algorithm
password_hash = "bcrypt"

[storage]
backend = "database"
db_url = "postgresql://user:password@localhost:5432/maildb"
```

### POP3 with LDAP Authentication

```toml
[server]
host = "0.0.0.0"
port = 110

[authentication]
backend = "ldap"
ldap_url = "ldap://ldap.example.com:389"
ldap_base_dn = "ou=users,dc=example,dc=com"
ldap_bind_dn = "cn=admin,dc=example,dc=com"
ldap_bind_password = "admin_password"
ldap_user_filter = "(&(objectClass=person)(mail=%u))"

[storage]
backend = "maildir"
maildir_path = "/var/mail"
```

## SMTP Server Configuration

### Basic SMTP Configuration

```toml
[server]
host = "0.0.0.0"
port = 25
timeout = 300

[relay]
# Allow relay from local network
allowed_networks = ["127.0.0.0/8", "192.168.0.0/16"]

[storage]
backend = "maildir"
maildir_path = "/var/mail"

[limits]
max_message_size = 10485760  # 10MB
max_recipients = 100
```

### SMTP with Authentication and TLS

```toml
[server]
host = "0.0.0.0"
port = 587  # Submission port
timeout = 300

[tls]
enabled = true
starttls = true  # Allow STARTTLS
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"

[authentication]
required = true  # Require authentication for sending
backend = "database"
db_url = "postgresql://user:password@localhost/maildb"

[storage]
backend = "maildir"
maildir_path = "/var/mail"
```

## Integrated Mail Server Configuration

### Complete Mail Server (SMTP + POP3 + IMAP)

```toml
# mail-server.toml

[global]
hostname = "mail.example.com"
domain = "example.com"

# Shared authentication backend
[authentication]
backend = "database"
db_url = "postgresql://mailuser:password@localhost:5432/maildb"
password_hash = "argon2"

# Shared storage backend
[storage]
backend = "maildir"
maildir_path = "/var/mail"
# Alternative: use database storage
# backend = "database"
# db_url = "postgresql://mailuser:password@localhost:5432/maildb"

# SMTP Server Configuration
[smtp]
enabled = true
host = "0.0.0.0"
port = 25
submission_port = 587
timeout = 300

[smtp.tls]
enabled = true
starttls = true
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"

[smtp.limits]
max_message_size = 52428800  # 50MB
max_recipients = 100
max_connections = 1000

# POP3 Server Configuration
[pop3]
enabled = true
host = "0.0.0.0"
port = 110
timeout = 600

[pop3.tls]
enabled = true
port = 995  # POP3S port
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"

[pop3.limits]
max_message_size = 52428800
max_connections = 500
max_messages_per_session = 1000

# IMAP Server Configuration
[imap]
enabled = true
host = "0.0.0.0"
port = 143
timeout = 1800  # 30 minutes

[imap.tls]
enabled = true
starttls = true
port = 993  # IMAPS port
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"

[imap.limits]
max_message_size = 52428800
max_connections = 1000

# Logging Configuration
[logging]
level = "info"
format = "json"
smtp_log = "/var/log/mail/smtp.log"
pop3_log = "/var/log/mail/pop3.log"
imap_log = "/var/log/mail/imap.log"
max_size = "100MB"
max_age = 30  # days
compress = true
```

## Security Configuration

### Security Best Practices Configuration

```toml
[security]
# Rate limiting
[security.rate_limit]
enabled = true
max_connections_per_ip = 10
max_auth_attempts = 3
auth_failure_delay = 5  # seconds
ban_duration = 3600  # 1 hour

# IP whitelist/blacklist
[security.access_control]
whitelist = ["192.168.1.0/24", "10.0.0.0/8"]
blacklist = ["203.0.113.0/24"]

# Password policies
[security.passwords]
min_length = 12
require_uppercase = true
require_lowercase = true
require_digit = true
require_special = true
hash_algorithm = "argon2"
hash_iterations = 3

# TLS settings
[security.tls]
min_version = "1.2"
cipher_suites = [
    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
]
prefer_server_ciphers = true

# Session security
[security.sessions]
timeout = 600
max_idle_time = 300
require_encryption = true
```

## Production Configuration

### High-Performance Production Setup

```toml
# production.toml

[global]
hostname = "mail.example.com"
domain = "example.com"
environment = "production"

# Performance settings
[performance]
worker_threads = 16
max_blocking_threads = 512
async_io_threads = 8

# Connection pooling
[database]
url = "postgresql://mailuser:password@db-cluster.example.com:5432/maildb"
max_connections = 100
min_connections = 10
connection_timeout = 5
idle_timeout = 600

# Redis cache for sessions and rate limiting
[cache]
backend = "redis"
url = "redis://cache-cluster.example.com:6379"
ttl = 3600
max_connections = 50

# SMTP Configuration
[smtp]
enabled = true
host = "0.0.0.0"
port = 25
submission_port = 587
timeout = 300

[smtp.tls]
enabled = true
starttls = true
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"
min_version = "1.2"

[smtp.limits]
max_message_size = 52428800
max_recipients = 100
max_connections = 2000
rate_limit_per_sender = 100  # messages per hour

[smtp.queue]
backend = "database"
retry_attempts = 5
retry_delay = 300  # 5 minutes
max_age = 432000  # 5 days

# POP3 Configuration
[pop3]
enabled = true
host = "0.0.0.0"
port = 110
timeout = 600

[pop3.tls]
enabled = true
port = 995
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"

[pop3.limits]
max_connections = 1000
max_messages_per_session = 1000
rate_limit_per_ip = 50  # connections per hour

# Authentication
[authentication]
backend = "database"
db_url = "postgresql://mailuser:password@db-cluster.example.com:5432/maildb"
password_hash = "argon2"
cache_ttl = 3600
mfa_enabled = true

# Storage
[storage]
backend = "s3"
s3_bucket = "mail-storage"
s3_region = "us-east-1"
s3_endpoint = "https://s3.amazonaws.com"
# Or use Maildir
# backend = "maildir"
# maildir_path = "/mnt/mail-storage"

# Monitoring and Metrics
[monitoring]
enabled = true
prometheus_port = 9090
health_check_port = 8080

[monitoring.metrics]
collect_interval = 60  # seconds
retention_days = 30

# Logging
[logging]
level = "info"
format = "json"
output = "syslog"
syslog_address = "localhost:514"
syslog_facility = "mail"

# Backup
[backup]
enabled = true
schedule = "0 2 * * *"  # Daily at 2 AM
destination = "s3://backups/mail"
retention_days = 90
```

### Multi-Node Cluster Configuration

```toml
# cluster-node.toml

[cluster]
enabled = true
node_id = "node-1"
nodes = [
    "node-1.example.com:7000",
    "node-2.example.com:7000",
    "node-3.example.com:7000",
]

[cluster.consensus]
algorithm = "raft"
election_timeout = 1000
heartbeat_interval = 100

[cluster.replication]
factor = 3
sync_mode = "async"

# Shared distributed storage
[storage]
backend = "distributed"
cluster_nodes = [
    "storage-1.example.com:9000",
    "storage-2.example.com:9000",
    "storage-3.example.com:9000",
]

# Load balancer configuration
[load_balancer]
algorithm = "round_robin"
health_check_interval = 5
health_check_timeout = 2
```

## Environment-Specific Configurations

### Development

```toml
[global]
environment = "development"
debug = true

[server]
host = "127.0.0.1"

[logging]
level = "debug"
output = "stdout"

[tls]
enabled = false  # Use plain text for easier debugging
```

### Staging

```toml
[global]
environment = "staging"
debug = false

[server]
host = "0.0.0.0"

[logging]
level = "info"
output = "file"

[tls]
enabled = true
# Use staging certificates
cert_path = "/etc/ssl/staging/cert.pem"
key_path = "/etc/ssl/staging/key.pem"
```

### Production

```toml
[global]
environment = "production"
debug = false

[server]
host = "0.0.0.0"

[logging]
level = "warn"
output = "syslog"

[tls]
enabled = true
cert_path = "/etc/ssl/certs/production.crt"
key_path = "/etc/ssl/private/production.key"

[security]
strict_mode = true
require_tls = true
```

## Docker Configuration

### Docker Compose Example

```yaml
# docker-compose.yml
version: '3.8'

services:
  mail-server:
    image: smbcloud-mail:latest
    container_name: mail-server
    ports:
      - "25:25"      # SMTP
      - "587:587"    # SMTP Submission
      - "110:110"    # POP3
      - "995:995"    # POP3S
      - "143:143"    # IMAP
      - "993:993"    # IMAPS
    volumes:
      - ./config:/etc/mail:ro
      - ./mail-data:/var/mail
      - ./logs:/var/log/mail
      - ./certs:/etc/ssl/mail:ro
    environment:
      - RUST_LOG=info
      - CONFIG_FILE=/etc/mail/production.toml
    depends_on:
      - postgres
      - redis
    restart: unless-stopped

  postgres:
    image: postgres:15
    container_name: mail-db
    volumes:
      - postgres-data:/var/lib/postgresql/data
    environment:
      - POSTGRES_DB=maildb
      - POSTGRES_USER=mailuser
      - POSTGRES_PASSWORD=securepassword
    restart: unless-stopped

  redis:
    image: redis:7
    container_name: mail-cache
    volumes:
      - redis-data:/data
    restart: unless-stopped

volumes:
  postgres-data:
  redis-data:
```

## Configuration Loading in Code

### Rust Configuration Loading Example

```rust
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct MailConfig {
    global: GlobalConfig,
    smtp: Option<SmtpConfig>,
    pop3: Option<Pop3Config>,
    imap: Option<ImapConfig>,
    authentication: AuthConfig,
    storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
struct Pop3Config {
    enabled: bool,
    host: String,
    port: u16,
    timeout: u64,
    tls: Option<TlsConfig>,
    limits: Option<LimitsConfig>,
}

fn load_config(path: &str) -> Result<MailConfig, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config: MailConfig = toml::from_str(&contents)?;
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config("mail-server.toml")?;
    
    if let Some(pop3_config) = config.pop3 {
        if pop3_config.enabled {
            println!("Starting POP3 server on {}:{}", 
                     pop3_config.host, pop3_config.port);
            // Start POP3 server with config
        }
    }
    
    Ok(())
}
```

## Notes

1. Always use strong passwords and secure password hashing (argon2, bcrypt)
2. Enable TLS/SSL for production environments
3. Implement rate limiting to prevent abuse
4. Use database or LDAP authentication in production
5. Regular backup of configuration and data
6. Monitor logs for security issues
7. Keep certificates up to date
8. Use environment variables for sensitive data (passwords, keys)
9. Test configurations in staging before production
10. Document any custom configuration changes
