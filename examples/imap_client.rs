use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use std::time::Duration;

/// Simple example demonstrating IMAP client interaction
/// 
/// NOTE: This example uses placeholder credentials ('username'/'password').
/// In production, always use secure authentication methods.
#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Connecting to IMAP server at 127.0.0.1:1143...");
    
    let stream = TcpStream::connect("127.0.0.1:1143").await?;
    let mut reader = BufReader::new(stream);
    
    // Read greeting
    let greeting = read_response(&mut reader).await?;
    println!("Server: {}", greeting);
    
    // Send CAPABILITY command
    send_command(&mut reader, b"a1 CAPABILITY\r\n").await?;
    
    // Login with placeholder credentials
    // WARNING: Never hardcode real credentials - use environment variables or config files
    send_command(&mut reader, b"a2 LOGIN username password\r\n").await?;
    
    // List mailboxes
    send_command(&mut reader, b"a3 LIST \"\" \"*\"\r\n").await?;
    
    // Select INBOX
    send_command(&mut reader, b"a4 SELECT INBOX\r\n").await?;
    
    // Get status of INBOX
    send_command(&mut reader, b"a5 STATUS INBOX (MESSAGES RECENT UNSEEN)\r\n").await?;
    
    // Search all messages
    send_command(&mut reader, b"a6 SEARCH ALL\r\n").await?;
    
    // Logout
    send_command(&mut reader, b"a7 LOGOUT\r\n").await?;
    
    println!("IMAP session completed successfully!");
    Ok(())
}

/// Helper function to send a command and read the complete response
async fn send_command(reader: &mut BufReader<TcpStream>, command: &[u8]) -> std::io::Result<()> {
    reader.get_mut().write_all(command).await?;
    let response = read_response(reader).await?;
    println!("Server: {}", response);
    Ok(())
}

/// Helper function to read a complete IMAP response
/// This handles multi-line responses by reading until we see a tagged completion response
async fn read_response(reader: &mut BufReader<TcpStream>) -> std::io::Result<String> {
    let mut response = String::new();
    let mut buffer = vec![0u8; 1024];
    
    // Read data with a timeout to handle the streaming nature of IMAP
    loop {
        tokio::select! {
            result = reader.read(&mut buffer) => {
                let n = result?;
                if n == 0 {
                    break;
                }
                response.push_str(&String::from_utf8_lossy(&buffer[..n]));
                
                // Check if we have a complete response (ends with a tagged response line)
                if response.contains(" OK ") || response.contains(" NO ") || response.contains(" BAD ") {
                    break;
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Timeout - assume response is complete
                if !response.is_empty() {
                    break;
                }
            }
        }
    }
    
    Ok(response.trim_end().to_string())
}
