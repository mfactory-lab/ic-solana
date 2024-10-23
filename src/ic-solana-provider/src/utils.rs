use {crate::constants::RPC_HOSTS_BLOCKLIST, url::Host};

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
