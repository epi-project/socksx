use anyhow::Result;
use num_traits::FromPrimitive;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub use s5_client::Socks5Client;
pub use s5_handler::Socks5Handler;

use crate::addresses::{self, Address};
use crate::constants::*;

mod s5_client;
mod s5_handler;

/// Represents the different commands for SOCKS5 protocol.
#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum Socks5Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

/// Represents a SOCKS5 request.
#[derive(Clone, Debug)]
pub struct Socks5Request {
    pub command: Socks5Command,
    pub destination: Address,
}

impl Socks5Request {
    /// Creates a new SOCKS5 request.
    ///
    /// # Arguments
    ///
    /// * `command` - The command type (e.g., Connect).
    /// * `destination` - The target address and port to connect to.
    ///
    /// # Returns
    ///
    /// A new `Socks5Request` instance.
    pub fn new(
        command: u8,
        destination: Address,
    ) -> Self {
        Socks5Request {
            command: Socks5Command::from_u8(command).unwrap(),
            destination,
        }
    }

    /// Converts the request into bytes suitable for transmission over a SOCKS5 connection.
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the request.
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = vec![SOCKS_VER_5, SOCKS_CMD_CONNECT, SOCKS_RSV];
        data.extend(self.destination.as_socks_bytes());

        data
    }
}

/// Represents different reply codes for SOCKS5 protocol.
#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum Socks5Reply {
    Success = 0x00,
    GeneralFailure = 0x01,
    ConnectionNotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TTLExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
    ConnectionAttemptTimeOut = 0x09,
}

/// Writes a SOCKS5 reply to the provided stream.
///
/// # Arguments
///
/// * `stream` - The output stream where the reply will be written.
/// * `reply` - The SOCKS5 reply code to be written.
///
/// # Returns
///
/// A `Result` indicating success or an error.
pub async fn write_reply<S>(
    stream: &mut S,
    reply: Socks5Reply,
) -> Result<()>
    where
        S: AsyncWrite + Unpin,
{
    let reply = [
        SOCKS_VER_5,
        reply as u8,
        SOCKS_RSV,
        SOCKS_ATYP_IPV4,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
    ];

    stream.write(&reply).await?;

    Ok(())
}

/// Reads a SOCKS5 reply from the provided stream and returns the associated address.
///
/// # Arguments
///
/// * `stream` - The input stream where the reply will be read from.
///
/// # Returns
///
/// A `Result` containing the address associated with the reply if successful, or an error if the reply indicates failure.
pub async fn read_reply<S>(stream: &mut S) -> Result<Address>
    where
        S: AsyncRead + Unpin,
{
    let mut operation_reply = [0; 3];
    stream.read_exact(&mut operation_reply).await?;

    let reply_code = operation_reply[1];
    ensure!(
        reply_code == SOCKS_REP_SUCCEEDED,
        "CONNECT operation failed: {}",
        reply_code
    );

    let binding = addresses::read_address(stream).await?;

    Ok(binding)
}
