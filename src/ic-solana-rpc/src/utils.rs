use std::str::FromStr;

use ic_solana::{
    rpc_client::{RpcError, RpcResult},
    types::{Pubkey, Signature},
};
use url::Host;

use crate::constants::RPC_HOSTS_BLOCKLIST;

pub fn hostname_from_url(url: &str) -> Option<String> {
    url::Url::parse(url).ok().and_then(|url| match url.host() {
        Some(Host::Domain(domain)) => {
            if !domain.contains(['{', '}']) {
                Some(domain.to_string())
            } else {
                None
            }
        }
        _ => None,
    })
}

pub fn validate_hostname(hostname: &str) -> Result<(), &'static str> {
    if RPC_HOSTS_BLOCKLIST.contains(&hostname) {
        Err("Hostname not allowed")
    } else {
        Ok(())
    }
}

pub fn parse_pubkey(address: &str) -> RpcResult<Pubkey> {
    Pubkey::from_str(address).map_err(|e| RpcError::ParseError(e.to_string()))
}

pub fn parse_pubkeys(addresses: Vec<String>) -> RpcResult<Vec<Pubkey>> {
    addresses.iter().map(|addr| parse_pubkey(addr)).collect()
}

pub fn parse_signature(signature: &str) -> RpcResult<Signature> {
    Signature::from_str(signature).map_err(|e| RpcError::ParseError(e.to_string()))
}

pub fn parse_signatures(signatures: Vec<String>) -> RpcResult<Vec<Signature>> {
    signatures.iter().map(|s| parse_signature(s)).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hostname_from_url() {
        assert_eq!(
            hostname_from_url("https://example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            hostname_from_url("https://example.com?k=v"),
            Some("example.com".to_string())
        );
        assert_eq!(
            hostname_from_url("https://example.com/{API_KEY}"),
            Some("example.com".to_string())
        );
        assert_eq!(
            hostname_from_url("https://example.com/path/{API_KEY}"),
            Some("example.com".to_string())
        );
        assert_eq!(
            hostname_from_url("https://example.com/path/{API_KEY}?k=v"),
            Some("example.com".to_string())
        );
        assert_eq!(hostname_from_url("https://{API_KEY}"), None);
        assert_eq!(hostname_from_url("https://{API_KEY}/path/"), None);
        assert_eq!(hostname_from_url("https://{API_KEY}.com"), None);
        assert_eq!(hostname_from_url("https://{API_KEY}.com/path/"), None);
        assert_eq!(hostname_from_url("https://example.{API_KEY}"), None);
        assert_eq!(hostname_from_url("https://example.{API_KEY}/path/"), None);
    }
}
