# POP3 Usage Scenarios and Workflows

This document provides practical, real-world scenarios demonstrating how to use the POP3 implementation in various situations.

## Table of Contents

1. [Basic Email Retrieval](#basic-email-retrieval)
2. [Download and Delete Workflow](#download-and-delete-workflow)
3. [Selective Message Management](#selective-message-management)
4. [Mobile Email Client](#mobile-email-client)
5. [Email Backup Solution](#email-backup-solution)
6. [Multi-Device Synchronization](#multi-device-synchronization)
7. [Email Migration](#email-migration)
8. [Automated Email Processing](#automated-email-processing)
9. [Error Recovery Scenarios](#error-recovery-scenarios)

## Basic Email Retrieval

### Scenario: Simple Email Client

A user wants to retrieve and read their emails using a simple email client.

```rust
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

fn retrieve_emails(username: &str, password: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Connect to server
    let mut stream = TcpStream::connect("mail.example.com:110")?;
    let mut reader = BufReader::new(stream.try_clone()?);
    
    // Read greeting
    let mut line = String::new();
    reader.read_line(&mut line)?;
    println!("Server: {}", line.trim());
    
    // Authenticate
    writeln!(stream, "USER {}", username)?;
    line.clear();
    reader.read_line(&mut line)?;
    
    writeln!(stream, "PASS {}", password)?;
    line.clear();
    reader.read_line(&mut line)?;
    
    if !line.starts_with("+OK") {
        return Err("Authentication failed".into());
    }
    
    // Get message count
    writeln!(stream, "STAT")?;
    line.clear();
    reader.read_line(&mut line)?;
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    let message_count: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    
    println!("You have {} message(s)", message_count);
    
    // Retrieve all messages
    let mut messages = Vec::new();
    
    for i in 1..=message_count {
        writeln!(stream, "RETR {}", i)?;
        line.clear();
        reader.read_line(&mut line)?;
        
        if line.starts_with("+OK") {
            let mut message = String::new();
            loop {
                line.clear();
                reader.read_line(&mut line)?;
                if line.trim() == "." {
                    break;
                }
                message.push_str(&line);
            }
            messages.push(message);
        }
    }
    
    // Close connection
    writeln!(stream, "QUIT")?;
    
    Ok(messages)
}

fn main() {
    match retrieve_emails("user@example.com", "password") {
        Ok(messages) => {
            for (i, msg) in messages.iter().enumerate() {
                println!("\n=== Message {} ===", i + 1);
                // Parse and display message
                display_message(msg);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn display_message(message: &str) {
    // Parse headers
    let mut in_headers = true;
    for line in message.lines() {
        if line.is_empty() {
            in_headers = false;
            continue;
        }
        
        if in_headers {
            if line.starts_with("From:") || line.starts_with("Subject:") || line.starts_with("Date:") {
                println!("{}", line);
            }
        } else {
            println!("{}", line);
        }
    }
}
```

## Download and Delete Workflow

### Scenario: Traditional POP3 Client

User wants to download emails to their local device and remove them from the server.

```rust
fn download_and_delete_emails(username: &str, password: &str, save_dir: &str) 
    -> Result<(), Box<dyn std::error::Error>> 
{
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let (message_count, _) = client.stat()?;
    println!("Downloading {} message(s)...", message_count);
    
    for i in 1..=message_count {
        // Retrieve message
        let message = client.retrieve(i)?;
        
        // Save to file
        let filename = format!("{}/message_{}.eml", save_dir, i);
        std::fs::write(&filename, message)?;
        println!("Saved message {} to {}", i, filename);
        
        // Delete from server
        client.delete(i)?;
        println!("Deleted message {} from server", i);
    }
    
    // Commit deletions and close
    client.quit()?;
    
    println!("Download complete. {} messages saved and deleted.", message_count);
    
    Ok(())
}

fn main() {
    let save_dir = "./emails";
    std::fs::create_dir_all(save_dir).unwrap();
    
    if let Err(e) = download_and_delete_emails("user@example.com", "password", save_dir) {
        eprintln!("Error: {}", e);
    }
}
```

## Selective Message Management

### Scenario: Download Only Specific Messages

User wants to download only messages from specific senders or with certain subjects.

```rust
fn download_selective_messages(
    username: &str, 
    password: &str,
    filter_from: Option<&str>,
    filter_subject: Option<&str>
) -> Result<Vec<String>, Box<dyn std::error::Error>> 
{
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let messages = client.list()?;
    let mut downloaded = Vec::new();
    
    for (msg_num, _msg_size) in messages {
        // Retrieve message headers using TOP command (if supported)
        // Or retrieve full message and parse
        let message = client.retrieve(msg_num)?;
        
        let should_download = check_message_matches(&message, filter_from, filter_subject);
        
        if should_download {
            println!("Downloading message {}...", msg_num);
            downloaded.push(message);
        } else {
            println!("Skipping message {}...", msg_num);
        }
    }
    
    client.quit()?;
    
    Ok(downloaded)
}

fn check_message_matches(message: &str, filter_from: Option<&str>, filter_subject: Option<&str>) -> bool {
    let mut from_match = filter_from.is_none();
    let mut subject_match = filter_subject.is_none();
    
    for line in message.lines() {
        if let Some(filter) = filter_from {
            if line.starts_with("From:") && line.contains(filter) {
                from_match = true;
            }
        }
        
        if let Some(filter) = filter_subject {
            if line.starts_with("Subject:") && line.contains(filter) {
                subject_match = true;
            }
        }
        
        if line.is_empty() {
            break; // End of headers
        }
    }
    
    from_match && subject_match
}

fn main() {
    // Download only messages from boss@company.com
    match download_selective_messages(
        "user@example.com",
        "password",
        Some("boss@company.com"),
        None
    ) {
        Ok(messages) => {
            println!("Downloaded {} matching messages", messages.len());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Mobile Email Client

### Scenario: Limited Bandwidth Email Access

Mobile device with limited bandwidth wants to check emails efficiently.

```rust
fn check_emails_mobile(username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    // Get mailbox statistics
    let (message_count, total_size) = client.stat()?;
    println!("Mailbox: {} messages, {} bytes total", message_count, total_size);
    
    // List messages to see sizes
    let messages = client.list()?;
    
    // Download only small messages (< 10KB) for preview
    for (msg_num, msg_size) in messages {
        if msg_size < 10240 {
            println!("\nDownloading small message {} ({} bytes)...", msg_num, msg_size);
            let message = client.retrieve(msg_num)?;
            display_message_preview(&message);
        } else {
            println!("\nMessage {} is large ({} bytes), skipping...", msg_num, msg_size);
            // Could use TOP command here to get headers only
        }
    }
    
    client.quit()?;
    Ok(())
}

fn display_message_preview(message: &str) {
    println!("--- Preview ---");
    let mut line_count = 0;
    for line in message.lines() {
        if line_count < 15 {
            println!("{}", line);
            line_count += 1;
        } else {
            println!("... (truncated)");
            break;
        }
    }
}
```

## Email Backup Solution

### Scenario: Automated Email Backup

Automatically backup all emails from server to local storage periodically.

```rust
use std::path::Path;
use chrono::Local;

fn backup_emails(
    username: &str,
    password: &str,
    backup_dir: &str
) -> Result<usize, Box<dyn std::error::Error>> 
{
    // Create dated backup directory
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = format!("{}/{}", backup_dir, timestamp);
    std::fs::create_dir_all(&backup_path)?;
    
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let (message_count, _) = client.stat()?;
    println!("Backing up {} messages to {}...", message_count, backup_path);
    
    let mut backed_up = 0;
    
    for i in 1..=message_count {
        match client.retrieve(i) {
            Ok(message) => {
                // Generate unique filename
                let msg_id = extract_message_id(&message)
                    .unwrap_or_else(|| format!("msg_{}", i));
                let filename = format!("{}/{}.eml", backup_path, sanitize_filename(&msg_id));
                
                std::fs::write(&filename, message)?;
                backed_up += 1;
                
                if backed_up % 10 == 0 {
                    println!("Backed up {} messages...", backed_up);
                }
            }
            Err(e) => {
                eprintln!("Error backing up message {}: {}", i, e);
            }
        }
    }
    
    // Don't delete from server (backup only)
    client.quit()?;
    
    println!("Backup complete: {} messages saved to {}", backed_up, backup_path);
    
    Ok(backed_up)
}

fn extract_message_id(message: &str) -> Option<String> {
    for line in message.lines() {
        if line.starts_with("Message-ID:") || line.starts_with("Message-Id:") {
            return Some(line.split(':').nth(1)?.trim().to_string());
        }
    }
    None
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

fn main() {
    let backup_dir = "./email_backups";
    std::fs::create_dir_all(backup_dir).unwrap();
    
    match backup_emails("user@example.com", "password", backup_dir) {
        Ok(count) => println!("Successfully backed up {} emails", count),
        Err(e) => eprintln!("Backup failed: {}", e),
    }
}
```

## Multi-Device Synchronization

### Scenario: Leave Copy on Server

User accesses email from multiple devices and wants to leave copies on server.

```rust
use std::collections::HashSet;
use std::fs;

fn sync_with_leave_copy(
    username: &str,
    password: &str,
    local_cache: &str
) -> Result<(), Box<dyn std::error::Error>> 
{
    // Load list of already downloaded message IDs
    let downloaded_ids = load_downloaded_ids(local_cache)?;
    
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let messages = client.list()?;
    let mut new_messages = Vec::new();
    
    // Download only new messages
    for (msg_num, _) in messages {
        let message = client.retrieve(msg_num)?;
        let msg_id = extract_message_id(&message)
            .unwrap_or_else(|| format!("msg_{}", msg_num));
        
        if !downloaded_ids.contains(&msg_id) {
            println!("New message: {}", msg_id);
            new_messages.push((msg_id, message));
        }
    }
    
    // Don't delete from server
    client.quit()?;
    
    // Save new messages locally
    for (msg_id, message) in new_messages {
        let filename = format!("{}/{}.eml", local_cache, sanitize_filename(&msg_id));
        fs::write(&filename, message)?;
        save_downloaded_id(local_cache, &msg_id)?;
    }
    
    Ok(())
}

fn load_downloaded_ids(cache_dir: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let index_file = format!("{}/downloaded.txt", cache_dir);
    
    if !Path::new(&index_file).exists() {
        return Ok(HashSet::new());
    }
    
    let content = fs::read_to_string(index_file)?;
    Ok(content.lines().map(|s| s.to_string()).collect())
}

fn save_downloaded_id(cache_dir: &str, msg_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let index_file = format!("{}/downloaded.txt", cache_dir);
    let mut content = fs::read_to_string(&index_file).unwrap_or_default();
    content.push_str(&format!("{}\n", msg_id));
    fs::write(index_file, content)?;
    Ok(())
}
```

## Email Migration

### Scenario: Migrate from Old Server to New Server

Transfer all emails from one server to another.

```rust
fn migrate_emails(
    old_server: &str,
    old_user: &str,
    old_pass: &str,
    new_server: &str,
    new_user: &str,
    new_pass: &str
) -> Result<usize, Box<dyn std::error::Error>> 
{
    // Connect to old server (POP3)
    let mut old_client = Pop3Client::connect(old_server, 110)?;
    old_client.authenticate(old_user, old_pass)?;
    
    let (message_count, _) = old_client.stat()?;
    println!("Migrating {} messages from {} to {}...", message_count, old_server, new_server);
    
    // Connect to new server (SMTP for delivery)
    let mut new_client = SmtpClient::connect(new_server, 587)?;
    new_client.authenticate(new_user, new_pass)?;
    
    let mut migrated = 0;
    
    for i in 1..=message_count {
        // Retrieve from old server
        match old_client.retrieve(i) {
            Ok(message) => {
                // Send to new server
                match new_client.send_raw_message(&message) {
                    Ok(_) => {
                        migrated += 1;
                        println!("Migrated message {}/{}", migrated, message_count);
                        
                        // Optionally delete from old server after successful migration
                        // old_client.delete(i)?;
                    }
                    Err(e) => {
                        eprintln!("Error migrating message {}: {}", i, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error retrieving message {}: {}", i, e);
            }
        }
        
        // Rate limiting - don't overwhelm servers
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    old_client.quit()?;
    new_client.quit()?;
    
    println!("Migration complete: {}/{} messages migrated", migrated, message_count);
    
    Ok(migrated)
}
```

## Automated Email Processing

### Scenario: Process Incoming Emails Automatically

Automatically process emails based on rules (e.g., save attachments, forward specific emails).

```rust
use std::path::Path;

struct EmailRule {
    from_filter: Option<String>,
    subject_filter: Option<String>,
    action: EmailAction,
}

enum EmailAction {
    SaveAttachments(String),
    Forward(String),
    Delete,
    Archive(String),
}

fn process_emails_automatically(
    username: &str,
    password: &str,
    rules: Vec<EmailRule>
) -> Result<(), Box<dyn std::error::Error>> 
{
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let messages = client.list()?;
    
    for (msg_num, _) in messages {
        let message = client.retrieve(msg_num)?;
        
        // Apply rules
        for rule in &rules {
            if matches_rule(&message, rule) {
                println!("Message {} matches rule, applying action...", msg_num);
                apply_action(&message, &rule.action)?;
                
                if matches!(rule.action, EmailAction::Delete) {
                    client.delete(msg_num)?;
                }
            }
        }
    }
    
    client.quit()?;
    Ok(())
}

fn matches_rule(message: &str, rule: &EmailRule) -> bool {
    let mut matches = true;
    
    if let Some(from_filter) = &rule.from_filter {
        let mut found = false;
        for line in message.lines() {
            if line.starts_with("From:") && line.contains(from_filter) {
                found = true;
                break;
            }
        }
        matches = matches && found;
    }
    
    if let Some(subject_filter) = &rule.subject_filter {
        let mut found = false;
        for line in message.lines() {
            if line.starts_with("Subject:") && line.contains(subject_filter) {
                found = true;
                break;
            }
        }
        matches = matches && found;
    }
    
    matches
}

fn apply_action(message: &str, action: &EmailAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        EmailAction::SaveAttachments(dir) => {
            println!("Saving attachments to {}", dir);
            // Parse message and save attachments
            save_attachments(message, dir)?;
        }
        EmailAction::Forward(address) => {
            println!("Forwarding to {}", address);
            // Forward message
            forward_message(message, address)?;
        }
        EmailAction::Delete => {
            println!("Marking for deletion");
        }
        EmailAction::Archive(dir) => {
            println!("Archiving to {}", dir);
            let filename = format!("{}/archived_{}.eml", dir, chrono::Local::now().timestamp());
            std::fs::write(filename, message)?;
        }
    }
    Ok(())
}

fn save_attachments(message: &str, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Simplified - in production, use proper MIME parsing library
    std::fs::create_dir_all(dir)?;
    println!("Attachments saved (implementation simplified)");
    Ok(())
}

fn forward_message(message: &str, to: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Forwarding message to {} (implementation simplified)", to);
    Ok(())
}

fn main() {
    let rules = vec![
        EmailRule {
            from_filter: Some("newsletter@company.com".to_string()),
            subject_filter: None,
            action: EmailAction::Archive("./newsletters".to_string()),
        },
        EmailRule {
            from_filter: Some("spam@".to_string()),
            subject_filter: None,
            action: EmailAction::Delete,
        },
        EmailRule {
            from_filter: None,
            subject_filter: Some("[URGENT]".to_string()),
            action: EmailAction::Forward("mobile@example.com".to_string()),
        },
    ];
    
    if let Err(e) = process_emails_automatically("user@example.com", "password", rules) {
        eprintln!("Error: {}", e);
    }
}
```

## Error Recovery Scenarios

### Scenario: Handling Network Interruptions

Robust email retrieval with retry logic and error recovery.

```rust
use std::time::Duration;
use std::thread;

fn retrieve_with_retry(
    username: &str,
    password: &str,
    max_retries: u32
) -> Result<Vec<String>, Box<dyn std::error::Error>> 
{
    let mut retry_count = 0;
    
    loop {
        match try_retrieve_emails(username, password) {
            Ok(messages) => return Ok(messages),
            Err(e) => {
                retry_count += 1;
                
                if retry_count >= max_retries {
                    return Err(format!("Failed after {} retries: {}", max_retries, e).into());
                }
                
                let wait_time = 2u64.pow(retry_count); // Exponential backoff
                eprintln!("Error: {}. Retrying in {} seconds... (attempt {}/{})", 
                         e, wait_time, retry_count, max_retries);
                thread::sleep(Duration::from_secs(wait_time));
            }
        }
    }
}

fn try_retrieve_emails(username: &str, password: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let messages = client.list()?;
    let mut retrieved = Vec::new();
    
    for (msg_num, _) in messages {
        match client.retrieve(msg_num) {
            Ok(msg) => retrieved.push(msg),
            Err(e) => {
                eprintln!("Warning: Failed to retrieve message {}: {}", msg_num, e);
                // Continue with other messages
            }
        }
    }
    
    client.quit()?;
    Ok(retrieved)
}

fn main() {
    match retrieve_with_retry("user@example.com", "password", 5) {
        Ok(messages) => {
            println!("Successfully retrieved {} messages", messages.len());
        }
        Err(e) => {
            eprintln!("Failed to retrieve emails: {}", e);
        }
    }
}
```

### Scenario: Partial Download Recovery

Resume downloading messages after interruption.

```rust
use std::collections::HashSet;

fn resume_download(
    username: &str,
    password: &str,
    save_dir: &str
) -> Result<(), Box<dyn std::error::Error>> 
{
    // Check which messages are already downloaded
    let downloaded = get_downloaded_messages(save_dir)?;
    
    let mut client = Pop3Client::connect("mail.example.com", 110)?;
    client.authenticate(username, password)?;
    
    let messages = client.list()?;
    let total = messages.len();
    let mut downloaded_count = downloaded.len();
    
    println!("Found {} total messages, {} already downloaded", total, downloaded_count);
    
    for (msg_num, _) in messages {
        if downloaded.contains(&msg_num) {
            println!("Skipping already downloaded message {}", msg_num);
            continue;
        }
        
        match client.retrieve(msg_num) {
            Ok(message) => {
                let filename = format!("{}/message_{}.eml", save_dir, msg_num);
                std::fs::write(&filename, message)?;
                downloaded_count += 1;
                println!("Downloaded message {}/{}", downloaded_count, total);
            }
            Err(e) => {
                eprintln!("Error downloading message {}: {}", msg_num, e);
                // Save progress and exit
                return Err(e);
            }
        }
    }
    
    client.quit()?;
    println!("Download complete!");
    Ok(())
}

fn get_downloaded_messages(save_dir: &str) -> Result<HashSet<usize>, Box<dyn std::error::Error>> {
    let mut downloaded = HashSet::new();
    
    if let Ok(entries) = std::fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.starts_with("message_") && filename.ends_with(".eml") {
                    if let Some(num_str) = filename.strip_prefix("message_").and_then(|s| s.strip_suffix(".eml")) {
                        if let Ok(num) = num_str.parse::<usize>() {
                            downloaded.insert(num);
                        }
                    }
                }
            }
        }
    }
    
    Ok(downloaded)
}
```

## Conclusion

These scenarios demonstrate practical uses of POP3 in various contexts:

1. **Basic Retrieval** - Simple email client functionality
2. **Download & Delete** - Traditional POP3 usage
3. **Selective Management** - Smart filtering and downloading
4. **Mobile Access** - Bandwidth-conscious access
5. **Backup** - Automated email backup
6. **Multi-Device** - Leave copy on server
7. **Migration** - Transfer emails between servers
8. **Automation** - Rule-based processing
9. **Error Recovery** - Robust error handling

Each scenario includes working code examples that can be adapted for specific use cases. The key is to choose the right approach based on your requirements for message persistence, bandwidth usage, and device synchronization.
