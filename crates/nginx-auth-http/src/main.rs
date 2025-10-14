use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

struct AuthServer {
    users: HashMap<String, String>,
}

impl AuthServer {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("admin".to_string(), "your_admin_password".to_string());
        users.insert("user".to_string(), "your_user_password".to_string());

        AuthServer { users }
    }

    fn handle_request(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let request = String::from_utf8_lossy(&buffer[..]);
        let lines: Vec<&str> = request.lines().collect();

        // Find Authorization header
        let auth_header = lines
            .iter()
            .find(|line| line.to_lowercase().starts_with("authorization:"))
            .map(|line| line.split_once(": ").unwrap_or(("", "")).1);

        let response = if let Some(auth) = auth_header {
            if auth.starts_with("Basic ") {
                match self.validate_credentials(&auth[6..]) {
                    true => "HTTP/1.1 200 OK\r\n\r\n",
                    false => "HTTP/1.1 401 Unauthorized\r\n\r\n",
                }
            } else {
                "HTTP/1.1 401 Unauthorized\r\n\r\n"
            }
        } else {
            "HTTP/1.1 401 Unauthorized\r\n\r\n"
        };

        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    fn validate_credentials(&self, encoded_creds: &str) -> bool {
        if let Ok(decoded) = general_purpose::STANDARD.decode(encoded_creds) {
            if let Ok(creds_str) = String::from_utf8(decoded) {
                if let Some((username, password)) = creds_str.split_once(':') {
                    return self.users.get(username).map_or(false, |p| p == password);
                }
            }
        }
        false
    }
}

fn main() {
    let auth_server = AuthServer::new();
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Auth server running on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                auth_server.handle_request(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
