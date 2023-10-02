use anyhow::Result;
use num_traits::FromPrimitive;

/// Represents SOCKS authentication methods.
#[repr(u8)]
#[derive(Clone, Debug, FromPrimitive, PartialEq)]
pub enum AuthMethod {
    NoAuthentication = 0x00,
    Gssapi = 0x01,
    UsernamePassword = 0x02,
    NoAcceptableMethods = 0xFF,
}

/// Enumerates the types of SOCKS options.
#[derive(Clone, Debug)]
pub enum SocksOption {
    AuthMethodAdvertisement(AuthMethodAdvertisementOption),
    AuthMethodSelection(AuthMethodSelectionOption),
    Metadata(MetadataOption),
    Unrecognized(UnrecognizedOption),
}

impl SocksOption {
    /// Converts the SOCKS option to a vector of bytes.
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        use SocksOption::*;

        match self {
            AuthMethodAdvertisement(option) => option.clone().into_socks_bytes(),
            AuthMethodSelection(option) => option.clone().into_socks_bytes(),
            Metadata(option) => option.clone().into_socks_bytes(),
            Unrecognized(option) => option.clone().into_socks_bytes(),
        }
    }
}

/// Represents the authentication methods supported by the server.
#[derive(Clone, Debug)]
pub struct AuthMethodAdvertisementOption {
    pub initial_data_length: u16,
    pub methods: Vec<AuthMethod>,
}

impl AuthMethodAdvertisementOption {
    /// Constructs a new `AuthMethodAdvertisementOption`.
    pub fn new(
        initial_data_length: u16,
        methods: Vec<AuthMethod>,
    ) -> Self {
        Self {
            initial_data_length,
            methods,
        }
    }

    /// Wraps the instance into a `SocksOption`.
    pub fn wrap(self) -> SocksOption {
        SocksOption::AuthMethodAdvertisement(self)
    }

    /// Deserializes the option from bytes.
    pub fn from_socks_bytes(bytes: Vec<u8>) -> Result<SocksOption> {
        ensure!(bytes.len() >= 2, "Expected at least two bytes, got: {}", bytes.len());
        let initial_data_length = ((bytes[0] as u16) << 8) | bytes[1] as u16;

        let methods = bytes
            .iter()
            .skip(2)
            .filter(|m| {
                let m = **m;
                // Ingore "No Authentication Required" (implied) and padding bytes.
                m > 0 && m < 3
            })
            .map(|m| AuthMethod::from_u8(*m).unwrap())
            .collect();

        Ok(Self::new(initial_data_length, methods).wrap())
    }

    /// Serializes the option into bytes.
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = self.initial_data_length.to_be_bytes().to_vec();
        data.extend(self.methods.iter().cloned().map(|m| m as u8));

        combine_and_pad(0x02, data)
    }
}

/// Represents the authentication methods selected by the client.
#[derive(Clone, Debug)]
pub struct AuthMethodSelectionOption {
    pub method: AuthMethod,
}

impl AuthMethodSelectionOption {
    /// Constructs a new `AuthMethodSelectionOption`.
    pub fn new(method: AuthMethod) -> Self {
        Self { method }
    }

    /// Wraps the instance into a `SocksOption`.
    pub fn wrap(self) -> SocksOption {
        SocksOption::AuthMethodSelection(self)
    }

    /// Deserializes the option from bytes.
    pub fn from_socks_bytes(bytes: Vec<u8>) -> Result<SocksOption> {
        ensure!(bytes.len() == 4, "Expected exactly four bytes, got: {}", bytes.len());

        let method = bytes[0];
        if let Some(method) = AuthMethod::from_u8(method) {
            Ok(Self::new(method).wrap())
        } else {
            bail!("Not a valid authentication method selection: {}", method)
        }
    }

    /// Serializes the option into bytes.
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let data = vec![self.method as u8];

        combine_and_pad(0x03, data)
    }
}

/// Represents a metadata option.
#[derive(Clone, Debug)]
pub struct MetadataOption {
    pub key: u16,
    pub value: String,
}

impl MetadataOption {
    /// Constructs a new `MetadataOption`.
    pub fn new(
        key: u16,
        value: String,
    ) -> Self {
        Self { key, value }
    }

    /// Wraps the instance into a `SocksOption`.
    pub fn wrap(self) -> SocksOption {
        SocksOption::Metadata(self)
    }

    /// Deserializes the option from bytes.
    pub fn from_socks_bytes(bytes: Vec<u8>) -> Result<SocksOption> {
        ensure!(bytes.len() >= 4, "Expected at least four bytes, got: {}", bytes.len());
        let key = ((bytes[0] as u16) << 8) | bytes[1] as u16;
        let length = ((bytes[2] as u16) << 8) | bytes[3] as u16;

        let value = bytes[4..(length as usize) + 4].to_vec();
        if let Ok(value) = String::from_utf8(value) {
            Ok(Self::new(key, value).wrap())
        } else {
            bail!("Not a valid metadata UTF-8 string: {:?}", bytes[2..].to_vec())
        }
    }

    /// Serializes the option into bytes.
    pub fn into_socks_bytes(self) -> Vec<u8> {
        let mut data = self.key.to_be_bytes().to_vec();
        data.extend((self.value.len() as u16).to_be_bytes().iter());
        data.extend(self.value.as_bytes().iter());

        // kind: 65000
        combine_and_pad(0xFDE8, data)
    }
}

/// Represents an unrecognized option.
#[derive(Clone, Debug)]
pub struct UnrecognizedOption {
    kind: u16,
    data: Vec<u8>,
}

impl UnrecognizedOption {
    /// Constructs a new `UnrecognizedOption`.
    pub fn new(
        kind: u16,
        data: Vec<u8>,
    ) -> Self {
        Self { kind, data }
    }

    /// Wraps the instance into a `SocksOption`.
    pub fn wrap(self) -> SocksOption {
        SocksOption::Unrecognized(self)
    }

    /// Deserializes the option from bytes.
    pub fn into_socks_bytes(self) -> Vec<u8> {
        combine_and_pad(self.kind, self.data)
    }
}

/// Combines and pads the SOCKS option bytes.
///
/// # Parameters
///
/// - `kind`: The kind of the SOCKS option.
/// - `data`: The data associated with the SOCKS option.
///
/// # Returns
///
/// A vector of bytes representing the padded SOCKS option.
fn combine_and_pad(
    kind: u16,
    data: Vec<u8>,
) -> Vec<u8> {
    // The total length of the option is the combined number of bytes of
    // the kind, length, and data fields, plus the number of padding bytes.
    let option_length = data.len() + 2 + 2;
    let padding_bytes = vec![0; 4 - (option_length % 4)];
    let total_length: u16 = (option_length + padding_bytes.len()) as u16;

    let mut bytes = vec![];
    bytes.extend(kind.to_be_bytes().iter());
    bytes.extend(total_length.to_be_bytes().iter());
    bytes.extend(data);
    bytes.extend(padding_bytes);

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test the AuthMethod enum conversion from primitive types
    #[test]
    fn test_auth_method_from_primitive() {
        let method = AuthMethod::from_u8(0x00);
        assert_eq!(method, Some(AuthMethod::NoAuthentication));
    }

    // Test the AuthMethodAdvertisementOption constructor
    #[test]
    fn test_auth_method_advertisement_option_new() {
        let option = AuthMethodAdvertisementOption::new(0, vec![AuthMethod::NoAuthentication]);
        assert_eq!(option.initial_data_length, 0);
        assert_eq!(option.methods, vec![AuthMethod::NoAuthentication]);
    }

    // Test wrapping an AuthMethodAdvertisementOption into a SocksOption
    #[test]
    fn test_auth_method_advertisement_option_wrap() {
        let option = AuthMethodAdvertisementOption::new(0, vec![]);
        let wrapped = option.wrap();
        if let SocksOption::AuthMethodAdvertisement(_) = wrapped {
            assert!(true);
        } else {
            assert!(false, "Expected AuthMethodAdvertisement variant");
        }
    }

    // Test the from_socks_bytes function for AuthMethodAdvertisementOption
    #[test]
    fn test_from_socks_bytes_auth_method_advertisement() {
        let bytes = vec![0x00, 0x02, 0x00, 0x01, 0x02];
        let result = AuthMethodAdvertisementOption::from_socks_bytes(bytes);
        // Verify the result according to your expectations
        assert!(result.is_ok());
    }
}