/// This example demonstrates how to apply a function to ingress traffic through the socks proxy.
/// This example uses ChaCha20 encryption/decryption as the function.
/// We can have other functions such as compression, decompression, firewall, VPN, annonimization, etc.
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use anyhow::Result;
use bytes::BytesMut;
use chacha20::{ChaCha20, Key, Nonce};
use chacha20::cipher::{NewCipher, StreamCipher};
use clap::Parser;
use dotenv::dotenv;
use pin_project_lite::pin_project;
use tokio::io::{self, AsyncBufRead, BufReader, BufWriter};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};

use socksx::{self, Socks5Handler, Socks6Handler, SocksHandler};

// Define a trait alias for the SocksHandler to simplify code.
type Handler = Arc<dyn SocksHandler + Sync + Send>;

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// Host (IP) for the SOCKS server
    #[clap(short, long, env = "HOST", default_value = "0.0.0.0")]
    host: String,

    /// Port for the SOCKS server
    #[clap(short, long, env = "PORT", default_value = "1080")]
    port: u16,

    /// SOCKS version
    #[clap(short, long, env = "SOCKS", default_value = "6", possible_values = &["5", "6"])]
    socks: u8,

    #[clap(subcommand)]
    function: Function,
}

// Define subcommands for the program.
#[derive(Parser, Clone)]
enum Function {
    /// Apply ChaCha20 encryption/decryption to ingress traffic
    #[clap(name = "chacha20")]
    ChaCha20 {
        /// Key to use for encryption (symmetric)
        #[clap(short, long, env = "CHACHA20_KEY")]
        key: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from a .env file.
    dotenv().ok();
    // Parse command-line arguments using Clap.
    let args = Args::parse();

    // Create a TCP listener bound to the specified host and port.
    let listener = TcpListener::bind(format!("{}:{}", args.host, args.port)).await?;
    // Determine the appropriate SOCKS handler based on the specified version.
    let handler: Handler = match args.socks {
        5 => Arc::new(Socks5Handler::default()),
        6 => Arc::new(Socks6Handler::default()),
        _ => unreachable!(),
    };

    // Main loop for accepting incoming connections and processing them.
    loop {
        let (incoming, _) = listener.accept().await?;
        let handler = Arc::clone(&handler);
        let function = args.function.clone();

        // Spawn a new asynchronous task to process each connection.
        tokio::spawn(process(incoming, handler, function));
    }
}

/// Process an incoming connection based on the specified function.
async fn process(
    source: TcpStream,
    handler: Handler,
    function: Function,
) -> Result<()> {
    let mut source = source;
    // Set up the destination connection using the SOCKS handler.
    let mut destination = handler.setup(&mut source).await?;

    // Apply a function to ingress traffic.
    match function {
        Function::ChaCha20 { key } => {
            let mut source = CryptStream::new(source, key);

            // Bidirectional data transfer between source and destination.
            tokio::io::copy_bidirectional(&mut source, &mut destination).await?;
        }
    }

    Ok(())
}

// Define a wrapper struct for encryption/decryption using ChaCha20.
pin_project! {
    #[derive(Debug)]
    pub struct CryptStream<RW> {
        #[pin]
        inner: BufReader<BufWriter<RW>>,
        key: String,
    }
}

impl<RW: AsyncRead + AsyncWrite> CryptStream<RW> {
    // Create a new CryptStream with encryption key.
    pub fn new(
        stream: RW,
        key: String,
    ) -> CryptStream<RW> {
        CryptStream {
            inner: BufReader::new(BufWriter::new(stream)),
            key,
        }
    }
}

// Implement the AsyncWrite trait for CryptStream.
impl<RW: AsyncRead + AsyncWrite> AsyncWrite for CryptStream<RW> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().inner.poll_shutdown(cx)
    }
}

// Implement the AsyncRead trait for CryptStream.
impl<RW: AsyncRead + AsyncWrite> AsyncRead for CryptStream<RW> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let reader = self.as_mut().project().inner;

        // Poll to fill the buffer.
        let remaining = match reader.poll_fill_buf(cx) {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }?;

        let amt = std::cmp::min(remaining.len(), buf.remaining());
        let mut data = BytesMut::from(&remaining[..amt]);

        let key = Key::from_slice(self.key[..].as_bytes());
        let nonce = Nonce::from_slice(b"secret nonce"); // TODO: random or implement counter ?

        // Apply keystream
        let mut cipher = ChaCha20::new(&key, &nonce);
        cipher.apply_keystream(&mut data);

        buf.put_slice(&data);
        self.as_mut().project().inner.consume(amt);

        Poll::Ready(Ok(()))
    }
}
