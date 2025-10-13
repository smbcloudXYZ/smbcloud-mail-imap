//! POP3 Integration Test Example
//!
//! This example demonstrates how to write integration tests for POP3 servers
//! and clients, showing best practices for testing mail systems.
//!
//! Usage:
//!   cargo run --example pop3_integration_test
//!
//! This example shows:
//! - Setting up test fixtures
//! - Testing complete POP3 workflows
//! - Mocking server responses
//! - Testing error conditions
//! - Integration between SMTP and POP3

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Test result
type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Simple POP3 test client
struct TestPop3Client {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl TestPop3Client {
    fn connect(addr: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let reader = BufReader::new(stream.try_clone()?);
        
        let mut client = TestPop3Client { stream, reader };
        let greeting = client.read_response()?;
        
        if !greeting.starts_with("+OK") {
            return Err(format!("Invalid greeting: {}", greeting).into());
        }
        
        Ok(client)
    }
    
    fn send_command(&mut self, cmd: &str) -> TestResult {
        writeln!(self.stream, "{}", cmd)?;
        self.stream.flush()?;
        Ok(())
    }
    
    fn read_response(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut response = String::new();
        self.reader.read_line(&mut response)?;
        Ok(response.trim_end().to_string())
    }
    
    fn read_multiline(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            let line = line.trim_end().to_string();
            if line == "." {
                break;
            }
            lines.push(line);
        }
        Ok(lines)
    }
}

/// Mock POP3 server for testing
struct MockPop3Server {
    listener: TcpListener,
    running: Arc<Mutex<bool>>,
}

impl MockPop3Server {
    fn new(port: u16) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
        listener.set_nonblocking(true)?;
        
        Ok(MockPop3Server {
            listener,
            running: Arc::new(Mutex::new(true)),
        })
    }
    
    fn start(&self) -> TestResult {
        let listener = self.listener.try_clone()?;
        let running = Arc::clone(&self.running);
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = Self::handle_client(&mut stream);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }
    
    fn handle_client(stream: &mut TcpStream) -> TestResult {
        writeln!(stream, "+OK Mock POP3 server ready")?;
        stream.flush()?;
        
        let mut reader = BufReader::new(stream.try_clone()?);
        
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() {
                break;
            }
            
            let cmd = line.trim().to_uppercase();
            
            let response = match cmd.as_str() {
                s if s.starts_with("USER") => "+OK User accepted",
                s if s.starts_with("PASS") => "+OK Password accepted",
                "STAT" => "+OK 2 640",
                "LIST" => "+OK 2 messages (640 octets)\r\n1 320\r\n2 320\r\n.",
                s if s.starts_with("RETR 1") => {
                    "+OK 320 octets\r\nFrom: test@example.com\r\nSubject: Test\r\n\r\nTest body\r\n."
                }
                s if s.starts_with("DELE") => "+OK Message deleted",
                "RSET" => "+OK Reset",
                "NOOP" => "+OK",
                "QUIT" => {
                    writeln!(stream, "+OK Bye")?;
                    stream.flush()?;
                    break;
                }
                _ => "-ERR Unknown command",
            };
            
            writeln!(stream, "{}", response)?;
            stream.flush()?;
        }
        
        Ok(())
    }
    
    fn stop(&self) {
        *self.running.lock().unwrap() = false;
    }
}

/// Test 1: Basic connection and authentication
fn test_basic_authentication() -> TestResult {
    println!("\n=== Test 1: Basic Authentication ===");
    
    let server = MockPop3Server::new(11001)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut client = TestPop3Client::connect("127.0.0.1:11001")?;
    
    client.send_command("USER testuser")?;
    let response = client.read_response()?;
    assert!(response.starts_with("+OK"), "USER failed: {}", response);
    
    client.send_command("PASS testpass")?;
    let response = client.read_response()?;
    assert!(response.starts_with("+OK"), "PASS failed: {}", response);
    
    client.send_command("QUIT")?;
    let _ = client.read_response();
    
    server.stop();
    
    println!("✓ Authentication test passed");
    Ok(())
}

/// Test 2: Message listing and retrieval
fn test_message_operations() -> TestResult {
    println!("\n=== Test 2: Message Operations ===");
    
    let server = MockPop3Server::new(11002)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut client = TestPop3Client::connect("127.0.0.1:11002")?;
    
    // Authenticate
    client.send_command("USER testuser")?;
    let _ = client.read_response()?;
    client.send_command("PASS testpass")?;
    let _ = client.read_response()?;
    
    // Get statistics
    client.send_command("STAT")?;
    let response = client.read_response()?;
    assert!(response.contains("2 640"), "STAT failed: {}", response);
    
    // List messages
    client.send_command("LIST")?;
    let _ = client.read_response()?;
    let messages = client.read_multiline()?;
    assert!(messages.len() >= 2, "LIST failed: expected 2 messages");
    
    // Retrieve message
    client.send_command("RETR 1")?;
    let _ = client.read_response()?;
    let content = client.read_multiline()?;
    assert!(!content.is_empty(), "RETR failed: empty message");
    
    client.send_command("QUIT")?;
    let _ = client.read_response();
    
    server.stop();
    
    println!("✓ Message operations test passed");
    Ok(())
}

/// Test 3: Message deletion workflow
fn test_deletion_workflow() -> TestResult {
    println!("\n=== Test 3: Deletion Workflow ===");
    
    let server = MockPop3Server::new(11003)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut client = TestPop3Client::connect("127.0.0.1:11003")?;
    
    // Authenticate
    client.send_command("USER testuser")?;
    let _ = client.read_response()?;
    client.send_command("PASS testpass")?;
    let _ = client.read_response()?;
    
    // Delete message
    client.send_command("DELE 1")?;
    let response = client.read_response()?;
    assert!(response.starts_with("+OK"), "DELE failed: {}", response);
    
    // Reset
    client.send_command("RSET")?;
    let response = client.read_response()?;
    assert!(response.starts_with("+OK"), "RSET failed: {}", response);
    
    // Delete again
    client.send_command("DELE 1")?;
    let response = client.read_response()?;
    assert!(response.starts_with("+OK"), "DELE failed: {}", response);
    
    client.send_command("QUIT")?;
    let _ = client.read_response();
    
    server.stop();
    
    println!("✓ Deletion workflow test passed");
    Ok(())
}

/// Test 4: Error handling
fn test_error_handling() -> TestResult {
    println!("\n=== Test 4: Error Handling ===");
    
    // Test connection to non-existent server
    println!("  Testing connection to non-existent server...");
    match TcpStream::connect_timeout(
        &"127.0.0.1:9999".parse().unwrap(),
        Duration::from_secs(1)
    ) {
        Ok(_) => return Err("Should have failed to connect".into()),
        Err(_) => println!("  ✓ Connection refused as expected"),
    }
    
    println!("✓ Error handling test passed");
    Ok(())
}

/// Test 5: Complete session workflow
fn test_complete_session() -> TestResult {
    println!("\n=== Test 5: Complete Session Workflow ===");
    
    let server = MockPop3Server::new(11005)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut client = TestPop3Client::connect("127.0.0.1:11005")?;
    
    // Full workflow
    let commands = vec![
        ("USER testuser", "+OK"),
        ("PASS testpass", "+OK"),
        ("STAT", "+OK"),
        ("LIST", "+OK"),
        ("NOOP", "+OK"),
        ("QUIT", "+OK"),
    ];
    
    for (cmd, expected) in commands {
        client.send_command(cmd)?;
        let response = client.read_response()?;
        
        // Handle multiline responses
        if cmd == "LIST" {
            let _ = client.read_multiline()?;
        }
        
        assert!(
            response.starts_with(expected),
            "{} failed: expected '{}', got '{}'",
            cmd, expected, response
        );
        
        if cmd == "QUIT" {
            break;
        }
    }
    
    server.stop();
    
    println!("✓ Complete session test passed");
    Ok(())
}

/// Test 6: Concurrent connections
fn test_concurrent_connections() -> TestResult {
    println!("\n=== Test 6: Concurrent Connections ===");
    
    let server = MockPop3Server::new(11006)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut handles = vec![];
    
    for i in 0..5 {
        let handle = thread::spawn(move || -> TestResult {
            let mut client = TestPop3Client::connect("127.0.0.1:11006")?;
            
            client.send_command("USER testuser")?;
            let _ = client.read_response()?;
            client.send_command("PASS testpass")?;
            let _ = client.read_response()?;
            client.send_command("STAT")?;
            let _ = client.read_response()?;
            client.send_command("QUIT")?;
            let _ = client.read_response();
            
            println!("  ✓ Client {} completed", i + 1);
            Ok(())
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap()?;
    }
    
    server.stop();
    
    println!("✓ Concurrent connections test passed");
    Ok(())
}

/// Test 7: Command sequencing
fn test_command_sequencing() -> TestResult {
    println!("\n=== Test 7: Command Sequencing ===");
    
    let server = MockPop3Server::new(11007)?;
    server.start()?;
    
    thread::sleep(Duration::from_millis(100));
    
    let mut client = TestPop3Client::connect("127.0.0.1:11007")?;
    
    // Test proper sequence
    client.send_command("USER testuser")?;
    let _ = client.read_response()?;
    client.send_command("PASS testpass")?;
    let _ = client.read_response()?;
    
    // Commands that require authentication
    let commands = vec!["STAT", "LIST", "NOOP"];
    for cmd in commands {
        client.send_command(cmd)?;
        let response = client.read_response()?;
        if cmd == "LIST" {
            let _ = client.read_multiline()?;
        }
        assert!(response.starts_with("+OK"), "{} failed: {}", cmd, response);
    }
    
    client.send_command("QUIT")?;
    let _ = client.read_response();
    
    server.stop();
    
    println!("✓ Command sequencing test passed");
    Ok(())
}

fn main() {
    println!("========================================");
    println!("POP3 Integration Test Examples");
    println!("========================================");
    println!("\nRunning comprehensive integration tests...\n");
    
    let tests: Vec<(&str, fn() -> TestResult)> = vec![
        ("Basic Authentication", test_basic_authentication),
        ("Message Operations", test_message_operations),
        ("Deletion Workflow", test_deletion_workflow),
        ("Error Handling", test_error_handling),
        ("Complete Session", test_complete_session),
        ("Concurrent Connections", test_concurrent_connections),
        ("Command Sequencing", test_command_sequencing),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (name, test_fn) in tests {
        match test_fn() {
            Ok(_) => {
                passed += 1;
            }
            Err(e) => {
                println!("✗ Test '{}' failed: {}", name, e);
                failed += 1;
            }
        }
    }
    
    println!("\n========================================");
    println!("Test Results");
    println!("========================================");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);
    
    if failed == 0 {
        println!("\n🎉 All tests passed!");
    } else {
        println!("\n⚠️  Some tests failed!");
    }
}
