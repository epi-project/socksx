/// Represents the username and password credentials for SOCKS authentication.
#[derive(Clone, Debug, PartialEq)]
pub struct Credentials {
    /// The username as a byte vector.
    pub username: Vec<u8>,
    /// The password as a byte vector.
    pub password: Vec<u8>,
}

impl Credentials {
    /// Creates a new `Credentials` instance.
    ///
    /// # Parameters
    ///
    /// * `username`: The username as a byte vector or convertible to a byte vector.
    /// * `password`: The password as a byte vector or convertible to a byte vector.
    pub fn new<S: Into<Vec<u8>>>(
        username: S,
        password: S,
    ) -> Self {
        let username = username.into();
        let password = password.into();

        Credentials { username, password }
    }

    /// Converts the `Credentials` into a byte sequence compatible with the SOCKS authentication protocol.
    ///
    /// # Returns
    ///
    /// Returns a vector of bytes containing the username and password in SOCKS-compatible format.
    pub fn as_socks_bytes(&self) -> Vec<u8> {
        // Append username
        let mut bytes = vec![self.username.len() as u8];
        bytes.extend(self.username.clone());

        // Append password
        bytes.push(self.password.len() as u8);
        bytes.extend(self.password.clone());

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_new() {
        let credentials = Credentials::new("username".to_string().into_bytes(), "password".to_string().into_bytes());
        assert_eq!(credentials.username, b"username".to_vec());
        assert_eq!(credentials.password, b"password".to_vec());
    }

    #[test]
    fn test_credentials_as_socks_bytes() {
        let credentials = Credentials::new("username".to_string().into_bytes(), "password".to_string().into_bytes());
        let socks_bytes = credentials.as_socks_bytes();
        assert_eq!(socks_bytes, vec![8, 117, 115, 101, 114, 110, 97, 109, 101, 8, 112, 97, 115, 115, 119, 111, 114, 100]);
    }
}
