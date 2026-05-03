use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

struct AuthServer {
    users: HashMap<String, String>,
    upstream_host: String,
    smtp_port: u16,
    imap_port: u16,
}

impl AuthServer {
    fn new() -> Self {
        let mut users = HashMap::new();
        // TODO: replace with secure storage (DB/KMS). This is only a demo.
        users.insert("admin".to_string(), "your_admin_password".to_string());
        users.insert("user".to_string(), "your_user_password".to_string());

        AuthServer {
            users,
            upstream_host: "127.0.0.1".to_string(),
            smtp_port: 2525, // Your smbcloud-mail-smtp SMTP server
            imap_port: 1143, // Your smbcloud-mail-smtp IMAP server
        }
    }

    fn handle_client(&self, mut stream: TcpStream) {
        let Ok((method, _path, headers)) = read_http_request(&mut stream) else {
            let _ = stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n");
            let _ = stream.flush();
            return;
        };

        if method != "GET" && method != "POST" {
            let _ = stream.write_all(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n");
            let _ = stream.flush();
            return;
        }

        let method_hdr = get_hdr(&headers, "auth-method").unwrap_or("plain");
        let user = get_hdr(&headers, "auth-user").unwrap_or("");
        let pass = get_hdr(&headers, "auth-pass").unwrap_or("");
        let proto = get_hdr(&headers, "auth-protocol").unwrap_or("smtp");

        // Validate credentials
        let mut ok = false;
        if method_hdr.eq_ignore_ascii_case("plain") {
            if !user.is_empty() && !pass.is_empty() {
                if let Some(stored) = self.users.get(user) {
                    ok = stored == pass;
                }
            }
        }

        if ok {
            let (host, port) = match proto {
                p if p.eq_ignore_ascii_case("smtp") => (&self.upstream_host, self.smtp_port),
                p if p.eq_ignore_ascii_case("imap") => (&self.upstream_host, self.imap_port),
                _ => (&self.upstream_host, self.smtp_port),
            };

            let resp = format!(
                "HTTP/1.1 200 OK\r\n\
                 Auth-Status: OK\r\n\
                 Auth-Server: {host}\r\n\
                 Auth-Port: {port}\r\n\
                 Auth-User: {user}\r\n\
                 Auth-Wait: 0\r\n\
                 \r\n"
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        } else {
            let resp =
                "HTTP/1.1 200 OK\r\nAuth-Status: Invalid login or password\r\nAuth-Wait: 2\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    }
}

fn get_hdr<'a>(headers: &'a Vec<(String, String)>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

fn read_http_request(
    stream: &mut TcpStream,
) -> Result<(String, String, Vec<(String, String)>), ()> {
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 1024];
    loop {
        let n = stream.read(&mut tmp).map_err(|_| ())?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if memchr_crlfcrlf(&buf) {
            break;
        }
        if buf.len() > 64 * 1024 {
            break;
        }
    }

    let req = String::from_utf8_lossy(&buf);
    let mut lines = req.split("\r\n");

    let req_line = lines.next().ok_or(())?;
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Ok((method, path, headers))
}

fn memchr_crlfcrlf(buf: &[u8]) -> bool {
    buf.windows(4).any(|w| w == b"\r\n\r\n")
}

fn main() -> std::io::Result<()> {
    let auth = AuthServer::new();
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    eprintln!("Mail auth_http server listening on 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                auth.handle_client(stream);
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
    Ok(())
}
