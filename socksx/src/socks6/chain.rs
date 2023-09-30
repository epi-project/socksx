use crate::addresses::ProxyAddress;
use crate::socks6::options::{MetadataOption, SocksOption};

/// The `SocksChain` struct is used for managing a chain of SOCKS proxy addresses.
#[derive(Clone, Debug)]
pub struct SocksChain {
    /// The current index within the links vector.
    pub index: usize,
    /// A vector containing the chain of proxy addresses.
    pub links: Vec<ProxyAddress>,
}

// Default implementation for `SocksChain`.
impl Default for SocksChain {
    // Creates a new `SocksChain` with index set to 0 and an empty links vector.
    fn default() -> Self {
        Self::new(0, vec![])
    }
}

impl SocksChain {
    /// Creates a new `SocksChain` with a given index and list of proxy addresses.
    pub fn new(
        index: usize,
        links: Vec<ProxyAddress>,
    ) -> Self {
        Self { index, links }
    }

    /// Returns a reference to the current `ProxyAddress` based on the index.
    /// Panics if index is out of bounds.
    pub fn current_link(&self) -> &ProxyAddress {
        self.links.get(self.index).unwrap()
    }

    /// Checks if there is a next `ProxyAddress` in the chain.
    /// Returns `true` if a next link exists, `false` otherwise.
    pub fn has_next(&self) -> bool {
        self.index + 1 < self.links.len()
    }

    /// Advances to the next `ProxyAddress` in the chain.
    /// Returns an `Option` containing a reference to the next `ProxyAddress`, if it exists.
    pub fn next_link(&mut self) -> Option<&ProxyAddress> {
        let link = self.links.get(self.index + 1);
        if link.is_some() {
            self.index += 1;
        }

        link
    }

    /// Inserts additional `ProxyAddress`es into the chain at the current position.
    /// If the chain is empty, appends the root and then the new links.
    pub fn detour(
        &mut self,
        links: &[ProxyAddress],
    ) {
        let links = links.iter().cloned();

        if self.links.is_empty() {
            // This means we're currently at the root.
            // We'll append ourself as the root link.
            self.links.push(ProxyAddress::root());
            self.links.extend(links);
        } else {
            let position = self.index + 1..self.index + 1;
            self.links.splice(position, links);
        }
    }

    /// Converts the `SocksChain` into a vector of `SocksOption`s.
    /// Adds metadata options to indicate the current index and total length of the chain.
    pub fn as_options(&self) -> Vec<SocksOption> {
        let mut chain_options: Vec<SocksOption> = self
            .links
            .iter()
            .enumerate()
            .map(|(i, c)| (i as u16, c.to_string()))
            .map(|(i, c)| MetadataOption::new(1000 + i, c).wrap())
            .collect();

        chain_options.push(MetadataOption::new(998, self.index.to_string()).wrap());
        chain_options.push(MetadataOption::new(999, self.links.len().to_string()).wrap());

        chain_options
    }
}

// Test cases for `SocksChain`.
#[cfg(test)]
mod tests {
    use super::*;

    // Tests default constructor
    #[test]
    pub fn test_default_constructor() {
        let chain = SocksChain::default();
        assert_eq!(chain.index, 0);
        assert_eq!(chain.links, vec![]);
    }

    // Tests custom constructor
    #[test]
    pub fn test_custom_constructor() {
        let chain = SocksChain::new(1, vec![ProxyAddress::new(6, String::from("localhost"), 1, None)]);
        assert_eq!(chain.index, 1);
        assert_eq!(chain.links, vec![ProxyAddress::new(6, String::from("localhost"), 1, None)]);
    }

    // Tests the `current_link` method
    #[test]
    #[should_panic]
    pub fn test_current_link_method() {
        let chain = SocksChain::new(1, vec![ProxyAddress::new(6, String::from("localhost"), 1, None)]);
        assert_eq!(chain.current_link(), &ProxyAddress::new(6, String::from("localhost"), 1, None));
    }

    // Test `has_next` method
    #[test]
    pub fn test_has_next() {
        let chain = SocksChain::new(0, vec![ProxyAddress::new(6, String::from("localhost"), 1, None)]);
        assert!(!chain.has_next());
    }

    // Test `next_link` method
    #[test]
    pub fn test_next_link() {
        let mut chain = SocksChain::new(0, vec![
            ProxyAddress::new(6, String::from("localhost"), 1, None),
            ProxyAddress::new(6, String::from("localhost"), 2, None),
        ]);
        assert_eq!(chain.next_link().unwrap().port, 2);
    }

    // Test `detour` method with empty chain
    #[test]
    pub fn test_detour_empty_chain() {
        let mut chain = SocksChain::default();
        chain.detour(&[ProxyAddress::new(6, String::from("localhost"), 1, None)]);
        assert_eq!(chain.index, 0);
        assert_eq!(chain.links[0], ProxyAddress::root());
        assert_eq!(chain.links[1], ProxyAddress::new(6, String::from("localhost"), 1, None));
    }

    // Tests the `detour` method by checking the order of ports after insertion.
    #[test]
    pub fn test_detour_method() {
        let mut chain = SocksChain::new(
            1,
            vec![
                ProxyAddress::new(6, String::from("localhost"), 1, None),
                ProxyAddress::new(6, String::from("localhost"), 2, None),
                ProxyAddress::new(6, String::from("localhost"), 3, None),
            ],
        );

        let extra = vec![
            ProxyAddress::new(6, String::from("localhost"), 4, None),
            ProxyAddress::new(6, String::from("localhost"), 5, None),
        ];
        chain.detour(&extra);

        let order: Vec<u16> = chain.links.iter().map(|l| l.port).collect();
        assert_eq!(order, vec![1, 2, 4, 5, 3]);
    }
}
