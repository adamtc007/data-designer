use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tower_lsp::{LspService, Server};
use std::net::SocketAddr;

pub async fn run_websocket_server(addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    log::info!("WebSocket LSP server listening on ws://{}", addr);

    while let Ok((stream, peer_addr)) = listener.accept().await {
        log::info!("New WebSocket connection from {}", peer_addr);

        tokio::spawn(async move {
            if let Err(e) = handle_websocket_connection(stream, peer_addr).await {
                log::error!("WebSocket connection error from {}: {}", peer_addr, e);
            }
        });
    }

    Ok(())
}

async fn handle_websocket_connection(
    stream: tokio::net::TcpStream,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    // Accept WebSocket connection
    let ws_stream = accept_async(stream).await?;
    log::info!("WebSocket handshake completed with {}", peer_addr);

    let (ws_sender, ws_receiver) = ws_stream.split();

    // Create adapters for LSP
    let ws_reader = WebSocketReader::new(ws_receiver);
    let ws_writer = WebSocketWriter::new(ws_sender);

    // Create LSP service
    let (service, socket) = LspService::new(|client| dsl_lsp::Backend::new(client));

    // Serve LSP over WebSocket
    Server::new(ws_reader, ws_writer, socket)
        .serve(service)
        .await;

    log::info!("WebSocket connection closed with {}", peer_addr);
    Ok(())
}

// WebSocket to AsyncRead adapter
pub struct WebSocketReader<R> {
    receiver: R,
    buffer: Vec<u8>,
    position: usize,
}

impl<R> WebSocketReader<R> {
    pub fn new(receiver: R) -> Self {
        WebSocketReader {
            receiver,
            buffer: Vec::new(),
            position: 0,
        }
    }
}

use tokio::io::{AsyncRead, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_tungstenite::tungstenite::Message;

impl<R> AsyncRead for WebSocketReader<R>
where
    R: StreamExt + Unpin,
    R::Item: Into<Result<Message, tokio_tungstenite::tungstenite::Error>>,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // If we have buffered data, return it first
        if self.position < self.buffer.len() {
            let available = self.buffer.len() - self.position;
            let to_copy = available.min(buf.remaining());
            buf.put_slice(&self.buffer[self.position..self.position + to_copy]);
            self.position += to_copy;

            // Clear buffer if we've consumed it all
            if self.position >= self.buffer.len() {
                self.buffer.clear();
                self.position = 0;
            }

            return Poll::Ready(Ok(()));
        }

        // Otherwise, try to receive a new message
        match Pin::new(&mut self.receiver).poll_next(cx) {
            Poll::Ready(Some(item)) => {
                match item.into() {
                    Ok(Message::Text(text)) => {
                        self.buffer = text.into_bytes();
                        self.position = 0;

                        let to_copy = self.buffer.len().min(buf.remaining());
                        buf.put_slice(&self.buffer[0..to_copy]);
                        self.position = to_copy;

                        Poll::Ready(Ok(()))
                    }
                    Ok(Message::Binary(data)) => {
                        self.buffer = data;
                        self.position = 0;

                        let to_copy = self.buffer.len().min(buf.remaining());
                        buf.put_slice(&self.buffer[0..to_copy]);
                        self.position = to_copy;

                        Poll::Ready(Ok(()))
                    }
                    Ok(Message::Close(_)) => {
                        Poll::Ready(Ok(()))
                    }
                    Ok(_) => Poll::Pending,
                    Err(e) => Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))),
                }
            }
            Poll::Ready(None) => Poll::Ready(Ok(())),
            Poll::Pending => Poll::Pending,
        }
    }
}

// WebSocket to AsyncWrite adapter
pub struct WebSocketWriter<W> {
    sender: W,
}

impl<W> WebSocketWriter<W> {
    pub fn new(sender: W) -> Self {
        WebSocketWriter { sender }
    }
}

use tokio::io::AsyncWrite;

impl<W> AsyncWrite for WebSocketWriter<W>
where
    W: SinkExt<Message> + Unpin,
    <W as futures_util::Sink<Message>>::Error: std::error::Error + Send + Sync + 'static,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match Pin::new(&mut self.sender).poll_ready(cx) {
            Poll::Ready(Ok(())) => {
                let message = Message::Text(String::from_utf8_lossy(buf).to_string());
                match Pin::new(&mut self.sender).start_send(message) {
                    Ok(()) => Poll::Ready(Ok(buf.len())),
                    Err(e) => Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))),
                }
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match Pin::new(&mut self.sender).poll_flush(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match Pin::new(&mut self.sender).poll_close(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}