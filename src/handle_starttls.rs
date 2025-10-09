use native_tls::{Identity, TlsAcceptor};
use tokio::net::TcpStream;
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;

use crate::{generate_certificate::generate_certificate, handle_session::handle_session};

pub async fn handle_starttls(stream: &mut TcpStream) -> anyhow::Result<()> {
    // ideally the certificate should only be loaded from here and not generated each time
    let (pem_certificate, pem_private_key) = generate_certificate()?;
    let identity = Identity::from_pkcs8(pem_certificate.as_bytes(), pem_private_key.as_bytes())?;
    let tls_acceptor = TlsAcceptor::builder(identity).build()?;
    let tls_acceptor = TokioTlsAcceptor::from(tls_acceptor);

    match tls_acceptor.accept(stream).await {
        Ok(tls_stream) => {
            // we can now handle the normal SMTP session
            handle_session(tls_stream).await?;
        }
        Err(e) => {
            tracing::error!("Error establishing SMTP TLS connection: {:?}", e);
        }
    };
    Ok(())
}
