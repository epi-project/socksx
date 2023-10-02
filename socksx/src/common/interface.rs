use anyhow::Result;
use async_trait::async_trait;
use tokio::net::TcpStream;

/// An asynchronous trait defining the core functionalities required for handling SOCKS requests.
#[async_trait]
pub trait SocksHandler {
    /// Accepts a SOCKS request from a client.
    ///
    /// # Parameters
    ///
    /// * `source`: A mutable reference to the source `TcpStream` from which the request originates.
    ///
    /// # Returns
    ///
    /// Returns `Result<()>` indicating the success or failure of the operation.
    async fn accept_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()>;

    /// Refuses a SOCKS request from a client.
    ///
    /// # Parameters
    ///
    /// * `source`: A reference to the source `TcpStream` from which the request originates.
    ///
    /// # Returns
    ///
    /// Returns `Result<()>` indicating the success or failure of the operation.
    async fn refuse_request(
        &self,
        source: &mut TcpStream,
    ) -> Result<()>;

    /// Sets up the SOCKS connection for a given source.
    ///
    /// # Parameters
    ///
    /// * `source`: A mutable reference to the source `TcpStream`.
    ///
    /// # Returns
    ///
    /// Returns a `Result<TcpStream>` containing the prepared `TcpStream` or an error.
    async fn setup(
        &self,
        source: &mut TcpStream,
    ) -> Result<TcpStream>;
}
