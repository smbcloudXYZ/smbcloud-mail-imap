use anyhow::Result;
use smbcloud_mail::listen::listen;

#[tokio::main]
async fn main() -> Result<()> {
    listen().await
}
