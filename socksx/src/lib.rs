//! This crate provides SOCKS proxy client and server implementations. It supports both SOCKS5 and SOCKS6 protocols.
//! 
//! While the crate is still in development, it is already usable. 
//! 
//! ## Chaining Features
//! For SOCKS version 5, chaining is not supported yet. It will be added in the future. Eg. Client -> Socks5 -> Destination  
//! For SOCKS version 6, chaining is supported. It means that you can chain multiple SOCKS6 proxies together. Eg. Client -> Socks6 -> Socks6 -> Destination  
//!



#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;
#[macro_use]
extern crate num_derive;

pub use tokio::io::copy_bidirectional;

/// Represents network addresses.
pub use addresses::{Address, ProxyAddress};
/// Manages user credentials.
pub use credentials::Credentials;
/// Handles SOCKS protocol.
pub use interface::SocksHandler;
/// SOCKS5 client and handler.
pub use socks5::{Socks5Client, Socks5Handler};
/// SOCKS6 client and handler.
pub use socks6::{Socks6Client, Socks6Handler};
pub use util::{get_original_dst, resolve_addr, try_read_initial_data};

/// Common network address representations
#[path = "./common/addresses.rs"]
pub mod addresses;

/// SOCKS protocol Constants used across the crate.
#[path = "./common/constants.rs"]
pub mod constants;

/// Credential management for the SOCKS proxy.
#[path = "./common/credentials.rs"]
pub mod credentials;

/// Main interface for handling SOCKS.
#[path = "./common/interface.rs"]
pub mod interface;

/// SOCKS5-specific implementations.
pub mod socks5;

/// SOCKS6-specific implementations.
pub mod socks6;

/// Utility functions and helpers.
#[path = "./common/util.rs"]
pub mod util;

