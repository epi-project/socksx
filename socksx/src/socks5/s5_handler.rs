use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::{constants::*, Credentials};
use crate::addresses::{self, ProxyAddress};
use crate::socks5::{self, Socks5Reply};
use crate::SocksHandler;

/// Represents a SOCKS5 handler for processing client requests.
#[derive(Clone)]
pub struct Socks5Handler {
    credentials: Option<Credentials>,
    //chain: Vec<ProxyAddress>,
}

impl Default for Socks5Handler {
    fn default() -> Self {
        Self::new(vec![])
    }
}

impl Socks5Handler {
    /// Creates a new `Socks5Handler` with an optional list of proxy addresses.
    ///
    /// # Arguments
    ///
    /// * `chain` - A vector of `ProxyAddress` instances representing proxy servers in a chain.
    ///
    /// # Returns
    ///
    /// A new `Socks5Handler` instance.
    pub fn new(_chain: Vec<ProxyAddress>) -> Self {
        Socks5Handler {
            credentials: None,
            //chain,
        }
    }
}

#[async_trait]
impl SocksHandler for Socks5Handler {
    /// Accepts a SOCKS5 client request and sets up a bidirectional connection.
    ///
    /// # Arguments
    ///
    /// * `source` - The TCP stream representing the client connection.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    async fn accept_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()> {
        let mut destination = self.setup(source).await?;

        // Start bidirectional copy, after this the connection closes.
        tokio::io::copy_bidirectional(source, &mut destination).await?;

        Ok(())
    }

    /// Refuses a SOCKS5 client request and notifies the client.
    ///
    /// # Arguments
    ///
    /// * `source` - The TCP stream representing the client connection.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error.
    async fn refuse_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()> {
        // Notify source that the connection is refused.
        socks5::write_reply(source, Socks5Reply::ConnectionRefused).await?;

        Ok(())
    }

    /// Sets up the SOCKS5 connection with a client.
    ///
    /// # Arguments
    ///
    /// * `source` - The TCP stream representing the client connection.
    ///
    /// # Returns
    ///
    /// A `Result` containing a TCP stream representing the destination connection.
    async fn setup(
        &self,
        source: &mut TcpStream,
    ) -> Result<TcpStream> {
        let mut request = [0; 2];
        source.read_exact(&mut request).await?;

        let socks_version = request[0];

        if socks_version != SOCKS_VER_5 {
            bail!("Client uses a different SOCKS version: {}.", socks_version);
        }

        // Get all authentication methods the client proposes.
        let nmethods = request[1] as usize;

        let mut methods = vec![0; nmethods];
        source.read_exact(&mut methods).await?;

        let method = if self.credentials.is_some() && methods.contains(&SOCKS_AUTH_USERNAME_PASSWORD) {
            SOCKS_AUTH_USERNAME_PASSWORD
        } else if methods.contains(&SOCKS_AUTH_NOT_REQUIRED) {
            SOCKS_AUTH_NOT_REQUIRED
        } else {
            SOCKS_AUTH_NO_ACCEPTABLE_METHODS
        };

        info!("Use authentication method: {}", method);

        let response = [SOCKS_VER_5, method];
        source.write(&response).await?;

        // Enter method-specific sub-negotiation
        if method == SOCKS_AUTH_USERNAME_PASSWORD {
            let mut request = [0; 2];
            source.read_exact(&mut request).await?;

            let auth_version = request[0];
            if auth_version != SOCKS_AUTH_VER {
                bail!(
                    "Client uses a different authentication method version: {}.",
                    auth_version
                );
            }

            let ulen = request[1] as usize;
            let mut uname = vec![0; ulen];
            source.read_exact(&mut uname).await?;

            let plen = request[1] as usize;
            let mut passwd = vec![0; plen];
            source.read_exact(&mut passwd).await?;

            let status = if let Some(Credentials { username, password }) = &self.credentials {
                if &uname != username || &passwd != password {
                    SOCKS_AUTH_SUCCESS
                } else {
                    0x01u8
                }
            } else {
                unreachable!()
            };

            let response = [SOCKS_VER_5, status];
            source.write(&response).await?;

            ensure!(status == SOCKS_AUTH_SUCCESS, "Username/password authentication failed.");
        }

        let mut request = [0; 3];
        source.read_exact(&mut request).await?;

        let command = request[1];
        if command != SOCKS_CMD_CONNECT {
            unimplemented!();
        }

        let destination = addresses::read_address(source).await?;
        let destination = TcpStream::connect(destination.to_string()).await?;

        // Notify source that the connection has been set up.
        socks5::write_reply(source, Socks5Reply::Success).await?;
        source.flush().await?;

        Ok(destination)
    }
}
