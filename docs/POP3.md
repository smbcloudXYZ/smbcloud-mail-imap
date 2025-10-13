# POP3 Protocol Documentation

This document provides comprehensive information about the POP3 (Post Office Protocol version 3) implementation, including setup, usage, testing, and integration with other mail protocols.

## Table of Contents

1. [Overview](#overview)
2. [POP3 Protocol Basics](#pop3-protocol-basics)
3. [Setup and Configuration](#setup-and-configuration)
4. [Usage Examples](#usage-examples)
5. [Testing](#testing)
6. [Integration with SMTP and IMAP](#integration-with-smtp-and-imap)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

## Overview

POP3 (Post Office Protocol version 3) is an Internet standard protocol used by email clients to retrieve emails from a mail server. It operates on TCP port 110 (or 995 for POP3S with SSL/TLS).

### Key Features

- Simple authentication mechanism (USER/PASS)
- Message retrieval and deletion
- Mailbox statistics and listing
- Minimal server-side state
- Optional TLS/SSL support (POP3S)

### POP3 vs IMAP

| Feature | POP3 | IMAP |
|---------|------|------|
| **Purpose** | Download and delete | Access and manage |
| **Messages** | Downloaded to client | Stored on server |
| **Folders** | No folder support | Multiple folders |
| **Offline Access** | Yes (after download) | Limited |
| **Multiple Clients** | Not ideal | Well supported |
| **Server Storage** | Minimal after download | Persistent |

## POP3 Protocol Basics

### Connection States

A POP3 session goes through three states:

1. **AUTHORIZATION** - Client authenticates with the server
2. **TRANSACTION** - Client retrieves and manages messages
3. **UPDATE** - Server commits changes (deletions) and closes connection

### Standard Commands

#### Authorization State Commands

| Command | Description | Example |
|---------|-------------|---------|
| `USER` | Specify username | `USER john@example.com` |
| `PASS` | Specify password | `PASS secretpassword` |
| `QUIT` | Close connection | `QUIT` |

#### Transaction State Commands

| Command | Description | Example |
|---------|-------------|---------|
| `STAT` | Get mailbox statistics | `STAT` |
| `LIST` | List messages | `LIST` or `LIST 1` |
| `RETR` | Retrieve message | `RETR 1` |
| `DELE` | Mark message for deletion | `DELE 1` |
| `NOOP` | No operation (keepalive) | `NOOP` |
| `RSET` | Unmark deleted messages | `RSET` |
| `TOP` | Get message headers | `TOP 1 10` |
| `UIDL` | Get unique message ID | `UIDL` or `UIDL 1` |
| `QUIT` | Close and commit | `QUIT` |

### Response Format

POP3 uses two types of responses:

- **Positive**: `+OK` followed by optional information
- **Negative**: `-ERR` followed by error message

Multi-line responses end with a line containing only `.` (period).

### Example Session

```
S: +OK POP3 server ready
C: USER testuser
S: +OK User accepted
C: PASS testpass
S: +OK Pass accepted
C: STAT
S: +OK 2 320
C: LIST
S: +OK 2 messages (320 octets)
S: 1 120
S: 2 200
S: .
C: RETR 1
S: +OK 120 octets
S: [message content]
S: .
C: DELE 1
S: +OK message 1 deleted
C: QUIT
S: +OK POP3 server signing off
```

## Setup and Configuration

### Basic Server Configuration

```rust
// Example POP3 server configuration structure
pub struct Pop3Config {
    /// Listening address
    pub host: String,
    
    /// Listening port (default: 110)
    pub port: u16,
    
    /// Enable TLS/SSL (POP3S on port 995)
    pub enable_tls: bool,
    
    /// Path to TLS certificate (if TLS enabled)
    pub tls_cert_path: Option<String>,
    
    /// Path to TLS private key (if TLS enabled)
    pub tls_key_path: Option<String>,
    
    /// Maximum message size (bytes)
    pub max_message_size: usize,
    
    /// Session timeout (seconds)
    pub timeout: u64,
    
    /// Authentication backend
    pub auth_backend: AuthBackend,
    
    /// Message storage backend
    pub storage_backend: StorageBackend,
}

impl Default for Pop3Config {
    fn default() -> Self {
        Pop3Config {
            host: "127.0.0.1".to_string(),
            port: 110,
            enable_tls: false,
            tls_cert_path: None,
            tls_key_path: None,
            max_message_size: 10 * 1024 * 1024, // 10MB
            timeout: 600, // 10 minutes
            auth_backend: AuthBackend::File,
            storage_backend: StorageBackend::Maildir,
        }
    }
}
```

### Starting the Server

```rust
use smbcloud_mail::pop3::Pop3Server;
use smbcloud_mail::pop3::Pop3Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create configuration
    let config = Pop3Config {
        host: "127.0.0.1".to_string(),
        port: 110,
        enable_tls: false,
        ..Default::default()
    };
    
    // Create and start server
    let server = Pop3Server::new(config);
    server.listen().await?;
    
    Ok(())
}
```

### Configuration File Example

Create a `pop3.toml` configuration file:

```toml
[server]
host = "127.0.0.1"
port = 110
timeout = 600

[tls]
enabled = false
cert_path = "/path/to/cert.pem"
key_path = "/path/to/key.pem"

[limits]
max_message_size = 10485760  # 10MB in bytes
max_connections = 100

[authentication]
backend = "file"
users_file = "/etc/pop3/users.txt"

[storage]
backend = "maildir"
maildir_path = "/var/mail"
```

## Usage Examples

### Using the Rust Client Example

```bash
# Run the POP3 client example
cargo run --example pop3_client
```

### Using Python

```bash
# Run the Python POP3 client
python3 docs/scripts/pop3-client.py
```

### Using netcat (nc) for Testing

```bash
# Connect to POP3 server
nc localhost 110

# Then type commands:
USER testuser
PASS testpass
STAT
LIST
RETR 1
QUIT
```

### Using telnet

```bash
telnet localhost 110
```

Then issue POP3 commands manually.

### Using mail clients

Configure your email client with:
- **Server**: localhost (or your server address)
- **Port**: 110 (or 995 for POP3S)
- **Username**: Your username
- **Password**: Your password
- **Security**: None (or SSL/TLS for POP3S)

## Testing

### Unit Tests

Example unit tests for POP3 functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pop3_connect() {
        let config = Pop3Config::default();
        let server = Pop3Server::new(config);
        // Test server initialization
        assert!(server.is_ok());
    }
    
    #[tokio::test]
    async fn test_pop3_authentication() {
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        let result = client.authenticate("testuser", "testpass").await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_pop3_stat() {
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        client.authenticate("testuser", "testpass").await.unwrap();
        let (count, size) = client.stat().await.unwrap();
        assert!(count >= 0);
        assert!(size >= 0);
    }
    
    #[tokio::test]
    async fn test_pop3_list() {
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        client.authenticate("testuser", "testpass").await.unwrap();
        let messages = client.list().await.unwrap();
        assert!(messages.is_ok());
    }
    
    #[tokio::test]
    async fn test_pop3_retrieve() {
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        client.authenticate("testuser", "testpass").await.unwrap();
        
        let (count, _) = client.stat().await.unwrap();
        if count > 0 {
            let message = client.retrieve(1).await.unwrap();
            assert!(!message.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_pop3_delete() {
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        client.authenticate("testuser", "testpass").await.unwrap();
        
        let (count, _) = client.stat().await.unwrap();
        if count > 0 {
            let result = client.delete(1).await;
            assert!(result.is_ok());
            
            // Reset to unmark
            client.reset().await.unwrap();
        }
    }
    
    #[tokio::test]
    async fn test_pop3_invalid_credentials() {
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        let result = client.authenticate("invalid", "invalid").await;
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_complete_pop3_session() {
        // Start POP3 server
        let config = Pop3Config::default();
        let server = Pop3Server::new(config);
        
        tokio::spawn(async move {
            server.listen().await.unwrap();
        });
        
        // Wait for server to start
        sleep(Duration::from_millis(100)).await;
        
        // Connect client
        let mut client = Pop3Client::connect("localhost", 110).await.unwrap();
        
        // Authenticate
        client.authenticate("testuser", "testpass").await.unwrap();
        
        // Get statistics
        let (count, size) = client.stat().await.unwrap();
        assert!(count >= 0);
        
        // List messages
        let messages = client.list().await.unwrap();
        assert_eq!(messages.len(), count);
        
        // Close
        client.quit().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_smtp_to_pop3_integration() {
        // Send email via SMTP
        let smtp_client = SmtpClient::connect("localhost", 2525).await.unwrap();
        smtp_client.send_email(
            "sender@example.com",
            "recipient@example.com",
            "Test Subject",
            "Test Body"
        ).await.unwrap();
        
        // Wait for delivery
        sleep(Duration::from_millis(100)).await;
        
        // Retrieve via POP3
        let mut pop3_client = Pop3Client::connect("localhost", 110).await.unwrap();
        pop3_client.authenticate("recipient@example.com", "password").await.unwrap();
        
        let (count, _) = pop3_client.stat().await.unwrap();
        assert!(count > 0);
        
        let message = pop3_client.retrieve(count).await.unwrap();
        assert!(message.contains("Test Subject"));
        assert!(message.contains("Test Body"));
    }
}
```

### Mock Testing

```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
pub trait Pop3Storage {
    fn get_message(&self, user: &str, msg_id: usize) -> Result<String, Error>;
    fn list_messages(&self, user: &str) -> Result<Vec<(usize, usize)>, Error>;
    fn delete_message(&self, user: &str, msg_id: usize) -> Result<(), Error>;
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    
    #[test]
    fn test_with_mock_storage() {
        let mut mock = MockPop3Storage::new();
        
        mock.expect_get_message()
            .with(eq("testuser"), eq(1))
            .times(1)
            .returning(|_, _| Ok("From: test@example.com\n\nTest message".to_string()));
        
        mock.expect_list_messages()
            .with(eq("testuser"))
            .times(1)
            .returning(|_| Ok(vec![(1, 100), (2, 200)]));
        
        // Test with mock
        let message = mock.get_message("testuser", 1).unwrap();
        assert!(message.contains("Test message"));
        
        let messages = mock.list_messages("testuser").unwrap();
        assert_eq!(messages.len(), 2);
    }
}
```

## Integration with SMTP and IMAP

### Shared Architecture

```
┌─────────────────────────────────────────────┐
│           Mail Server Architecture           │
├─────────────────────────────────────────────┤
│                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │   SMTP   │  │   POP3   │  │   IMAP   │  │
│  │  Server  │  │  Server  │  │  Server  │  │
│  │ Port 25  │  │ Port 110 │  │ Port 143 │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  │
│       │             │             │         │
│       └─────────────┴─────────────┘         │
│                     │                       │
│         ┌───────────▼───────────┐           │
│         │   Message Storage     │           │
│         │  (Maildir/mbox/DB)   │           │
│         └───────────────────────┘           │
│                     │                       │
│         ┌───────────▼───────────┐           │
│         │  User Authentication  │           │
│         │    (LDAP/DB/File)    │           │
│         └───────────────────────┘           │
└─────────────────────────────────────────────┘
```

### Shared Storage Example

```rust
pub trait MessageStorage: Send + Sync {
    /// Store a new message
    fn store_message(&self, user: &str, message: &[u8]) -> Result<String, Error>;
    
    /// Get a message by ID
    fn get_message(&self, user: &str, message_id: &str) -> Result<Vec<u8>, Error>;
    
    /// List all messages for a user
    fn list_messages(&self, user: &str) -> Result<Vec<MessageInfo>, Error>;
    
    /// Delete a message
    fn delete_message(&self, user: &str, message_id: &str) -> Result<(), Error>;
    
    /// Mark message as seen (for IMAP)
    fn mark_seen(&self, user: &str, message_id: &str) -> Result<(), Error>;
}

// Maildir implementation (works for all protocols)
pub struct MaildirStorage {
    base_path: PathBuf,
}

impl MessageStorage for MaildirStorage {
    fn store_message(&self, user: &str, message: &[u8]) -> Result<String, Error> {
        let user_dir = self.base_path.join(user).join("new");
        std::fs::create_dir_all(&user_dir)?;
        
        let msg_id = format!("{}.{}", SystemTime::now()
            .duration_since(UNIX_EPOCH)?.as_secs(), uuid::Uuid::new_v4());
        
        let msg_path = user_dir.join(&msg_id);
        std::fs::write(msg_path, message)?;
        
        Ok(msg_id)
    }
    
    fn get_message(&self, user: &str, message_id: &str) -> Result<Vec<u8>, Error> {
        let paths = [
            self.base_path.join(user).join("new").join(message_id),
            self.base_path.join(user).join("cur").join(message_id),
        ];
        
        for path in &paths {
            if path.exists() {
                return Ok(std::fs::read(path)?);
            }
        }
        
        Err(Error::MessageNotFound)
    }
    
    // ... other methods
}
```

### User Management Example

```rust
pub trait UserAuthenticator: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> Result<bool, Error>;
    fn user_exists(&self, username: &str) -> Result<bool, Error>;
}

// File-based authentication (simple example)
pub struct FileAuthenticator {
    users_file: PathBuf,
}

impl UserAuthenticator for FileAuthenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<bool, Error> {
        let contents = std::fs::read_to_string(&self.users_file)?;
        
        for line in contents.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 && parts[0] == username {
                // In production, use proper password hashing (bcrypt, argon2, etc.)
                return Ok(parts[1] == password);
            }
        }
        
        Ok(false)
    }
    
    fn user_exists(&self, username: &str) -> Result<bool, Error> {
        let contents = std::fs::read_to_string(&self.users_file)?;
        Ok(contents.lines().any(|line| line.starts_with(username)))
    }
}
```

### Complete Integration Example

```rust
use std::sync::Arc;

pub struct MailServer {
    smtp_server: SmtpServer,
    pop3_server: Pop3Server,
    imap_server: ImapServer,
    storage: Arc<dyn MessageStorage>,
    auth: Arc<dyn UserAuthenticator>,
}

impl MailServer {
    pub fn new(config: MailServerConfig) -> Result<Self, Error> {
        let storage = Arc::new(MaildirStorage::new(&config.maildir_path)?);
        let auth = Arc::new(FileAuthenticator::new(&config.users_file)?);
        
        Ok(MailServer {
            smtp_server: SmtpServer::new(
                config.smtp_config,
                storage.clone(),
                auth.clone()
            ),
            pop3_server: Pop3Server::new(
                config.pop3_config,
                storage.clone(),
                auth.clone()
            ),
            imap_server: ImapServer::new(
                config.imap_config,
                storage.clone(),
                auth.clone()
            ),
            storage,
            auth,
        })
    }
    
    pub async fn start(&self) -> Result<(), Error> {
        // Start all servers concurrently
        tokio::try_join!(
            self.smtp_server.listen(),
            self.pop3_server.listen(),
            self.imap_server.listen(),
        )?;
        
        Ok(())
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MailServerConfig::from_file("mail-server.toml")?;
    let server = MailServer::new(config)?;
    server.start().await?;
    Ok(())
}
```

## Troubleshooting

### Common Issues

#### 1. Connection Refused

**Problem**: Client cannot connect to server

**Possible Causes**:
- Server not running
- Firewall blocking port 110
- Wrong host/port configuration

**Solutions**:
```bash
# Check if server is running
netstat -an | grep :110

# Test connection
telnet localhost 110

# Check firewall
sudo ufw status
sudo ufw allow 110/tcp
```

#### 2. Authentication Failures

**Problem**: `USER` or `PASS` commands return `-ERR`

**Possible Causes**:
- Wrong username/password
- User doesn't exist
- Authentication backend not configured

**Solutions**:
- Verify credentials
- Check server logs
- Test with known-good credentials

```bash
# Check server logs
tail -f /var/log/pop3/server.log

# Test authentication
echo -e "USER testuser\r\nPASS testpass\r\nQUIT\r\n" | nc localhost 110
```

#### 3. Messages Not Found

**Problem**: `RETR` or `DELE` commands fail

**Possible Causes**:
- Invalid message number
- Message already deleted
- Storage backend issues

**Solutions**:
```bash
# List messages first
echo -e "USER testuser\r\nPASS testpass\r\nLIST\r\nQUIT\r\n" | nc localhost 110

# Check mailbox directory
ls -la /var/mail/testuser/new/
ls -la /var/mail/testuser/cur/
```

#### 4. Connection Timeouts

**Problem**: Connection drops after period of inactivity

**Possible Causes**:
- Server timeout too short
- Network issues
- Firewall timeout

**Solutions**:
- Increase server timeout setting
- Send periodic `NOOP` commands
- Adjust firewall settings

#### 5. TLS/SSL Errors

**Problem**: POP3S connection fails

**Possible Causes**:
- Invalid certificate
- Wrong port (should be 995)
- TLS not enabled on server

**Solutions**:
```bash
# Test TLS connection
openssl s_client -connect localhost:995

# Verify certificate
openssl x509 -in /path/to/cert.pem -text -noout
```

### Debug Mode

Enable debug logging:

```rust
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    
    // ... start server
}
```

### Network Debugging

```bash
# Capture POP3 traffic
sudo tcpdump -i any -A port 110

# Monitor connections
watch -n 1 'netstat -an | grep :110'

# Check server process
ps aux | grep pop3
```

## Best Practices

### Security

1. **Use TLS/SSL**: Always use POP3S (port 995) for production
2. **Strong Authentication**: Use secure password hashing (bcrypt, argon2)
3. **Rate Limiting**: Implement login attempt rate limiting
4. **Input Validation**: Validate all client inputs
5. **Timeouts**: Set appropriate connection timeouts

### Performance

1. **Connection Pooling**: Reuse database connections
2. **Caching**: Cache frequently accessed data
3. **Async I/O**: Use async/await for I/O operations
4. **Resource Limits**: Set max connections, message sizes
5. **Indexing**: Index message storage for fast lookups

### Reliability

1. **Error Handling**: Handle all errors gracefully
2. **Logging**: Log all operations for debugging
3. **Monitoring**: Monitor server health and metrics
4. **Backups**: Regular backup of message storage
5. **Testing**: Comprehensive unit and integration tests

### Client Development

1. **Connection Management**: Properly close connections
2. **Error Recovery**: Handle and retry failed operations
3. **Timeouts**: Set reasonable timeouts
4. **Message Handling**: Process messages efficiently
5. **State Tracking**: Track message states (seen, deleted)

### Example: Secure Production Configuration

```toml
[server]
host = "0.0.0.0"
port = 110
timeout = 600

[tls]
enabled = true
port = 995
cert_path = "/etc/ssl/certs/pop3.crt"
key_path = "/etc/ssl/private/pop3.key"
min_tls_version = "1.2"

[security]
max_auth_attempts = 3
auth_timeout = 30
rate_limit_per_ip = 10

[limits]
max_message_size = 52428800  # 50MB
max_connections = 1000
max_messages_per_session = 1000

[logging]
level = "info"
file = "/var/log/pop3/server.log"
max_size = "100MB"
max_age = 30  # days

[authentication]
backend = "database"
db_url = "postgresql://user:pass@localhost/maildb"

[storage]
backend = "maildir"
maildir_path = "/var/mail"
```

## Additional Resources

- [RFC 1939 - POP3 Protocol](https://tools.ietf.org/html/rfc1939)
- [RFC 2595 - POP3 with TLS](https://tools.ietf.org/html/rfc2595)
- [RFC 2449 - POP3 Extension Mechanism](https://tools.ietf.org/html/rfc2449)

## Support

For issues, questions, or contributions:
- GitHub Issues: [Repository Issues](https://github.com/smbcloudXYZ/smbcloud-mail-smtp/issues)
- Documentation: [Project Wiki](https://github.com/smbcloudXYZ/smbcloud-mail-smtp/wiki)
