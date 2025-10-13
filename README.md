# Mail Server (SMTP + POP3)

A basic mail server implementation in Rust with SMTP and POP3 protocol support, featuring TLS/STARTTLS support.

## Features

### SMTP Server
- ✅ Basic SMTP protocol support (HELO/EHLO, MAIL FROM, RCPT TO, DATA, QUIT)
- ✅ STARTTLS support for secure connections
- ✅ Self-signed certificate generation for TLS
- ✅ Multiple recipient support
- ✅ Command validation and error handling

### POP3 Server
- ✅ Complete POP3 protocol implementation (RFC 1939)
- ✅ User authentication (USER/PASS commands)
- ✅ Message retrieval (STAT, LIST, RETR commands)
- ✅ Message deletion (DELE, RSET commands)
- ✅ Session management (QUIT command with update state)
- ✅ Capability advertisement (CAPA command)
- ✅ Keep-alive support (NOOP command)

### General
- ✅ Async/await with Tokio runtime
- ✅ Structured logging with tracing
- ✅ Shared message storage between protocols
- ✅ Concurrent server operation

## Running the Server

```bash
cargo run
```

The servers listen on:
- SMTP: `127.0.0.1:2525`
- POP3: `127.0.0.1:2110`

## Testing

### SMTP Testing

You can test the SMTP server using `nc` (netcat) or `telnet`:

```bash
echo -e "EHLO test.com\r\nMAIL FROM: <sender@example.com>\r\nRCPT TO: <recipient@example.com>\r\nDATA\r\nSubject: Test\r\n\r\nTest message\r\n.\r\nQUIT\r\n" | nc localhost 2525
```

### POP3 Testing

Test POP3 commands:

```bash
# List messages
echo -e "USER test\r\nPASS password\r\nLIST\r\nQUIT\r\n" | nc localhost 2110

# Retrieve a message
echo -e "USER test\r\nPASS password\r\nRETR 1\r\nQUIT\r\n" | nc localhost 2110

# Check mailbox statistics
echo -e "USER test\r\nPASS password\r\nSTAT\r\nQUIT\r\n" | nc localhost 2110

# Delete a message
echo -e "USER test\r\nPASS password\r\nDELE 1\r\nQUIT\r\n" | nc localhost 2110
```

## Supported Commands

### SMTP Commands
- `HELO/EHLO` - Initiate SMTP session
- `MAIL FROM` - Specify sender email address
- `RCPT TO` - Specify recipient email address (supports multiple)
- `DATA` - Begin email message content
- `RSET` - Reset the current transaction
- `NOOP` - No operation (keepalive)
- `QUIT` - Close connection
- `STARTTLS` - Upgrade connection to TLS

### POP3 Commands
- `USER` - Specify username for authentication
- `PASS` - Provide password for authentication
- `STAT` - Get mailbox statistics (message count and total size)
- `LIST` - List messages with sizes (all or specific message)
- `RETR` - Retrieve a specific message
- `DELE` - Mark a message for deletion
- `NOOP` - No operation (keep-alive)
- `RSET` - Reset session (unmark deleted messages)
- `QUIT` - End session and apply deletions
- `CAPA` - List server capabilities

## Building

```bash
cargo build --release
```

## Dependencies

- `tokio` - Async runtime
- `anyhow` - Error handling
- `tracing` / `tracing-subscriber` - Logging
- `regex` - SMTP command parsing
- `native-tls` / `tokio-native-tls` - TLS support
- `openssl` - Certificate generation
- `futures` - Stream handling
- `tokio-util` - Codec utilities
