//! POP3 Server Example
//!
//! This example demonstrates how to set up and run a basic POP3 server
//! that can handle client connections, authentication, and message operations.
//!
//! Usage:
//!   cargo run --example pop3_server
//!
//! This example shows:
//! - Setting up a POP3 server
//! - Handling client connections
//! - Processing POP3 commands
//! - Managing mailbox state
//! - Proper error handling

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// Represents a mail message
#[derive(Clone, Debug)]
struct Message {
    id: usize,
    size: usize,
    content: String,
    deleted: bool,
}

/// User mailbox
#[derive(Clone, Debug)]
struct Mailbox {
    messages: Vec<Message>,
}

impl Mailbox {
    fn new() -> Self {
        Mailbox {
            messages: Vec::new(),
        }
    }
    
    fn add_sample_messages(&mut self) {
        // Add some sample messages for testing
        self.messages.push(Message {
            id: 1,
            size: 250,
            content: "From: sender1@example.com\r\nTo: user@example.com\r\nSubject: Test Message 1\r\n\r\nThis is the first test message.\r\n".to_string(),
            deleted: false,
        });
        
        self.messages.push(Message {
            id: 2,
            size: 300,
            content: "From: sender2@example.com\r\nTo: user@example.com\r\nSubject: Test Message 2\r\n\r\nThis is the second test message with more content.\r\n".to_string(),
            deleted: false,
        });
        
        self.messages.push(Message {
            id: 3,
            size: 180,
            content: "From: sender3@example.com\r\nTo: user@example.com\r\nSubject: Short Message\r\n\r\nBrief message.\r\n".to_string(),
            deleted: false,
        });
    }
    
    fn stat(&self) -> (usize, usize) {
        let count = self.messages.iter().filter(|m| !m.deleted).count();
        let size = self.messages.iter()
            .filter(|m| !m.deleted)
            .map(|m| m.size)
            .sum();
        (count, size)
    }
    
    fn list_all(&self) -> Vec<(usize, usize)> {
        self.messages.iter()
            .filter(|m| !m.deleted)
            .map(|m| (m.id, m.size))
            .collect()
    }
    
    fn get_message(&self, id: usize) -> Option<&Message> {
        self.messages.iter()
            .find(|m| m.id == id && !m.deleted)
    }
    
    fn delete_message(&mut self, id: usize) -> bool {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id && !m.deleted) {
            msg.deleted = true;
            true
        } else {
            false
        }
    }
    
    fn reset(&mut self) {
        for msg in &mut self.messages {
            msg.deleted = false;
        }
    }
    
    fn commit_deletions(&mut self) {
        self.messages.retain(|m| !m.deleted);
    }
}

/// POP3 server state
struct Pop3Server {
    users: Arc<Mutex<HashMap<String, String>>>, // username -> password
    mailboxes: Arc<Mutex<HashMap<String, Mailbox>>>, // username -> mailbox
}

impl Pop3Server {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("testuser".to_string(), "testpass".to_string());
        users.insert("alice".to_string(), "password123".to_string());
        users.insert("bob".to_string(), "secret".to_string());
        
        let mut mailboxes = HashMap::new();
        
        // Create sample mailbox for testuser
        let mut testuser_mailbox = Mailbox::new();
        testuser_mailbox.add_sample_messages();
        mailboxes.insert("testuser".to_string(), testuser_mailbox);
        
        // Create sample mailbox for alice
        let mut alice_mailbox = Mailbox::new();
        alice_mailbox.add_sample_messages();
        mailboxes.insert("alice".to_string(), alice_mailbox);
        
        // Create empty mailbox for bob
        mailboxes.insert("bob".to_string(), Mailbox::new());
        
        Pop3Server {
            users: Arc::new(Mutex::new(users)),
            mailboxes: Arc::new(Mutex::new(mailboxes)),
        }
    }
    
    fn handle_client(&self, mut stream: TcpStream) {
        println!("New client connected from {}", stream.peer_addr().unwrap());
        
        // Send greeting
        if let Err(e) = writeln!(stream, "+OK POP3 server ready") {
            eprintln!("Error sending greeting: {}", e);
            return;
        }
        
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut authenticated_user: Option<String> = None;
        let mut pending_user: Option<String> = None;
        
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break, // Connection closed
                Ok(_) => {
                    let line = line.trim();
                    println!("< {}", line);
                    
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    let command = parts[0].to_uppercase();
                    let args = if parts.len() > 1 { parts[1] } else { "" };
                    
                    let response = match command.as_str() {
                        "USER" => self.handle_user(args, &mut pending_user),
                        "PASS" => self.handle_pass(args, &pending_user, &mut authenticated_user),
                        "STAT" => self.handle_stat(&authenticated_user),
                        "LIST" => self.handle_list(&authenticated_user, args),
                        "RETR" => self.handle_retr(&authenticated_user, args),
                        "DELE" => self.handle_dele(&authenticated_user, args),
                        "NOOP" => self.handle_noop(&authenticated_user),
                        "RSET" => self.handle_rset(&authenticated_user),
                        "QUIT" => {
                            let resp = self.handle_quit(&mut authenticated_user);
                            if let Err(e) = writeln!(stream, "{}", resp) {
                                eprintln!("Error sending response: {}", e);
                            }
                            break;
                        }
                        "CAPA" => self.handle_capa(),
                        _ => "-ERR Unknown command".to_string(),
                    };
                    
                    println!("> {}", response.lines().next().unwrap_or(""));
                    if let Err(e) = write!(stream, "{}", response) {
                        eprintln!("Error sending response: {}", e);
                        break;
                    }
                    
                    if let Err(e) = stream.flush() {
                        eprintln!("Error flushing stream: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from client: {}", e);
                    break;
                }
            }
        }
        
        println!("Client disconnected");
    }
    
    fn handle_user(&self, username: &str, pending_user: &mut Option<String>) -> String {
        if username.is_empty() {
            return "-ERR Missing username\r\n".to_string();
        }
        
        let users = self.users.lock().unwrap();
        if users.contains_key(username) {
            *pending_user = Some(username.to_string());
            "+OK User accepted\r\n".to_string()
        } else {
            "-ERR User not found\r\n".to_string()
        }
    }
    
    fn handle_pass(
        &self,
        password: &str,
        pending_user: &Option<String>,
        authenticated_user: &mut Option<String>,
    ) -> String {
        let username = match pending_user {
            Some(u) => u,
            None => return "-ERR Send USER first\r\n".to_string(),
        };
        
        let users = self.users.lock().unwrap();
        if let Some(stored_pass) = users.get(username) {
            if stored_pass == password {
                *authenticated_user = Some(username.clone());
                return "+OK Password accepted\r\n".to_string();
            }
        }
        
        "-ERR Invalid password\r\n".to_string()
    }
    
    fn handle_stat(&self, authenticated_user: &Option<String>) -> String {
        let username = match authenticated_user {
            Some(u) => u,
            None => return "-ERR Not authenticated\r\n".to_string(),
        };
        
        let mailboxes = self.mailboxes.lock().unwrap();
        if let Some(mailbox) = mailboxes.get(username) {
            let (count, size) = mailbox.stat();
            format!("+OK {} {}\r\n", count, size)
        } else {
            "-ERR Mailbox not found\r\n".to_string()
        }
    }
    
    fn handle_list(&self, authenticated_user: &Option<String>, args: &str) -> String {
        let username = match authenticated_user {
            Some(u) => u,
            None => return "-ERR Not authenticated\r\n".to_string(),
        };
        
        let mailboxes = self.mailboxes.lock().unwrap();
        let mailbox = match mailboxes.get(username) {
            Some(mb) => mb,
            None => return "-ERR Mailbox not found\r\n".to_string(),
        };
        
        if args.is_empty() {
            // List all messages
            let (count, size) = mailbox.stat();
            let mut response = format!("+OK {} messages ({} octets)\r\n", count, size);
            for (id, size) in mailbox.list_all() {
                response.push_str(&format!("{} {}\r\n", id, size));
            }
            response.push_str(".\r\n");
            response
        } else {
            // List specific message
            if let Ok(id) = args.parse::<usize>() {
                if let Some(msg) = mailbox.get_message(id) {
                    format!("+OK {} {}\r\n", id, msg.size)
                } else {
                    "-ERR No such message\r\n".to_string()
                }
            } else {
                "-ERR Invalid message number\r\n".to_string()
            }
        }
    }
    
    fn handle_retr(&self, authenticated_user: &Option<String>, args: &str) -> String {
        let username = match authenticated_user {
            Some(u) => u,
            None => return "-ERR Not authenticated\r\n".to_string(),
        };
        
        if args.is_empty() {
            return "-ERR Missing message number\r\n".to_string();
        }
        
        let id = match args.parse::<usize>() {
            Ok(id) => id,
            Err(_) => return "-ERR Invalid message number\r\n".to_string(),
        };
        
        let mailboxes = self.mailboxes.lock().unwrap();
        let mailbox = match mailboxes.get(username) {
            Some(mb) => mb,
            None => return "-ERR Mailbox not found\r\n".to_string(),
        };
        
        if let Some(msg) = mailbox.get_message(id) {
            format!("+OK {} octets\r\n{}\r\n.\r\n", msg.size, msg.content)
        } else {
            "-ERR No such message\r\n".to_string()
        }
    }
    
    fn handle_dele(&self, authenticated_user: &Option<String>, args: &str) -> String {
        let username = match authenticated_user {
            Some(u) => u,
            None => return "-ERR Not authenticated\r\n".to_string(),
        };
        
        if args.is_empty() {
            return "-ERR Missing message number\r\n".to_string();
        }
        
        let id = match args.parse::<usize>() {
            Ok(id) => id,
            Err(_) => return "-ERR Invalid message number\r\n".to_string(),
        };
        
        let mut mailboxes = self.mailboxes.lock().unwrap();
        let mailbox = match mailboxes.get_mut(username) {
            Some(mb) => mb,
            None => return "-ERR Mailbox not found\r\n".to_string(),
        };
        
        if mailbox.delete_message(id) {
            format!("+OK Message {} deleted\r\n", id)
        } else {
            "-ERR No such message\r\n".to_string()
        }
    }
    
    fn handle_noop(&self, authenticated_user: &Option<String>) -> String {
        if authenticated_user.is_none() {
            return "-ERR Not authenticated\r\n".to_string();
        }
        "+OK\r\n".to_string()
    }
    
    fn handle_rset(&self, authenticated_user: &Option<String>) -> String {
        let username = match authenticated_user {
            Some(u) => u,
            None => return "-ERR Not authenticated\r\n".to_string(),
        };
        
        let mut mailboxes = self.mailboxes.lock().unwrap();
        if let Some(mailbox) = mailboxes.get_mut(username) {
            mailbox.reset();
            let (count, size) = mailbox.stat();
            format!("+OK Maildrop has {} messages ({} octets)\r\n", count, size)
        } else {
            "-ERR Mailbox not found\r\n".to_string()
        }
    }
    
    fn handle_quit(&self, authenticated_user: &mut Option<String>) -> String {
        if let Some(username) = authenticated_user.take() {
            let mut mailboxes = self.mailboxes.lock().unwrap();
            if let Some(mailbox) = mailboxes.get_mut(&username) {
                mailbox.commit_deletions();
            }
        }
        "+OK POP3 server signing off\r\n".to_string()
    }
    
    fn handle_capa(&self) -> String {
        let mut response = "+OK Capability list follows\r\n".to_string();
        response.push_str("USER\r\n");
        response.push_str("UIDL\r\n");
        response.push_str("TOP\r\n");
        response.push_str(".\r\n");
        response
    }
}

fn main() {
    println!("========================================");
    println!("POP3 Server Example");
    println!("========================================\n");
    
    let server = Arc::new(Pop3Server::new());
    
    let listener = match TcpListener::bind("127.0.0.1:110") {
        Ok(listener) => {
            println!("POP3 server listening on 127.0.0.1:110");
            println!("\nTest users:");
            println!("  - testuser / testpass (has 3 messages)");
            println!("  - alice / password123 (has 3 messages)");
            println!("  - bob / secret (empty mailbox)");
            println!("\nPress Ctrl+C to stop the server\n");
            listener
        }
        Err(e) => {
            eprintln!("Failed to bind to port 110: {}", e);
            eprintln!("\nNote: Port 110 requires root/administrator privileges.");
            eprintln!("Try running with sudo, or modify the code to use port 1110.");
            return;
        }
    };
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let server = Arc::clone(&server);
                thread::spawn(move || {
                    server.handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
