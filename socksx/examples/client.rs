/// A simple SOCKS client that connects to a destination server through a proxy.
/// This serves as an example of how to use the socksx crate.
/// This also serves as a test to ensure that the crate works as expected.
use anyhow::Result;
use clap::{App, Arg};
use socksx::{Socks5Client, Socks6Client};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments using the Clap library.
    let args = App::new("Client")
        .arg(
            Arg::new("VERSION")
                .short('s')
                .long("socks")
                .help("The SOCKS version to use")
                .possible_values(&["5", "6"])
                .default_value("6"),
        )
        .arg(
            Arg::new("PROXY_HOST")
                .long("host")
                .help("The IP/hostname of the proxy")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("PROXY_PORT")
                .long("port")
                .help("The port of the proxy server")
                .default_value("1080"),
        )
        .arg(
            Arg::new("DEST_HOST")
                .long("dest_host")
                .help("The IP/hostname of the destination")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("DEST_PORT")
                .long("dest_port")
                .help("The port of the destination server")
                .default_value("12345"),
        )
        .get_matches();

    // Extract values from command-line arguments.
    let proxy_host = args.value_of("PROXY_HOST").unwrap();
    let proxy_port = args.value_of("PROXY_PORT").unwrap();
    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);

    let dest_host = args.value_of("DEST_HOST").unwrap();
    let dest_port = args.value_of("DEST_PORT").unwrap();
    let dest_addr = format!("{}:{}", dest_host, dest_port);

    // Determine the SOCKS version specified in the arguments.
    match args.value_of("VERSION") {
        Some("5") => connect_v5(proxy_addr, dest_addr).await,
        Some("6") => connect_v6(proxy_addr, dest_addr).await,
        Some(version) => panic!("Unsupported version: {}", version),
        None => unreachable!(),
    }
}

/// Connects to a destination using SOCKS5 protocol.
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
