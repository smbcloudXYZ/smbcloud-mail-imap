use native_tls::{Identity, TlsAcceptor};
use tokio::net::TcpStream;
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;

use crate::generate_certificate::generate_certificate;

pub async fn handle_starttls(stream: TcpStream) -> anyhow::Result<()> {
    // ideally the certificate should only be loaded from here and not generated each time
    let (pem_certificate, pem_private_key) = generate_certificate()?;
    let identity = Identity::from_pkcs8(&pem_certificate, &pem_private_key)?;
    let tls_acceptor = TlsAcceptor::builder(identity).build()?;
    let tls_acceptor = TokioTlsAcceptor::from(tls_acceptor);

    match tls_acceptor.accept(stream).await {
        Ok(tls_stream) => {
            // we can now handle the normal SMTP session over TLS
            // For now, we'll just create a framed connection to handle encrypted SMTP
            use tokio_util::codec::{Framed, LinesCodec};
            use futures::StreamExt;
            
            let mut framed = Framed::new(tls_stream, LinesCodec::new());
            while let Some(line_str) = framed.next().await {
                let _line = line_str?;
                // Basic echo for now - in a real implementation, this would handle SMTP commands
                break;
            }
        }
        Err(e) => {
            tracing::error!("Error establishing SMTP TLS connection: {:?}", e);
        }
    };
    Ok(())
}
