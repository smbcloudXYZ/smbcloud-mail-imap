//! POP3 Client Example
//!
//! This example demonstrates how to connect to a POP3 server and perform
//! common operations such as authentication, listing messages, retrieving
//! messages, and deleting messages.
//!
//! Usage:
//!   cargo run --example pop3_client
//!
//! This example shows:
//! - Connecting to a POP3 server
//! - Authenticating with USER and PASS commands
//! - Getting mailbox statistics with STAT
//! - Listing messages with LIST
//! - Retrieving messages with RETR
//! - Deleting messages with DELE
//! - Properly closing connection with QUIT

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

/// Represents a POP3 client connection
struct Pop3Client {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
}

impl Pop3Client {
    /// Connect to a POP3 server
    fn connect(host: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Connecting to {}:{}...", host, port);
        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        let reader = BufReader::new(stream.try_clone()?);
        
        let mut client = Pop3Client { stream, reader };
        
        // Read the server greeting
        let greeting = client.read_response()?;
        println!("Server greeting: {}", greeting);
        
        if !greeting.starts_with("+OK") {
            return Err(format!("Server did not send +OK greeting: {}", greeting).into());
        }
        
        Ok(client)
    }
    
    /// Send a command to the server
    fn send_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("> {}", command);
        writeln!(self.stream, "{}", command)?;
        self.stream.flush()?;
        Ok(())
    }
    
    /// Read a single-line response from the server
    fn read_response(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let mut response = String::new();
        self.reader.read_line(&mut response)?;
        let response = response.trim_end().to_string();
        println!("< {}", response);
        Ok(response)
    }
    
    /// Read a multi-line response from the server (ends with ".")
    fn read_multiline_response(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut lines = Vec::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line)?;
            let line = line.trim_end().to_string();
            println!("< {}", line);
            
            if line == "." {
                break;
            }
            lines.push(line);
        }
        Ok(lines)
    }
    
    /// Authenticate with username and password
    fn authenticate(&mut self, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Authenticating ===");
        
        // Send USER command
        self.send_command(&format!("USER {}", username))?;
        let response = self.read_response()?;
        if !response.starts_with("+OK") {
            return Err(format!("USER command failed: {}", response).into());
        }
        
        // Send PASS command
        self.send_command(&format!("PASS {}", password))?;
        let response = self.read_response()?;
        if !response.starts_with("+OK") {
            return Err(format!("PASS command failed: {}", response).into());
        }
        
        println!("Authentication successful!");
        Ok(())
    }
    
    /// Get mailbox statistics (STAT command)
    fn stat(&mut self) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        println!("\n=== Getting mailbox statistics ===");
        
        self.send_command("STAT")?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("STAT command failed: {}", response).into());
        }
        
        // Parse response: +OK <message_count> <mailbox_size>
        let parts: Vec<&str> = response.split_whitespace().collect();
        if parts.len() >= 3 {
            let message_count = parts[1].parse::<usize>()?;
            let mailbox_size = parts[2].parse::<usize>()?;
            println!("Mailbox has {} messages ({} bytes)", message_count, mailbox_size);
            Ok((message_count, mailbox_size))
        } else {
            Err("Invalid STAT response format".into())
        }
    }
    
    /// List all messages (LIST command)
    fn list(&mut self) -> Result<Vec<(usize, usize)>, Box<dyn std::error::Error>> {
        println!("\n=== Listing all messages ===");
        
        self.send_command("LIST")?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("LIST command failed: {}", response).into());
        }
        
        let lines = self.read_multiline_response()?;
        let mut messages = Vec::new();
        
        for line in lines {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let msg_num = parts[0].parse::<usize>()?;
                let msg_size = parts[1].parse::<usize>()?;
                messages.push((msg_num, msg_size));
                println!("  Message {}: {} bytes", msg_num, msg_size);
            }
        }
        
        Ok(messages)
    }
    
    /// List a specific message (LIST command with message number)
    fn list_message(&mut self, msg_num: usize) -> Result<usize, Box<dyn std::error::Error>> {
        println!("\n=== Listing message {} ===", msg_num);
        
        self.send_command(&format!("LIST {}", msg_num))?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("LIST command failed: {}", response).into());
        }
        
        // Parse response: +OK <msg_num> <msg_size>
        let parts: Vec<&str> = response.split_whitespace().collect();
        if parts.len() >= 3 {
            let msg_size = parts[2].parse::<usize>()?;
            println!("Message {} size: {} bytes", msg_num, msg_size);
            Ok(msg_size)
        } else {
            Err("Invalid LIST response format".into())
        }
    }
    
    /// Retrieve a message (RETR command)
    fn retrieve(&mut self, msg_num: usize) -> Result<String, Box<dyn std::error::Error>> {
        println!("\n=== Retrieving message {} ===", msg_num);
        
        self.send_command(&format!("RETR {}", msg_num))?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("RETR command failed: {}", response).into());
        }
        
        let lines = self.read_multiline_response()?;
        let message = lines.join("\n");
        println!("Retrieved message ({} lines)", lines.len());
        
        Ok(message)
    }
    
    /// Delete a message (DELE command)
    fn delete(&mut self, msg_num: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Deleting message {} ===", msg_num);
        
        self.send_command(&format!("DELE {}", msg_num))?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("DELE command failed: {}", response).into());
        }
        
        println!("Message {} marked for deletion", msg_num);
        Ok(())
    }
    
    /// Reset the session (RSET command) - unmarks deleted messages
    fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Resetting session ===");
        
        self.send_command("RSET")?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("RSET command failed: {}", response).into());
        }
        
        println!("Session reset - all delete marks removed");
        Ok(())
    }
    
    /// No-op command (NOOP) - keeps connection alive
    fn noop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Sending NOOP ===");
        
        self.send_command("NOOP")?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("NOOP command failed: {}", response).into());
        }
        
        println!("NOOP successful");
        Ok(())
    }
    
    /// Close the connection (QUIT command)
    fn quit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Closing connection ===");
        
        self.send_command("QUIT")?;
        let response = self.read_response()?;
        
        if !response.starts_with("+OK") {
            return Err(format!("QUIT command failed: {}", response).into());
        }
        
        println!("Connection closed successfully");
        Ok(())
    }
}

/// Example 1: Complete email retrieval session
fn example_complete_session() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n========================================");
    println!("EXAMPLE 1: Complete Email Retrieval Session");
    println!("========================================\n");
    
    let mut client = Pop3Client::connect("localhost", 110)?;
    
    // Authenticate
    client.authenticate("testuser", "testpass")?;
    
    // Get mailbox statistics
    let (message_count, _mailbox_size) = client.stat()?;
    
    // List all messages
    let _messages = client.list()?;
    
    // Retrieve first message if available
    if message_count > 0 {
        let message_content = client.retrieve(1)?;
        println!("\nFirst message preview:");
        println!("---");
        for (i, line) in message_content.lines().take(10).enumerate() {
            println!("{}", line);
            if i == 9 && message_content.lines().count() > 10 {
                println!("... (truncated)");
            }
        }
        println!("---");
    }
    
    // Close connection
    client.quit()?;
    
    Ok(())
}

/// Example 2: Handling multiple messages
fn example_multiple_messages() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n========================================");
    println!("EXAMPLE 2: Handling Multiple Messages");
    println!("========================================\n");
    
    let mut client = Pop3Client::connect("localhost", 110)?;
    client.authenticate("testuser", "testpass")?;
    
    // Get all messages
    let messages = client.list()?;
    
    println!("\nProcessing {} messages:", messages.len());
    for (msg_num, msg_size) in messages {
        println!("\nProcessing message {} ({} bytes):", msg_num, msg_size);
        
        // Retrieve each message
        match client.retrieve(msg_num) {
            Ok(content) => {
                println!("  Successfully retrieved {} bytes", content.len());
                // Process message content here
            }
            Err(e) => {
                println!("  Error retrieving message: {}", e);
            }
        }
    }
    
    client.quit()?;
    Ok(())
}

/// Example 3: Message deletion workflow
fn example_deletion_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n========================================");
    println!("EXAMPLE 3: Message Deletion Workflow");
    println!("========================================\n");
    
    let mut client = Pop3Client::connect("localhost", 110)?;
    client.authenticate("testuser", "testpass")?;
    
    let (message_count, _) = client.stat()?;
    
    if message_count > 0 {
        // Mark first message for deletion
        client.delete(1)?;
        
        // Change our mind - reset to unmark
        client.reset()?;
        
        // Verify message is still there
        let messages = client.list()?;
        println!("\nAfter reset, mailbox still has {} messages", messages.len());
        
        // Delete again (this time for real)
        client.delete(1)?;
        
        // Quit to commit the deletion
        println!("\nNote: Deletion will be committed when we QUIT");
    } else {
        println!("No messages to delete");
    }
    
    client.quit()?;
    Ok(())
}

/// Example 4: Error handling
fn example_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n========================================");
    println!("EXAMPLE 4: Error Handling");
    println!("========================================\n");
    
    // Example: Authentication failure
    println!("--- Testing authentication failure ---");
    match Pop3Client::connect("localhost", 110) {
        Ok(mut client) => {
            match client.authenticate("wronguser", "wrongpass") {
                Ok(_) => println!("Unexpected: Authentication succeeded"),
                Err(e) => println!("Expected error: {}", e),
            }
            let _ = client.quit();
        }
        Err(e) => {
            println!("Connection failed (expected if server not running): {}", e);
        }
    }
    
    // Example: Invalid message number
    println!("\n--- Testing invalid message number ---");
    match Pop3Client::connect("localhost", 110) {
        Ok(mut client) => {
            if client.authenticate("testuser", "testpass").is_ok() {
                match client.retrieve(99999) {
                    Ok(_) => println!("Unexpected: Retrieved non-existent message"),
                    Err(e) => println!("Expected error: {}", e),
                }
                let _ = client.quit();
            }
        }
        Err(e) => {
            println!("Connection failed (expected if server not running): {}", e);
        }
    }
    
    Ok(())
}

/// Example 5: Connection error scenarios
fn example_connection_errors() {
    println!("\n========================================");
    println!("EXAMPLE 5: Connection Error Scenarios");
    println!("========================================\n");
    
    // Try to connect to non-existent server
    println!("--- Testing connection to non-existent server ---");
    match Pop3Client::connect("localhost", 9999) {
        Ok(_) => println!("Unexpected: Connection succeeded"),
        Err(e) => println!("Expected error: {}", e),
    }
    
    // Try to connect with invalid hostname
    println!("\n--- Testing connection to invalid hostname ---");
    match Pop3Client::connect("invalid.example.nonexistent", 110) {
        Ok(_) => println!("Unexpected: Connection succeeded"),
        Err(e) => println!("Expected error: {}", e),
    }
}

fn main() {
    println!("===========================================");
    println!("POP3 Client Examples");
    println!("===========================================");
    println!("\nThis example demonstrates various POP3 operations.");
    println!("Note: These examples assume a POP3 server is running on localhost:110");
    println!("      with user 'testuser' and password 'testpass'.");
    println!("\nIf the server is not running, you'll see connection errors,");
    println!("which is normal and demonstrates error handling.\n");
    
    // Run all examples
    // Note: In real usage, you'd typically only run one session at a time
    
    if let Err(e) = example_complete_session() {
        println!("Example 1 error: {}", e);
    }
    
    if let Err(e) = example_multiple_messages() {
        println!("Example 2 error: {}", e);
    }
    
    if let Err(e) = example_deletion_workflow() {
        println!("Example 3 error: {}", e);
    }
    
    if let Err(e) = example_error_handling() {
        println!("Example 4 error: {}", e);
    }
    
    example_connection_errors();
    
    println!("\n===========================================");
    println!("Examples completed!");
    println!("===========================================");
}
