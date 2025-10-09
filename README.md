# SMTP Server

A basic SMTP server implementation in Rust with TLS/STARTTLS support.

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

## Dependencies

- `tokio` - Async runtime
- `anyhow` - Error handling
- `tracing` / `tracing-subscriber` - Logging
- `regex` - SMTP command parsing
- `native-tls` / `tokio-native-tls` - TLS support
- `openssl` - Certificate generation
- `futures` - Stream handling
- `tokio-util` - Codec utilities
