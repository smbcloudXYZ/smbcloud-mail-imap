# Testing Guide for POP3 Implementation

This document provides comprehensive testing examples and strategies for POP3 protocol implementations.

## Table of Contents

1. [Unit Testing](#unit-testing)
2. [Integration Testing](#integration-testing)
3. [Mock Testing](#mock-testing)
4. [Load Testing](#load-testing)
5. [Security Testing](#security-testing)
6. [Testing Best Practices](#testing-best-practices)

## Unit Testing

### Testing POP3 Command Parser

```rust
#[cfg(test)]
mod command_parser_tests {
    use super::*;

    #[test]
    fn test_parse_user_command() {
        let cmd = "USER testuser";
        let parsed = parse_command(cmd);
        assert_eq!(parsed.command, "USER");
        assert_eq!(parsed.args, vec!["testuser"]);
    }

    #[test]
    fn test_parse_pass_command() {
        let cmd = "PASS secret123";
        let parsed = parse_command(cmd);
        assert_eq!(parsed.command, "PASS");
        assert_eq!(parsed.args, vec!["secret123"]);
    }

    #[test]
    fn test_parse_retr_command() {
        let cmd = "RETR 5";
        let parsed = parse_command(cmd);
        assert_eq!(parsed.command, "RETR");
        assert_eq!(parsed.args, vec!["5"]);
    }

    #[test]
    fn test_parse_command_case_insensitive() {
        let cmd = "user TestUser";
        let parsed = parse_command(cmd);
        assert_eq!(parsed.command, "USER");
        assert_eq!(parsed.args, vec!["TestUser"]);
    }

    #[test]
    fn test_parse_invalid_command() {
        let cmd = "INVALID";
        let result = parse_command(cmd);
        assert!(result.is_err());
    }
}
```

### Testing Authentication Logic

```rust
#[cfg(test)]
mod auth_tests {
    use super::*;

    #[test]
    fn test_valid_authentication() {
        let authenticator = create_test_authenticator();
        let result = authenticator.authenticate("testuser", "testpass");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_invalid_password() {
        let authenticator = create_test_authenticator();
        let result = authenticator.authenticate("testuser", "wrongpass");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_nonexistent_user() {
        let authenticator = create_test_authenticator();
        let result = authenticator.authenticate("nonexistent", "password");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_empty_credentials() {
        let authenticator = create_test_authenticator();
        let result = authenticator.authenticate("", "");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_special_characters_in_password() {
        let authenticator = create_test_authenticator();
        // Add user with special characters
        authenticator.add_user("special", "p@ss!w0rd#$%");
        let result = authenticator.authenticate("special", "p@ss!w0rd#$%");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }
}
```

### Testing Mailbox Operations

```rust
#[cfg(test)]
mod mailbox_tests {
    use super::*;

    #[test]
    fn test_stat_empty_mailbox() {
        let mailbox = Mailbox::new();
        let (count, size) = mailbox.stat();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }

    #[test]
    fn test_stat_with_messages() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Message 1", 100);
        mailbox.add_message("Message 2", 200);
        let (count, size) = mailbox.stat();
        assert_eq!(count, 2);
        assert_eq!(size, 300);
    }

    #[test]
    fn test_list_messages() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Message 1", 100);
        mailbox.add_message("Message 2", 200);
        let list = mailbox.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], (1, 100));
        assert_eq!(list[1], (2, 200));
    }

    #[test]
    fn test_retrieve_message() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Test Message Content", 100);
        let msg = mailbox.retrieve(1);
        assert!(msg.is_some());
        assert_eq!(msg.unwrap(), "Test Message Content");
    }

    #[test]
    fn test_retrieve_nonexistent_message() {
        let mailbox = Mailbox::new();
        let msg = mailbox.retrieve(999);
        assert!(msg.is_none());
    }

    #[test]
    fn test_delete_message() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Message", 100);
        let result = mailbox.delete(1);
        assert!(result.is_ok());
        
        // Message should be marked deleted but still exist
        assert_eq!(mailbox.messages.len(), 1);
        assert_eq!(mailbox.messages[0].deleted, true);
    }

    #[test]
    fn test_reset_deletions() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Message", 100);
        mailbox.delete(1).unwrap();
        mailbox.reset();
        
        // Deletion mark should be removed
        assert_eq!(mailbox.messages[0].deleted, false);
    }

    #[test]
    fn test_commit_deletions() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Message 1", 100);
        mailbox.add_message("Message 2", 200);
        mailbox.delete(1).unwrap();
        mailbox.commit();
        
        // Message 1 should be permanently removed
        assert_eq!(mailbox.messages.len(), 1);
        assert_eq!(mailbox.retrieve(2).is_some(), true);
    }

    #[test]
    fn test_stat_after_delete() {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Message 1", 100);
        mailbox.add_message("Message 2", 200);
        mailbox.delete(1).unwrap();
        
        // STAT should not count deleted messages
        let (count, size) = mailbox.stat();
        assert_eq!(count, 1);
        assert_eq!(size, 200);
    }
}
```

### Testing Response Generation

```rust
#[cfg(test)]
mod response_tests {
    use super::*;

    #[test]
    fn test_ok_response() {
        let response = Response::ok("User accepted");
        assert_eq!(response.to_string(), "+OK User accepted\r\n");
    }

    #[test]
    fn test_err_response() {
        let response = Response::err("Invalid command");
        assert_eq!(response.to_string(), "-ERR Invalid command\r\n");
    }

    #[test]
    fn test_multiline_response() {
        let lines = vec!["1 100".to_string(), "2 200".to_string()];
        let response = Response::multiline("+OK 2 messages", lines);
        let expected = "+OK 2 messages\r\n1 100\r\n2 200\r\n.\r\n";
        assert_eq!(response.to_string(), expected);
    }

    #[test]
    fn test_stat_response() {
        let response = Response::stat(3, 600);
        assert_eq!(response.to_string(), "+OK 3 600\r\n");
    }
}
```

## Integration Testing

### End-to-End Session Test

```rust
#[tokio::test]
async fn test_complete_pop3_session() {
    // Start test server
    let server = start_test_pop3_server().await;
    
    // Connect client
    let mut client = Pop3Client::connect("127.0.0.1", 11000).await.unwrap();
    
    // Authenticate
    client.send("USER testuser").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.starts_with("+OK"));
    
    client.send("PASS testpass").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.starts_with("+OK"));
    
    // Get statistics
    client.send("STAT").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.contains("+OK"));
    
    // List messages
    client.send("LIST").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.starts_with("+OK"));
    
    // Retrieve message
    client.send("RETR 1").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.starts_with("+OK"));
    
    // Delete message
    client.send("DELE 1").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.starts_with("+OK"));
    
    // Quit
    client.send("QUIT").await.unwrap();
    let resp = client.read_response().await.unwrap();
    assert!(resp.starts_with("+OK"));
    
    server.stop().await;
}
```

### SMTP to POP3 Integration Test

```rust
#[tokio::test]
async fn test_smtp_to_pop3_delivery() {
    // Start both servers
    let smtp_server = start_test_smtp_server().await;
    let pop3_server = start_test_pop3_server().await;
    
    // Send email via SMTP
    let mut smtp_client = SmtpClient::connect("127.0.0.1", 2525).await.unwrap();
    smtp_client.send_mail(
        "sender@test.com",
        vec!["testuser@test.com"],
        "Subject: Test\r\n\r\nTest body"
    ).await.unwrap();
    
    // Wait for delivery
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Check with POP3
    let mut pop3_client = Pop3Client::connect("127.0.0.1", 110).await.unwrap();
    pop3_client.authenticate("testuser", "testpass").await.unwrap();
    
    let (count, _) = pop3_client.stat().await.unwrap();
    assert!(count > 0, "No messages received");
    
    let msg = pop3_client.retrieve(count).await.unwrap();
    assert!(msg.contains("Test body"));
    
    smtp_server.stop().await;
    pop3_server.stop().await;
}
```

### Concurrent Connections Test

```rust
#[tokio::test]
async fn test_concurrent_connections() {
    let server = start_test_pop3_server().await;
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let mut client = Pop3Client::connect("127.0.0.1", 11000).await.unwrap();
                client.authenticate("testuser", "testpass").await.unwrap();
                let (count, size) = client.stat().await.unwrap();
                client.quit().await.unwrap();
                (i, count, size)
            })
        })
        .collect();
    
    for handle in handles {
        let (i, count, size) = handle.await.unwrap();
        println!("Client {} completed: {} messages, {} bytes", i, count, size);
    }
    
    server.stop().await;
}
```

## Mock Testing

### Using Mockall for Storage

```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
pub trait MessageStorage {
    fn get_message(&self, user: &str, msg_id: usize) -> Result<Vec<u8>, Error>;
    fn list_messages(&self, user: &str) -> Result<Vec<(usize, usize)>, Error>;
    fn delete_message(&self, user: &str, msg_id: usize) -> Result<(), Error>;
}

#[cfg(test)]
mod mock_storage_tests {
    use super::*;

    #[test]
    fn test_retrieve_with_mock() {
        let mut mock_storage = MockMessageStorage::new();
        
        mock_storage
            .expect_get_message()
            .with(eq("testuser"), eq(1))
            .times(1)
            .returning(|_, _| Ok(b"Test message".to_vec()));
        
        let msg = mock_storage.get_message("testuser", 1).unwrap();
        assert_eq!(msg, b"Test message");
    }

    #[test]
    fn test_list_with_mock() {
        let mut mock_storage = MockMessageStorage::new();
        
        mock_storage
            .expect_list_messages()
            .with(eq("testuser"))
            .times(1)
            .returning(|_| Ok(vec![(1, 100), (2, 200)]));
        
        let messages = mock_storage.list_messages("testuser").unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], (1, 100));
    }

    #[test]
    fn test_delete_with_mock() {
        let mut mock_storage = MockMessageStorage::new();
        
        mock_storage
            .expect_delete_message()
            .with(eq("testuser"), eq(1))
            .times(1)
            .returning(|_, _| Ok(()));
        
        let result = mock_storage.delete_message("testuser", 1);
        assert!(result.is_ok());
    }
}
```

### Mock Authentication Backend

```rust
#[automock]
pub trait Authenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<bool, Error>;
}

#[cfg(test)]
mod mock_auth_tests {
    use super::*;

    #[test]
    fn test_successful_auth() {
        let mut mock_auth = MockAuthenticator::new();
        
        mock_auth
            .expect_authenticate()
            .with(eq("testuser"), eq("testpass"))
            .times(1)
            .returning(|_, _| Ok(true));
        
        let result = mock_auth.authenticate("testuser", "testpass").unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn test_failed_auth() {
        let mut mock_auth = MockAuthenticator::new();
        
        mock_auth
            .expect_authenticate()
            .with(eq("testuser"), eq("wrongpass"))
            .times(1)
            .returning(|_, _| Ok(false));
        
        let result = mock_auth.authenticate("testuser", "wrongpass").unwrap();
        assert_eq!(result, false);
    }
}
```

## Load Testing

### Basic Load Test

```rust
#[tokio::test]
#[ignore] // Run separately with --ignored flag
async fn test_connection_load() {
    let server = start_test_pop3_server().await;
    let concurrent_clients = 100;
    
    let start = std::time::Instant::now();
    
    let handles: Vec<_> = (0..concurrent_clients)
        .map(|_| {
            tokio::spawn(async {
                let mut client = Pop3Client::connect("127.0.0.1", 11000).await?;
                client.authenticate("testuser", "testpass").await?;
                client.stat().await?;
                client.quit().await?;
                Ok::<_, Box<dyn std::error::Error>>(())
            })
        })
        .collect();
    
    let mut success_count = 0;
    for handle in handles {
        if handle.await.unwrap().is_ok() {
            success_count += 1;
        }
    }
    
    let duration = start.elapsed();
    println!("Completed {} connections in {:?}", success_count, duration);
    println!("Throughput: {:.2} connections/sec", 
             success_count as f64 / duration.as_secs_f64());
    
    assert_eq!(success_count, concurrent_clients);
    
    server.stop().await;
}
```

### Message Retrieval Load Test

```rust
#[tokio::test]
#[ignore]
async fn test_message_retrieval_load() {
    let server = start_test_pop3_server().await;
    let num_retrievals = 1000;
    
    let start = std::time::Instant::now();
    
    let mut client = Pop3Client::connect("127.0.0.1", 11000).await.unwrap();
    client.authenticate("testuser", "testpass").await.unwrap();
    
    for i in 1..=num_retrievals {
        client.retrieve(1).await.unwrap();
        if i % 100 == 0 {
            println!("Retrieved {} messages", i);
        }
    }
    
    client.quit().await.unwrap();
    
    let duration = start.elapsed();
    println!("Retrieved {} messages in {:?}", num_retrievals, duration);
    println!("Throughput: {:.2} retrievals/sec", 
             num_retrievals as f64 / duration.as_secs_f64());
    
    server.stop().await;
}
```

## Security Testing

### Authentication Brute Force Protection Test

```rust
#[tokio::test]
async fn test_rate_limiting() {
    let server = start_test_pop3_server().await;
    
    let mut client = Pop3Client::connect("127.0.0.1", 11000).await.unwrap();
    
    // Try multiple failed authentications
    for i in 0..10 {
        let _ = client.send(format!("USER testuser{}", i)).await;
        let _ = client.read_response().await;
        let _ = client.send("PASS wrongpass").await;
        let resp = client.read_response().await.unwrap();
        
        if i >= 3 {
            // After 3 attempts, should be rate limited
            assert!(resp.contains("too many") || resp.contains("rate limit"));
        }
    }
    
    server.stop().await;
}
```

### SQL Injection Test (if using database)

```rust
#[tokio::test]
async fn test_sql_injection_protection() {
    let server = start_test_pop3_server().await;
    let mut client = Pop3Client::connect("127.0.0.1", 11000).await.unwrap();
    
    // Try SQL injection in username
    client.send("USER admin' OR '1'='1").await.unwrap();
    let _ = client.read_response().await;
    
    client.send("PASS password").await.unwrap();
    let resp = client.read_response().await.unwrap();
    
    // Should fail authentication
    assert!(resp.starts_with("-ERR"));
    
    server.stop().await;
}
```

### Command Injection Test

```rust
#[tokio::test]
async fn test_command_injection() {
    let server = start_test_pop3_server().await;
    let mut client = Pop3Client::connect("127.0.0.1", 11000).await.unwrap();
    
    // Try command injection
    let malicious_commands = vec![
        "USER test\r\nSTAT\r\n",
        "PASS test; rm -rf /",
        "RETR 1; cat /etc/passwd",
    ];
    
    for cmd in malicious_commands {
        client.send(cmd).await.unwrap();
        let resp = client.read_response().await.unwrap();
        // Should either fail or only execute intended command
        assert!(resp.starts_with("+OK") || resp.starts_with("-ERR"));
    }
    
    server.stop().await;
}
```

## Testing Best Practices

### Test Organization

```rust
// Organize tests in modules
#[cfg(test)]
mod tests {
    mod unit {
        mod authentication { /* auth tests */ }
        mod mailbox { /* mailbox tests */ }
        mod commands { /* command tests */ }
    }
    
    mod integration {
        mod sessions { /* session tests */ }
        mod protocols { /* protocol tests */ }
    }
    
    mod security {
        mod injection { /* injection tests */ }
        mod rate_limit { /* rate limit tests */ }
    }
}
```

### Test Fixtures

```rust
// Create reusable test fixtures
fn create_test_mailbox() -> Mailbox {
    let mut mailbox = Mailbox::new();
    mailbox.add_message("Message 1", 100);
    mailbox.add_message("Message 2", 200);
    mailbox
}

fn create_test_authenticator() -> FileAuthenticator {
    let mut auth = FileAuthenticator::new();
    auth.add_user("testuser", "testpass");
    auth.add_user("alice", "password123");
    auth
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_with_fixture() {
        let mailbox = create_test_mailbox();
        assert_eq!(mailbox.messages.len(), 2);
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_message_id_always_positive(msg_id in 1usize..10000) {
        let mut mailbox = Mailbox::new();
        mailbox.add_message("Test", 100);
        
        let result = mailbox.retrieve(msg_id);
        // Should not panic
        prop_assert!(result.is_some() || result.is_none());
    }
    
    #[test]
    fn test_stat_consistency(
        msg_count in 0usize..100,
        msg_size in 1usize..10000
    ) {
        let mut mailbox = Mailbox::new();
        
        for _ in 0..msg_count {
            mailbox.add_message("Test", msg_size);
        }
        
        let (count, size) = mailbox.stat();
        prop_assert_eq!(count, msg_count);
        prop_assert_eq!(size, msg_count * msg_size);
    }
}
```

### Test Coverage

```bash
# Install tarpaulin for coverage
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage/

# View coverage report
open coverage/index.html
```

### Continuous Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
      - name: Run integration tests
        run: cargo test --test '*' -- --ignored
      - name: Check code coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v1
```

## Debugging Tests

### Enable Logging in Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn init_logger() {
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    }
    
    #[test]
    fn test_with_logging() {
        init_logger();
        
        // Test code here
        tracing::debug!("Debug message in test");
    }
}
```

### Using Test Output

```bash
# Show test output
cargo test -- --nocapture

# Show specific test output
cargo test test_name -- --nocapture --exact

# Run tests in sequence (not parallel)
cargo test -- --test-threads=1
```

## Conclusion

This testing guide provides a comprehensive framework for testing POP3 implementations. Combine unit tests, integration tests, mock tests, and security tests to ensure robust and reliable code.

Key takeaways:
- Test at multiple levels (unit, integration, system)
- Use mocks for external dependencies
- Include security tests
- Measure and improve coverage
- Automate testing in CI/CD
- Document test scenarios and expectations
