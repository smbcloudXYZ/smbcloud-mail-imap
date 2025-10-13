use std::io::{Read, Write};
use std::net::TcpStream;

/// Simple example demonstrating IMAP client interaction
fn main() -> std::io::Result<()> {
    println!("Connecting to IMAP server at 127.0.0.1:1143...");
    
    let mut stream = TcpStream::connect("127.0.0.1:1143")?;
    let mut buffer = [0; 1024];
    
    // Read greeting
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // Send CAPABILITY command
    stream.write_all(b"a1 CAPABILITY\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // Login
    stream.write_all(b"a2 LOGIN testuser testpass\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // List mailboxes
    stream.write_all(b"a3 LIST \"\" \"*\"\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // Select INBOX
    stream.write_all(b"a4 SELECT INBOX\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // Get status of INBOX
    stream.write_all(b"a5 STATUS INBOX (MESSAGES RECENT UNSEEN)\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // Search all messages
    stream.write_all(b"a6 SEARCH ALL\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    // Logout
    stream.write_all(b"a7 LOGOUT\r\n")?;
    let n = stream.read(&mut buffer)?;
    println!("Server: {}", String::from_utf8_lossy(&buffer[..n]));
    
    println!("IMAP session completed successfully!");
    Ok(())
}
