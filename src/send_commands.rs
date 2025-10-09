use futures::{SinkExt, StreamExt, stream::iter};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LinesCodec};

pub async fn send_commands<S>(
    framed: &mut Framed<S, LinesCodec>,
    commands: Vec<String>,
) -> anyhow::Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // only need to add \r because the codec only adds \n
    let messages = iter(commands.into_iter().map(|x| format!("{}\r", x)));
    framed.send_all(&mut messages.map(Ok)).await?;
    Ok(())
}
