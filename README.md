# Mail Server

A mail server implementation in Rust with support for SMTP, POP3, and IMAP protocols with TLS/STARTTLS support.

## Features

- ✅ Basic SMTP protocol support (HELO/EHLO, MAIL FROM, RCPT TO, DATA, QUIT)
- ✅ STARTTLS support for secure connections
- ✅ Self-signed certificate generation for TLS
- ✅ Async/await with Tokio runtime
- ✅ Structured logging with tracing
- ✅ Multiple recipient support
- ✅ Command validation and error handling

## Running the Server

```bash
cargo run
```

The server listens on `127.0.0.1:2525` by default.

## Testing

You can test the server using `nc` (netcat) or `telnet`:

```bash
echo -e "EHLO test.com\r\nMAIL FROM: <sender@example.com>\r\nRCPT TO: <recipient@example.com>\r\nDATA\r\nSubject: Test\r\n\r\nTest message\r\n.\r\nQUIT\r\n" | nc localhost 2525
```

## Supported Commands

- `HELO/EHLO` - Initiate SMTP session
- `MAIL FROM` - Specify sender email address
- `RCPT TO` - Specify recipient email address (supports multiple)
- `DATA` - Begin email message content
- `RSET` - Reset the current transaction
- `NOOP` - No operation (keepalive)
- `QUIT` - Close connection
- `STARTTLS` - Upgrade connection to TLS

## Building

```bash
cargo build --release
```

## POP3 Protocol Support

This repository includes comprehensive examples and documentation for the POP3 (Post Office Protocol version 3) implementation.

### POP3 Features

- ✅ USER/PASS authentication
- ✅ STAT command for mailbox statistics
- ✅ LIST command for message listing
- ✅ RETR command for message retrieval
- ✅ DELE command for message deletion
- ✅ RSET command to unmark deletions
- ✅ NOOP keepalive command
- ✅ Optional TLS/SSL support (POP3S)

### POP3 Examples

The repository includes comprehensive POP3 examples:

#### Rust Examples

```bash
# Run the POP3 client example
cargo run --example pop3_client

# Run the POP3 server example
cargo run --example pop3_server

# Run integration tests
cargo run --example pop3_integration_test
```

#### Python Example

```bash
# Run the Python POP3 client
python3 docs/scripts/pop3-client.py
```

#### Manual Testing with netcat

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

### POP3 Documentation

For comprehensive POP3 documentation, see [docs/POP3.md](docs/POP3.md), which includes:

- Protocol basics and command reference
- Setup and configuration guide
- Usage examples for clients and servers
- Unit and integration testing examples
- Integration with SMTP and IMAP
- Troubleshooting guide
- Best practices for security and performance

### POP3 Quick Start

```rust
// Example: Connect to POP3 server and retrieve messages
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("localhost:110")?;
    let mut reader = BufReader::new(stream.try_clone()?);
    
    // Read greeting
    let mut greeting = String::new();
    reader.read_line(&mut greeting)?;
    println!("{}", greeting);
    
    // Authenticate
    writeln!(stream, "USER testuser")?;
    let mut response = String::new();
    reader.read_line(&mut response)?;
    
    writeln!(stream, "PASS testpass")?;
    response.clear();
    reader.read_line(&mut response)?;
    
    // Get mailbox statistics
    writeln!(stream, "STAT")?;
    response.clear();
    reader.read_line(&mut response)?;
    println!("{}", response);
    
    // Close connection
    writeln!(stream, "QUIT")?;
    
    Ok(())
}
```

See [examples/pop3_client.rs](examples/pop3_client.rs) for a complete client implementation with all POP3 commands.

## Dependencies

- `tokio` - Async runtime
- `anyhow` - Error handling
- `tracing` / `tracing-subscriber` - Logging
- `regex` - SMTP command parsing
- `native-tls` / `tokio-native-tls` - TLS support
- `openssl` - Certificate generation
- `futures` - Stream handling
- `tokio-util` - Codec utilities
