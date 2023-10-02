/// SOCKS protocol version 5 identifier.
pub const SOCKS_VER_5: u8 = 0x05u8;
/// SOCKS protocol version 6 identifier.
pub const SOCKS_VER_6: u8 = 0x06u8;

/// Version identifier for SOCKS authentication.
pub const SOCKS_AUTH_VER: u8 = 0x01u8;
/// Code for no authentication required.
pub const SOCKS_AUTH_NOT_REQUIRED: u8 = 0x00u8;
/// Code for username/password authentication.
pub const SOCKS_AUTH_USERNAME_PASSWORD: u8 = 0x02u8;
/// Code for no acceptable authentication methods.
pub const SOCKS_AUTH_NO_ACCEPTABLE_METHODS: u8 = 0xFFu8;
/// Code for successful authentication.
pub const SOCKS_AUTH_SUCCESS: u8 = 0x00u8;
/// Code for failed authentication.
pub const SOCKS_AUTH_FAILED: u8 = 0x01u8;

/// Option kind for stack in SOCKS protocol.
pub const SOCKS_OKIND_STACK: u16 = 0x01u16;
/// Option kind for advertising authentication methods.
pub const SOCKS_OKIND_AUTH_METH_ADV: u16 = 0x02u16;
/// Option kind for selecting authentication methods.
pub const SOCKS_OKIND_AUTH_METH_SEL: u16 = 0x03u16;
/// Option kind for authentication data.
pub const SOCKS_OKIND_AUTH_DATA: u16 = 0x04u16;

/// Command code for no operation.
pub const SOCKS_CMD_NOOP: u8 = 0x00u8;
/// Command code for establishing a TCP/IP stream connection.
pub const SOCKS_CMD_CONNECT: u8 = 0x01u8;
/// Command code for establishing a TCP/IP port binding.
pub const SOCKS_CMD_BIND: u8 = 0x02u8;
/// Command code for associating a UDP port.
pub const SOCKS_CMD_UDP_ASSOCIATE: u8 = 0x03u8;

/// Padding byte for SOCKS protocol.
pub const SOCKS_PADDING: u8 = 0x00u8;
/// Reserved byte for SOCKS protocol.
pub const SOCKS_RSV: u8 = 0x00u8;

/// Address type identifier for IPv4 addresses.
pub const SOCKS_ATYP_IPV4: u8 = 0x01u8;
/// Address type identifier for domain names.
pub const SOCKS_ATYP_DOMAINNAME: u8 = 0x03u8;
/// Address type identifier for IPv6 addresses.
pub const SOCKS_ATYP_IPV6: u8 = 0x04u8;

/// Reply code for succeeded operation.
pub const SOCKS_REP_SUCCEEDED: u8 = 0x00u8;
