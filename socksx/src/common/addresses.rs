use std::convert::{TryFrom, TryInto};
use std::net::{IpAddr, SocketAddr};

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt};
use url::Url;

use crate::{constants::*, Credentials};

/// Represents a SOCKS proxy address.
#[derive(Clone, Debug, PartialEq)]
pub struct ProxyAddress {
    /// The version of the SOCKS protocol.
    pub socks_version: u8,
    /// The hostname or IP address of the proxy.
    pub host: String,
    /// The port number of the proxy.
    pub port: u16,
    /// Optional credentials for authentication.
    pub credentials: Option<Credentials>,
}


impl ProxyAddress {
    /// Creates a new `ProxyAddress` instance.
    pub fn new(
        socks_version: u8,
        host: String,
        port: u16,
        credentials: Option<Credentials>,
    ) -> Self {
        Self {
            socks_version,
            host,
            port,
            credentials,
        }
    }

    /// Creates a root `ProxyAddress` with predefined settings.
    pub fn root() -> Self {
        ProxyAddress::new(6, String::from("root"), 1080, None)
    }
}


impl ToString for ProxyAddress {
    // Converts the `ProxyAddress` to a string representation.
    fn to_string(&self) -> String {
        format!("socks{}://{}:{}", self.socks_version, self.host, self.port)
    }
}

impl TryFrom<String> for ProxyAddress {
    type Error = anyhow::Error;

    // Converts a string to a `ProxyAddress`.
    fn try_from(proxy_addr: String) -> Result<Self> {
        let proxy_addr = Url::parse(&proxy_addr)?;

        ensure!(
            proxy_addr.host().is_some(),
            "Missing explicit IP/host in proxy address."
        );
        ensure!(proxy_addr.port().is_some(), "Missing explicit port in proxy address.");

        let socks_version = match proxy_addr.scheme() {
            "socks5" => SOCKS_VER_5,
            "socks6" => SOCKS_VER_6,
            scheme => bail!("Unrecognized SOCKS scheme: {}", scheme),
        };

        let username = proxy_addr.username();
        let credentials = if username.is_empty() {
            None
        } else {
            let password = proxy_addr.password().unwrap_or_default();
            Some(Credentials::new(username, password))
        };

        Ok(Self::new(
            socks_version,
            proxy_addr.host().map(|h| h.to_string()).unwrap(),
            proxy_addr.port().unwrap(),
            credentials,
        ))
    }
}

/// Represents a network address, which could be either a domain name or an IP address.
#[derive(Clone, Debug)]
pub enum Address {
    /// An address represented by a domain name.
    Domainname { host: String, port: u16 },
    /// An address represented by an IP address.
    Ip(SocketAddr),
}


impl Address {
    /// Creates a new `Address` instance.
    pub fn new<S: Into<String>>(
        host: S,
        port: u16,
    ) -> Self {
        let host = host.into();

        if let Ok(host) = host.parse::<IpAddr>() {
            Address::Ip(SocketAddr::new(host, port))
        } else {
            Address::Domainname { host, port }
        }
    }

    /// Converts the `Address` into a byte sequence compatible with the SOCKS protocol.
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        match self {
            Address::Ip(dst_addr) => {
                match dst_addr.ip() {
                    IpAddr::V4(host) => {
                        bytes.push(SOCKS_ATYP_IPV4);
                        bytes.extend(host.octets().iter());
                    }
                    IpAddr::V6(host) => {
                        bytes.push(SOCKS_ATYP_IPV6);
                        bytes.extend(host.octets().iter());
                    }
                }

                bytes.extend(dst_addr.port().to_be_bytes().iter())
            }
            Address::Domainname { host, port } => {
                bytes.push(SOCKS_ATYP_DOMAINNAME);

                let host = host.as_bytes();
                bytes.push(host.len() as u8);
                bytes.extend(host);

                bytes.extend(port.to_be_bytes().iter());
            }
        }

        bytes
    }
}

impl ToString for Address {
    // Converts the `Address` to a string representation.
    fn to_string(&self) -> String {
        match self {
            Address::Domainname { host, port } => format!("{}:{}", host, port),
            Address::Ip(socket_addr) => socket_addr.to_string(),
        }
    }
}

/// Tries to convert a `SocketAddr` into an `Address`.
impl TryFrom<SocketAddr> for Address {
    type Error = anyhow::Error;

    fn try_from(addr: SocketAddr) -> Result<Self, Self::Error> {
        addr.to_string().try_into()
    }
}

/// Tries to convert a `String` into an `Address`.
impl TryFrom<String> for Address {
    type Error = anyhow::Error;

    fn try_from(addr: String) -> Result<Self> {
        if let Some((host, port)) = addr.split_once(':') {
            Ok(Address::new(host, port.parse()?))
        } else {
            bail!("Address doesn't seperate host and port by ':'.")
        }
    }
}

/// Tries to convert a `ProxyAddress` into an `Address`.
impl TryFrom<&ProxyAddress> for Address {
    type Error = anyhow::Error;

    fn try_from(addr: &ProxyAddress) -> Result<Self> {
        format!("{}:{}", addr.host, addr.port).try_into()
    }
}

/// Reads the destination address from a stream and returns it as an `Address`.
pub async fn read_address<S>(stream: &mut S) -> Result<Address>
where
    S: AsyncRead + Unpin,
{
    // Read address type.
    let mut address_type = [0; 1];
    stream.read_exact(&mut address_type).await?;

    let dst_addr = match address_type[0] {
        SOCKS_ATYP_IPV4 => {
            let mut dst_addr = [0; 4];
            stream.read_exact(&mut dst_addr).await?;

            IpAddr::from(dst_addr).to_string()
        }
        SOCKS_ATYP_IPV6 => {
            let mut dst_addr = [0; 16];
            stream.read_exact(&mut dst_addr).await?;

            IpAddr::from(dst_addr).to_string()
        }
        SOCKS_ATYP_DOMAINNAME => {
            let mut length = [0; 1];
            stream.read_exact(&mut length).await?;

            let mut dst_addr = vec![0; length[0] as usize];
            stream.read_exact(&mut dst_addr).await?;

            String::from_utf8_lossy(&dst_addr[..]).to_string()
        }
        _ => unreachable!(),
    };

    // Read destination port.
    let mut dst_port = [0; 2];
    stream.read_exact(&mut dst_port).await?;

    let dst_port = ((dst_port[0] as u16) << 8) | dst_port[1] as u16;

    Ok(Address::new(dst_addr, dst_port))
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use anyhow::Result;

    use super::*;

    #[test]
    fn test_proxy_address_new() {
        let proxy_address = ProxyAddress::new(5, "localhost".to_string(), 1080, None);
        assert_eq!(proxy_address.socks_version, 5);
        assert_eq!(proxy_address.host, "localhost");
        assert_eq!(proxy_address.port, 1080);
        assert!(proxy_address.credentials.is_none());
    }

    #[test]
    fn test_proxy_address_root() {
        let root_address = ProxyAddress::root();
        assert_eq!(root_address.socks_version, 6);
        assert_eq!(root_address.host, "root");
        assert_eq!(root_address.port, 1080);
        assert!(root_address.credentials.is_none());
    }

    #[test]
    fn test_address_new_domain() {
        let address = Address::new("example.com", 80);
        match address {
            Address::Domainname { host, port } => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 80);
            },
            _ => panic!("Expected a domain name address"),
        }
    }

    #[test]
    fn test_address_new_ip() {
        let address = Address::new("192.168.1.1", 22);
        match address {
            Address::Ip(socket_addr) => {
                assert_eq!(socket_addr.ip().to_string(), "192.168.1.1");
                assert_eq!(socket_addr.port(), 22);
            },
            _ => panic!("Expected an IP address"),
        }
    }

    #[test]
    fn test_proxy_address_try_from_valid_string() -> Result<()> {
        let proxy_str = "socks5://localhost:1080".to_string();
        let proxy_address: ProxyAddress = proxy_str.try_into()?;
        assert_eq!(proxy_address.socks_version, SOCKS_VER_5);
        assert_eq!(proxy_address.host, "localhost");
        assert_eq!(proxy_address.port, 1080);
        Ok(())
    }

    #[test]
    fn test_proxy_address_try_from_invalid_string() {
        let proxy_str = "invalid://localhost:1080".to_string();
        let result: Result<ProxyAddress> = proxy_str.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_address_try_from_valid_string() -> Result<()> {
        let addr_str = "localhost:8000".to_string();
        let address: Address = addr_str.try_into()?;
        match address {
            Address::Domainname { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 8000);
            },
            _ => panic!("Expected a domain name address"),
        }
        Ok(())
    }

    #[test]
    fn test_address_try_from_invalid_string() {
        let addr_str = "localhost&8000".to_string();
        let result: Result<Address> = addr_str.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_address_try_from_socket_addr() -> Result<()> {
        let socket_addr: SocketAddr = "192.168.1.1:22".parse()?;
        let address: Address = socket_addr.try_into()?;
        match address {
            Address::Ip(addr) => {
                assert_eq!(addr.ip().to_string(), "192.168.1.1");
                assert_eq!(addr.port(), 22);
            },
            _ => panic!("Expected an IP address"),
        }
        Ok(())
    }

    #[test]
    fn test_address_try_from_proxy_address() -> Result<()> {
        let proxy_address = ProxyAddress::new(5, "localhost".to_string(), 1080, None);
        let address: Address = (&proxy_address).try_into()?;
        match address {
            Address::Domainname { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 1080);
            },
            _ => panic!("Expected a domain name address"),
        }
        Ok(())
    }

    // TODO: Add tests for `read_address` function once we have a way to mock the `AsyncRead`.
}
