# IMAP Implementation Documentation

This document describes the IMAP (Internet Message Access Protocol) implementation in the smbcloud-mail server.

## Overview

The IMAP implementation provides a complete RFC 3501-compliant IMAP4rev1 server that works alongside the existing SMTP implementation. Messages received via SMTP are stored in mailboxes that can be accessed through IMAP.

## Architecture

### Core Components

1. **Message Module** (`src/imap/message.rs`)
   - `Message`: Represents an email message with metadata
   - `MessageFlag`: Enum for standard IMAP flags (Seen, Answered, Flagged, Deleted, Draft, Recent)
   - Supports UID and sequence number addressing
   - Parses subject and body from raw email content

2. **Mailbox Module** (`src/imap/mailbox.rs`)
   - `Mailbox`: Container for messages with thread-safe access
   - `MailboxStore`: Central storage for multiple mailboxes
   - Default mailboxes: INBOX, Sent, Drafts, Trash
   - Provides message counting (exists, recent, unseen)
   - UID management with uidvalidity and uidnext

3. **State Module** (`src/imap/state.rs`)
   - `ImapState`: State machine for IMAP protocol
   - States: NotAuthenticated, Authenticated, Selected, Logout
   - Enforces proper command sequencing

4. **Commands Module** (`src/imap/commands.rs`)
   - `ImapCommand`: Parser for IMAP commands
   - `ImapResponse`: Formatter for IMAP responses
   - `ImapCommandHandler`: Command execution logic
   - Implements all essential IMAP commands

5. **Session Module** (`src/imap/session.rs`)
   - `ImapSession`: Async session handler
   - Manages client connections
   - Processes commands and generates responses
   - Uses Tokio's async I/O

6. **Listen Module** (`src/imap/listen.rs`)
   - TCP listener for IMAP connections
   - Spawns session handlers for each connection
   - Runs on port 1143 by default

7. **Storage Module** (`src/storage.rs`)
   - `MessageStorage`: Bridge between SMTP and IMAP
   - Shared mailbox store
   - Thread-safe message storage

## Implemented Commands

### Authentication Commands
- **CAPABILITY**: Lists server capabilities
- **LOGIN**: Authenticates user (accepts any credentials for demo)
- **LOGOUT**: Closes connection

### Mailbox Commands
- **LIST/LSUB**: Lists available mailboxes
- **SELECT**: Selects a mailbox in read-write mode
- **EXAMINE**: Selects a mailbox in read-only mode
- **STATUS**: Gets mailbox status without selecting

### Message Commands
- **FETCH**: Retrieves message data (FLAGS, UID, BODY, ENVELOPE, etc.)
- **SEARCH**: Searches for messages (simplified implementation)
- **NOOP**: No operation (keepalive)

## Protocol State Machine

```
┌─────────────────────┐
│  NotAuthenticated   │
└──────────┬──────────┘
           │ LOGIN
           ▼
┌─────────────────────┐
│   Authenticated     │
└──────────┬──────────┘
           │ SELECT/EXAMINE
           ▼
┌─────────────────────┐
│     Selected        │
└──────────┬──────────┘
           │ LOGOUT
           ▼
┌─────────────────────┐
│      Logout         │
└─────────────────────┘
```

## Integration with SMTP

1. **Shared Storage**: Both SMTP and IMAP use the same `MessageStorage` instance
2. **Message Flow**: 
   - Email received via SMTP → Stored in `MessageStorage` → Accessible via IMAP INBOX
3. **Concurrent Operation**: SMTP (port 2525) and IMAP (port 1143) run simultaneously

## Usage Examples

### Basic IMAP Session
```bash
# Connect and list mailboxes
printf "a1 LOGIN user pass\r\na2 LIST \"\" \"*\"\r\na3 LOGOUT\r\n" | nc localhost 1143
```

### Check INBOX Status
```bash
printf "a1 LOGIN user pass\r\na2 STATUS INBOX (MESSAGES RECENT UNSEEN)\r\na3 LOGOUT\r\n" | nc localhost 1143
```

### Read Messages
```bash
printf "a1 LOGIN user pass\r\na2 SELECT INBOX\r\na3 FETCH 1:* (FLAGS BODY[])\r\na4 LOGOUT\r\n" | nc localhost 1143
```

## Testing

### Run Unit Tests
```bash
cargo test
```

### Run Example Client
```bash
cargo run --example imap_client
```

### Manual Testing
1. Start the server: `cargo run`
2. Connect with netcat: `nc localhost 1143`
3. Send IMAP commands interactively

## Limitations and Future Enhancements

### Current Limitations
- No persistent storage (messages lost on restart)
- Simplified authentication (accepts any credentials)
- No SSL/TLS support for IMAP
- Limited SEARCH criteria support
- No support for mailbox hierarchy separators
- No IDLE, APPEND, or COPY commands

### Potential Enhancements
- Persistent storage (database or file system)
- Real authentication system
- STARTTLS/SSL support for IMAP
- Full SEARCH command support
- IDLE command for push notifications
- APPEND command for storing messages
- COPY/MOVE commands for message management
- Quota management
- Access control lists (ACLs)

## Security Considerations

1. **Authentication**: Current implementation accepts any credentials - implement proper authentication for production
2. **Encryption**: No TLS support yet - add STARTTLS for secure connections
3. **Input Validation**: Basic validation implemented - add more robust checks
4. **Resource Limits**: No limits on mailbox size or connection count - add limits for production

## Performance Notes

- Thread-safe using `Arc<Mutex<>>` for shared state
- Async I/O with Tokio for efficient connection handling
- Messages stored in memory for fast access
- Supports concurrent SMTP and IMAP operations

## Compliance

The implementation follows RFC 3501 (IMAP4rev1) for core functionality:
- Protocol state management ✓
- Command syntax and responses ✓
- Message flags ✓
- UID support ✓
- Mailbox operations ✓

Not yet implemented:
- Full SEARCH criteria
- All FETCH data items
- Mailbox subscriptions
- Message sequence number updates after expunge

## Contributing

When extending the IMAP implementation:
1. Follow existing code patterns
2. Add unit tests for new functionality
3. Update this documentation
4. Test with standard IMAP clients (Thunderbird, Mail.app, etc.)
5. Ensure thread safety for concurrent access
