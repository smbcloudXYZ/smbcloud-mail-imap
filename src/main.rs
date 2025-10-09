use anyhow::Result;
use smbcloud_mail::listen::listen;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt::init();
    
    listen().await
}
