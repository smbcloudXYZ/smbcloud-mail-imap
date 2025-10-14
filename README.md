# SMTP and IMAP Mail Server

A basic SMTP and IMAP server implementation in Rust with TLS/STARTTLS support.

## Features

### SMTP
- ✅ Basic SMTP protocol support (HELO/EHLO, MAIL FROM, RCPT TO, DATA, QUIT)
- ✅ STARTTLS support for secure connections
- ✅ Self-signed certificate generation for TLS
- ✅ Multiple recipient support
- ✅ Command validation and error handling
- ✅ Dual-mode operation: separate ports for incoming and outgoing mail

### IMAP
- ✅ IMAP4rev1 protocol implementation
- ✅ Essential commands (LOGIN, SELECT, EXAMINE, FETCH, SEARCH, LIST, STATUS, LOGOUT, CAPABILITY)
- ✅ Mailbox management (INBOX, Sent, Drafts, Trash)
- ✅ Message storage and retrieval
- ✅ UID and sequence number support
- ✅ Message flags (Seen, Answered, Flagged, Deleted, Draft, Recent)
- ✅ Protocol state management

### General
- ✅ Async/await with Tokio runtime
- ✅ Structured logging with tracing
- ✅ Shared message storage between SMTP and IMAP

## Running the Server

```bash
cargo run
```

The SMTP server listens on two ports:
- `127.0.0.1:2525` - **Receive mode**: For incoming mail from other servers (messages stored in INBOX)
- `127.0.0.1:2526` - **Submit mode**: For outgoing mail submission from clients (messages logged and relayed)

The IMAP server listens on `127.0.0.1:1143` by default.

## Testing

### Testing SMTP

You can test the SMTP server using `nc` (netcat) or `telnet`:

**Testing receive mode (port 2525):**
```bash
echo -e "EHLO test.com\r\nMAIL FROM: <sender@example.com>\r\nRCPT TO: <recipient@example.com>\r\nDATA\r\nSubject: Test\r\n\r\nTest message\r\n.\r\nQUIT\r\n" | nc localhost 2525
```

**Testing submit mode (port 2526):**
```bash
echo -e "EHLO test.com\r\nMAIL FROM: <sender@example.com>\r\nRCPT TO: <recipient@example.com>\r\nDATA\r\nSubject: Outgoing Test\r\n\r\nOutgoing message\r\n.\r\nQUIT\r\n" | nc localhost 2526
```

### Testing IMAP

You can test the IMAP server using `nc` (netcat) or `telnet`:

```bash
# Connect and login
echo -e "a1 CAPABILITY\r\na2 LOGIN user pass\r\na3 LIST \"\" \"*\"\r\na4 SELECT INBOX\r\na5 LOGOUT\r\n" | nc localhost 1143

# Fetch messages
echo -e "a1 LOGIN user pass\r\na2 SELECT INBOX\r\na3 FETCH 1 (FLAGS BODY[])\r\na4 LOGOUT\r\n" | nc localhost 1143
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

### IMAP Commands

- `CAPABILITY` - List server capabilities
- `LOGIN` - Authenticate user
- `LIST/LSUB` - List available mailboxes
- `SELECT` - Select a mailbox (read-write)
- `EXAMINE` - Select a mailbox (read-only)
- `FETCH` - Retrieve message data
- `SEARCH` - Search for messages
- `STATUS` - Get mailbox status
- `LOGOUT` - Close connection
- `NOOP` - No operation (keepalive)

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
