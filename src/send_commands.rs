use futures::{SinkExt, StreamExt, stream::iter};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_util::codec::{Framed, LinesCodec};

pub async fn send_commands(
    framed: &mut Framed<TcpStream, LinesCodec>,
    commands: Vec<String>,
) -> anyhow::Result<()> {
    // only need to add \r because the codec only adds \n
    let messages = iter(commands.into_iter().map(|x| format!("{}\r", x)));
    framed.send_all(&mut messages.map(Ok)).await?;
    Ok(())
}
