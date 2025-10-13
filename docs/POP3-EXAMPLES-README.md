# POP3 Examples and Documentation - Quick Reference

This document provides a quick reference to all POP3 examples and documentation in this repository.

## 📚 Documentation Files

### Main Documentation
- **[POP3.md](POP3.md)** - Comprehensive POP3 protocol documentation
  - Protocol basics and command reference
  - Setup and configuration guide
  - Integration with SMTP and IMAP
  - Troubleshooting guide
  - Best practices

### Configuration
- **[config-examples.md](config-examples.md)** - Configuration examples
  - Basic configurations for development
  - Production configurations with TLS
  - Database and LDAP authentication
  - Docker and cluster configurations
  - Environment-specific settings

### Testing
- **[TESTING.md](TESTING.md)** - Comprehensive testing guide
  - Unit testing examples
  - Integration testing patterns
  - Mock testing strategies
  - Load testing examples
  - Security testing
  - CI/CD setup

### Scenarios
- **[SCENARIOS.md](SCENARIOS.md)** - Real-world usage scenarios
  - Basic email retrieval
  - Download and delete workflows
  - Selective message management
  - Mobile client scenarios
  - Email backup solutions
  - Multi-device synchronization
  - Email migration
  - Automated processing
  - Error recovery

## 💻 Code Examples

### Rust Examples

#### 1. POP3 Client (`examples/pop3_client.rs`)
Complete POP3 client implementation demonstrating:
- Connection to POP3 server
- USER/PASS authentication
- STAT command (mailbox statistics)
- LIST command (message listing)
- RETR command (message retrieval)
- DELE command (message deletion)
- RSET command (reset deletions)
- NOOP command (keepalive)
- QUIT command (close connection)
- Error handling for all operations

**Run it:**
```bash
cargo run --example pop3_client
```

**Example output:**
```
===========================================
POP3 Client Examples
===========================================

This example demonstrates various POP3 operations.
Note: These examples assume a POP3 server is running on localhost:110

EXAMPLE 1: Complete Email Retrieval Session
========================================
Connecting to localhost:110...
```

#### 2. POP3 Server (`examples/pop3_server.rs`)
Mock POP3 server implementation for testing:
- Multi-threaded connection handling
- User authentication
- Mailbox management
- All standard POP3 commands
- Sample test data

**Run it:**
```bash
cargo run --example pop3_server
```

**Features:**
- Pre-configured test users (testuser/testpass, alice/password123, bob/secret)
- Sample messages for testing
- Proper POP3 protocol responses
- Transaction state management

#### 3. Integration Tests (`examples/pop3_integration_test.rs`)
Comprehensive integration test suite with:
- Basic authentication tests
- Message operation tests
- Deletion workflow tests
- Error handling tests
- Complete session tests
- Concurrent connection tests
- Command sequencing tests

**Run it:**
```bash
cargo run --example pop3_integration_test
```

**Test Coverage:**
- ✅ 7 integration test scenarios
- ✅ Mock server implementation
- ✅ Concurrent client testing
- ✅ Error condition testing

### Python Examples

#### POP3 Client (`docs/scripts/pop3-client.py`)
Python implementation using `poplib` library:
- 9 complete example scenarios
- All major POP3 operations
- Email parsing with `email` library
- Secure POP3S support
- Comprehensive error handling

**Run it:**
```bash
python3 docs/scripts/pop3-client.py
```

**Examples included:**
1. Basic connection and authentication
2. List all messages
3. Retrieve and parse message
4. Retrieve all messages
5. Delete messages
6. TOP command (headers only)
7. UIDL command (unique IDs)
8. Error handling scenarios
9. Secure POP3S connection

## 🚀 Quick Start

### 1. Basic POP3 Client Usage

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let mut stream = TcpStream::connect("localhost:110")?;
    let mut reader = BufReader::new(stream.try_clone()?);
    
    // Read greeting
    let mut line = String::new();
    reader.read_line(&mut line)?;
    println!("{}", line);
    
    // Authenticate
    writeln!(stream, "USER testuser")?;
    reader.read_line(&mut line)?;
    
    writeln!(stream, "PASS testpass")?;
    line.clear();
    reader.read_line(&mut line)?;
    
    // Get statistics
    writeln!(stream, "STAT")?;
    line.clear();
    reader.read_line(&mut line)?;
    println!("Mailbox: {}", line);
    
    // Close
    writeln!(stream, "QUIT")?;
    
    Ok(())
}
```

### 2. Using netcat for Manual Testing

```bash
# Connect to POP3 server
nc localhost 110

# Then type these commands:
USER testuser
PASS testpass
STAT
LIST
RETR 1
DELE 1
QUIT
```

### 3. Python Quick Start

```python
import poplib

# Connect and authenticate
pop3 = poplib.POP3("localhost", 110)
pop3.user("testuser")
pop3.pass_("testpass")

# Get statistics
message_count, mailbox_size = pop3.stat()
print(f"{message_count} messages, {mailbox_size} bytes")

# List messages
response, message_list, octets = pop3.list()
for msg in message_list:
    print(msg.decode())

# Retrieve message
response, lines, octets = pop3.retr(1)
message = b'\n'.join(lines)
print(message.decode())

# Close
pop3.quit()
```

## 📋 Command Reference

### Authorization State
| Command | Description | Example |
|---------|-------------|---------|
| `USER` | Username | `USER john@example.com` |
| `PASS` | Password | `PASS secretpassword` |

### Transaction State
| Command | Description | Example |
|---------|-------------|---------|
| `STAT` | Get statistics | `STAT` → `+OK 5 3200` |
| `LIST` | List messages | `LIST` or `LIST 1` |
| `RETR` | Retrieve message | `RETR 1` |
| `DELE` | Mark for deletion | `DELE 1` |
| `NOOP` | No operation | `NOOP` |
| `RSET` | Reset deletions | `RSET` |
| `QUIT` | Close connection | `QUIT` |

## 🔧 Configuration Examples

### Basic Development Setup

```toml
[server]
host = "127.0.0.1"
port = 110

[authentication]
backend = "file"
users_file = "users.txt"

[storage]
backend = "maildir"
maildir_path = "./mailboxes"
```

### Production Setup with TLS

```toml
[server]
host = "0.0.0.0"
port = 995  # POP3S

[tls]
enabled = true
cert_path = "/etc/ssl/certs/mail.crt"
key_path = "/etc/ssl/private/mail.key"

[authentication]
backend = "database"
db_url = "postgresql://user:pass@localhost/maildb"

[storage]
backend = "maildir"
maildir_path = "/var/mail"
```

## 🧪 Testing

### Run All Tests
```bash
# Compile examples
cargo build --examples

# Run integration tests
cargo run --example pop3_integration_test

# Run Python examples
python3 docs/scripts/pop3-client.py
```

### Manual Testing with Server
```bash
# Terminal 1: Start server
cargo run --example pop3_server

# Terminal 2: Connect with client
cargo run --example pop3_client
```

## 📊 Test Coverage

- **Unit Tests**: Command parsing, authentication, mailbox operations
- **Integration Tests**: Complete sessions, concurrent connections, error handling
- **Security Tests**: Rate limiting, injection protection, authentication
- **Load Tests**: Concurrent connections, message retrieval performance

## 🔐 Security Best Practices

1. **Always use TLS/SSL in production** (POP3S on port 995)
2. **Use strong password hashing** (argon2, bcrypt)
3. **Implement rate limiting** for authentication attempts
4. **Validate all inputs** to prevent injection attacks
5. **Set appropriate timeouts** to prevent resource exhaustion
6. **Monitor and log** all authentication attempts
7. **Keep certificates up to date**

## 🐛 Troubleshooting

### Connection Refused
```bash
# Check if server is running
netstat -an | grep :110

# Test connection
telnet localhost 110
```

### Authentication Fails
```bash
# Check server logs
tail -f /var/log/pop3/server.log

# Test with known-good credentials
echo -e "USER testuser\r\nPASS testpass\r\nQUIT\r\n" | nc localhost 110
```

### Messages Not Found
```bash
# List messages first
echo -e "USER testuser\r\nPASS testpass\r\nLIST\r\nQUIT\r\n" | nc localhost 110

# Check mailbox directory
ls -la /var/mail/testuser/new/
```

## 📖 Additional Resources

- **RFC 1939** - POP3 Protocol Specification
- **RFC 2595** - Using TLS with POP3
- **RFC 2449** - POP3 Extension Mechanism

## 🤝 Contributing

When adding new examples or documentation:

1. Follow existing code style and structure
2. Add comprehensive comments
3. Include error handling
4. Provide working examples
5. Update this README
6. Add tests for new functionality

## 📝 License

See the main repository LICENSE file for license information.

---

**Need help?** Check the detailed documentation files or open an issue on GitHub.
