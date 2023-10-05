/// A simple SOCKS client that connects to a destination server through a proxy.
/// This serves as an example of how to use the socksx crate.
/// This also serves as a test to ensure that the crate works as expected.
use anyhow::Result;
use clap::Parser;
use socksx::{Socks5Client, Socks6Client};
use tokio::io::AsyncWriteExt;


/***** ARGUMENTS *****/
#[derive(Debug, Parser)]
#[clap(name = "Client")]
struct Arguments {
    #[clap(name="VERSION", short='s', long="socks", default_value="6", help="The SOCKS version to use")]
    version    : u8,
    #[clap(name="PROXY_HOST", long="host", default_value="127.0.0.1", help="The IP/hostname of the proxy")]
    proxy_host : String,
    #[clap(name="PROXY_PORT", long="port", default_value="1080", help="The port of the proxy server")]
    proxy_port : u16,
    #[clap(name="DEST_HOST", long="dest_host", default_value="127.0.0.1", help="The IP/hostname of the destination")]
    dest_host  : String,
    #[clap(name="DEST_PORT", long="dest_port", default_value="12345", help="The port of the destination server")]
    dest_port  : u16,
}





/***** ENTRYPOINT *****/
#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments using the Clap library.
    let args = Arguments::parse();

    // Extract values from command-line arguments.
    let proxy_addr = format!("{}:{}", args.proxy_host, args.proxy_port);
    let dest_addr = format!("{}:{}", args.dest_host, args.dest_port);

    // Determine the appropriate SOCKS handler based on the specified version and restricting them to 5 and 6
    match args.version {
        5 => connect_v5(proxy_addr, dest_addr).await,
        6 => connect_v6(proxy_addr, dest_addr).await,
        version => panic!("Unsupported version: {}", version),
    }
}

/// Connects to a destination through a proxy using SOCKS5 protocol, then sends an example message through the network tunnel.
/// 
/// # Arguments
/// - `proxy_addr`: The address of the SOCKS5 proxy through which the traffic will be tunnelled.
/// - `dest_addr`: The address to which the traffic should be sent after the proxy.
/// 
/// # Errors
/// This function can error if we failed to connect to the given proxy or failed to send it an example message.
async fn connect_v5(
    proxy_addr: String,
    dest_addr: String,
) -> Result<()> {
    // Create a SOCKS5 client.
    let client = Socks5Client::new(proxy_addr, None).await?;

    // Connect to the destination.
    let (mut outgoing, _) = client.connect(dest_addr).await?;

    // Write a message to the destination.
    outgoing.write(String::from("Hello, world!\n").as_bytes()).await?;

    Ok(())
}

/// Connects to a destination using SOCKS6 protocol.
async fn connect_v6(
    proxy_addr: String,
    dest_addr: String,
) -> Result<()> {
    // Create a SOCKS6 client.
    let client = Socks6Client::new(proxy_addr, None).await?;

    // Connect to the destination.
    let (mut outgoing, _) = client.connect(dest_addr, None, None).await?;

    // Write a message to the destination.
    outgoing.write(String::from("Hello, world!\n").as_bytes()).await?;

    Ok(())
}
