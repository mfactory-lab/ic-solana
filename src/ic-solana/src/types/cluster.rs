use std::{error::Error, fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cluster {
    Testnet,
    Mainnet,
    Devnet,
    #[default]
    Localnet,
    Debug,
    Custom(String, String),
}

#[derive(Debug)]
pub enum ClusterError {
    InvalidCluster,
    UrlParseError(url::ParseError),
    SetPortError,
    SetSchemeError,
}

impl fmt::Display for ClusterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClusterError::InvalidCluster => write!(f, "Invalid cluster"),
            ClusterError::UrlParseError(e) => write!(f, "URL parse error: {}", e),
            ClusterError::SetPortError => write!(f, "Unable to set port"),
            ClusterError::SetSchemeError => write!(f, "Unable to set scheme"),
        }
    }
}

impl Error for ClusterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClusterError::UrlParseError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<url::ParseError> for ClusterError {
    fn from(err: url::ParseError) -> ClusterError {
        ClusterError::UrlParseError(err)
    }
}

impl FromStr for Cluster {
    type Err = ClusterError;

    fn from_str(s: &str) -> Result<Cluster, ClusterError> {
        match s.to_ascii_lowercase().as_str() {
            "t" | "testnet" => Ok(Cluster::Testnet),
            "m" | "mainnet" => Ok(Cluster::Mainnet),
            "d" | "devnet" => Ok(Cluster::Devnet),
            "l" | "localnet" => Ok(Cluster::Localnet),
            "g" | "debug" => Ok(Cluster::Debug),
            _ if s.starts_with("http") => {
                let mut ws_url = Url::parse(s)?;
                if let Some(port) = ws_url.port() {
                    ws_url
                        .set_port(Some(port + 1))
                        .map_err(|_| ClusterError::SetPortError)?;
                }
                if ws_url.scheme() == "https" {
                    ws_url.set_scheme("wss").map_err(|_| ClusterError::SetSchemeError)?;
                } else {
                    ws_url.set_scheme("ws").map_err(|_| ClusterError::SetSchemeError)?;
                }
                Ok(Cluster::Custom(s.to_string(), ws_url.to_string()))
            }
            _ => Err(ClusterError::InvalidCluster),
        }
    }
}

impl fmt::Display for Cluster {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cluster_str = match self {
            Cluster::Testnet => "testnet",
            Cluster::Mainnet => "mainnet",
            Cluster::Devnet => "devnet",
            Cluster::Localnet => "localnet",
            Cluster::Debug => "debug",
            Cluster::Custom(url, _ws_url) => url,
        };
        write!(f, "{cluster_str}")
    }
}

impl Cluster {
    pub fn host_str(&self) -> Option<String> {
        Url::parse(self.url())
            .ok()
            .and_then(|u| u.host_str().map(|host| host.to_string()))
    }

    pub fn url(&self) -> &str {
        match self {
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Testnet => "https://api.testnet.solana.com",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
            Cluster::Localnet => "http://127.0.0.1:8899",
            Cluster::Debug => "http://34.90.18.145:8899",
            Cluster::Custom(url, _ws_url) => url,
        }
    }

    pub fn ws_url(&self) -> &str {
        match self {
            Cluster::Devnet => "wss://api.devnet.solana.com",
            Cluster::Testnet => "wss://api.testnet.solana.com",
            Cluster::Mainnet => "wss://api.mainnet-beta.solana.com",
            Cluster::Localnet => "ws://127.0.0.1:8900",
            Cluster::Debug => "ws://34.90.18.145:8900",
            Cluster::Custom(_url, ws_url) => ws_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cluster(name: &str, cluster: Cluster) {
        assert_eq!(Cluster::from_str(name).unwrap(), cluster);
    }

    #[test]
    fn test_cluster_parse() {
        test_cluster("testnet", Cluster::Testnet);
        test_cluster("mainnet", Cluster::Mainnet);
        test_cluster("devnet", Cluster::Devnet);
        test_cluster("localnet", Cluster::Localnet);
        test_cluster("debug", Cluster::Debug);
    }

    #[test]
    #[should_panic]
    fn test_cluster_bad_parse() {
        let bad_url = "httq://my_custom_url.test.net";
        Cluster::from_str(bad_url).unwrap();
    }

    #[test]
    fn test_http_port() {
        let url = "http://my-url.com:7000/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "ws://my-url.com:7001/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_http_no_port() {
        let url = "http://my-url.com/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "ws://my-url.com/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_https_port() {
        let url = "https://my-url.com:7000/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "wss://my-url.com:7001/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_https_no_port() {
        let url = "https://my-url.com/";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "wss://my-url.com/".to_string()),
            cluster
        );
    }

    #[test]
    fn test_upper_case() {
        let url = "http://my-url.com/FooBar";
        let cluster = Cluster::from_str(url).unwrap();
        assert_eq!(
            Cluster::Custom(url.to_string(), "ws://my-url.com/FooBar".to_string()),
            cluster
        );
    }
}
