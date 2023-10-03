/// This is a simple redirector that redirects all incoming TCP connections through a SOCKS proxy to
/// a different destination. This is useful for redirecting traffic from a specific application
/// through a proxy.
use anyhow::Result;
use clap::Parser;
use clap::builder::PossibleValuesParser;
use tokio::net::{TcpListener, TcpStream};

use socksx::{self, Socks5Client, Socks6Client};


/***** ARGUMENTS *****/
#[derive(Debug, Parser)]
#[clap(name = "Redirector")]
struct Arguments {
    #[clap(name="VERSION", short='s', long="socks", value_parser=PossibleValuesParser::new(["5", "6"]), default_value="6", help="The SOCKS version to use")]
    version    : u8,
    #[clap(name="PROXY_HOST", long="host", default_value="127.0.0.1", help="The IP/hostname of the proxy")]
    proxy_host : String,
    #[clap(name="PROXY_PORT", long="port", default_value="1080", help="The port of the proxy server")]
    proxy_port : u16,
}





/***** ENTRYPOINT *****/
// iptables -t nat -A OUTPUT ! -d $PROXY_HOST/32 -o eth0 -p tcp -m tcp -j REDIRECT --to-ports 42000
#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    let proxy_addr = format!("{}:{}", args.proxy_host, args.proxy_port);

    let listener = TcpListener::bind("127.0.0.1:42000").await?;
    match args.version {
        5 => {
            let client = Socks5Client::new(proxy_addr, None).await?;

            loop {
                let (stream, _) = listener.accept().await?;
                tokio::spawn(redirect_v5(stream, client.clone()));
            }
        }
        6 => {
            let client = Socks6Client::new(proxy_addr, None).await?;

            loop {
                let (stream, _) = listener.accept().await?;
                tokio::spawn(redirect_v6(stream, client.clone()));
            }
        }
        version => panic!("Unsupported version: {}", version),
    };
}

/// Redirect an incoming TCP stream through a SOCKS5
/// proxy. The original destination of the stream has
/// been preserved, by iptables, as an socket option.
async fn redirect_v5(
    incoming: TcpStream,
    client: Socks5Client,
) -> Result<()> {
    let mut incoming = incoming;

    let dst_addr = socksx::get_original_dst(&incoming)?.to_string();
    let (mut outgoing, _) = client.connect(dst_addr).await?;

    socksx::copy_bidirectional(&mut incoming, &mut outgoing).await?;

    Ok(())
}

/// Redirect an incoming TCP stream through a SOCKS6
/// proxy. The original destination of the stream has
/// been preserved, by iptables, as an socket option.
async fn redirect_v6(
    incoming: TcpStream,
    client: Socks6Client,
) -> Result<()> {
    let mut incoming = incoming;

    let dst_addr = socksx::get_original_dst(&incoming)?.to_string();
    let initial_data = socksx::try_read_initial_data(&mut incoming).await?;
    let (mut outgoing, _) = client.connect(dst_addr, initial_data, None).await?;

    socksx::copy_bidirectional(&mut incoming, &mut outgoing).await?;

    Ok(())
}
