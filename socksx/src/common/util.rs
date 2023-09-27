use std::net::SocketAddr;

use anyhow::Result;
use tokio::net::{self, TcpStream};

/// Retrieves the original destination address from a socket on a Linux system.
///
/// # Parameters
///
/// * `socket`: A reference to a socket implementing `AsRawFd`.
///
/// # Returns
///
/// Returns a `Result` containing the original `SocketAddr` or an error.
#[cfg(target_os = "linux")]
pub fn get_original_dst<S: std::os::unix::io::AsRawFd>(socket: &S) -> Result<SocketAddr> {
    use nix::sys::socket::{self, InetAddr, sockopt};

    let original_dst = socket::getsockopt(socket.as_raw_fd(), sockopt::OriginalDst)?;
    let original_dst = InetAddr::V4(original_dst).to_std();

    println!("{original_dst}");
    Ok(original_dst)
}

/// Retrieves the original destination address from a socket on a Windows system.
///
/// # Parameters
///
/// * `socket`: A reference to a socket implementing `AsRawSocket`.
///
/// # Returns
///
/// Returns a `Result` containing the original `SocketAddr` or an error.
#[cfg(target_os = "windows")]
pub fn get_original_dst<S: std::os::windows::io::AsRawSocket>(socket: &S) -> Result<SocketAddr> {
    use std::str::FromStr;
    use windows::core::PSTR;
    use windows::Win32::Networking::WinSock::{getsockopt, SO_ORIGINAL_DST, SOCKET, SOL_SOCKET};

    // Attempt to recover the original destination
    let original_dst: String = unsafe {
        // Write the socket option to a buffer
        let mut original_dst : [u8; 256] = [0; 256];
        let mut n_bytes      : i32       = 256;
        if getsockopt(SOCKET(socket.as_raw_socket() as usize), SOL_SOCKET, SO_ORIGINAL_DST as i32, PSTR((&mut original_dst) as *mut u8), &mut n_bytes) != 0 {
            panic!("Failed to get original address from socket");
        }

        // Parse it as an address
        String::from_utf8_lossy(&original_dst[..n_bytes as usize]).into()
    };

    // Now return the parsed socket address
    println!("{original_dst}");
    Ok(SocketAddr::from_str(&original_dst)?)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn get_original_dst<S>(_socket: S) -> Result<SocketAddr> {
    todo!();
}

/// Resolves a given address to a `SocketAddr`.
///
/// # Parameters
///
/// * `addr`: The address, either as a domain name or IP address.
///
/// # Returns
///
/// Returns a `Result` containing the resolved `SocketAddr` or an error.
pub async fn resolve_addr<S: Into<String>>(addr: S) -> Result<SocketAddr> {
    let addr: String = addr.into();

    // First, try to parse address as socket address.
    if let Ok(addr) = addr.parse() {
        return Ok(addr);
    }

    // Otherwise, address is probably a domain name.
    let addresses: Vec<SocketAddr> = net::lookup_host(addr).await?.collect();
    match addresses[..] {
        [first, ..] => Ok(first),
        [] => bail!("Domain name didn't resolve to an IP address."),
    }
}

/// Attempts to read the initial data from a TCP stream.
///
/// # Parameters
///
/// * `stream`: A mutable reference to a `TcpStream`.
///
/// # Returns
///
/// Returns a `Result` containing an `Option` with the read data as a `Vec<u8>` or an error.
pub async fn try_read_initial_data(stream: &mut TcpStream) -> Result<Option<Vec<u8>>> {
    let mut initial_data = Vec::with_capacity(2usize.pow(14)); // 16KB is the max

    stream.readable().await?;
    match stream.try_read_buf(&mut initial_data) {
        Ok(0) => Ok(None),
        Ok(_) => Ok(Some(initial_data)),
        Err(e) => {
            return Err(e.into());
        }
    }
}
