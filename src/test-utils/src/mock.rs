use std::collections::BTreeSet;

use pocket_ic::common::rest::{CanisterHttpHeader, CanisterHttpMethod, CanisterHttpReply, CanisterHttpRequest};

pub struct MockOutcallBody(pub Vec<u8>);

impl From<String> for MockOutcallBody {
    fn from(string: String) -> Self {
        Self(string.into_bytes())
    }
}
impl<'a> From<&'a str> for MockOutcallBody {
    fn from(s: &'a str) -> Self {
        Self(s.as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for MockOutcallBody {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl<'a> From<&'a [u8]> for MockOutcallBody {
    fn from(bytes: &'a [u8]) -> Self {
        Self(bytes.to_vec())
    }
}

#[derive(Clone, Debug)]
pub struct MockOutcallBuilder(MockOutcall);

impl MockOutcallBuilder {
    pub fn new(status: u16, body: impl Into<MockOutcallBody>) -> Self {
        Self(MockOutcall {
            method: None,
            url: None,
            request_headers: None,
            request_body: None,
            max_response_bytes: None,
            response: CanisterHttpReply {
                status,
                headers: vec![],
                body: body.into().0,
            },
        })
    }

    pub fn with_method(mut self, method: CanisterHttpMethod) -> Self {
        self.0.method = Some(method);
        self
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.0.url = Some(url.into());
        self
    }

    pub fn with_request_headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = (S, S)>,
        S: Into<String>,
    {
        self.0.request_headers = Some(
            headers
                .into_iter()
                .map(|(name, value)| CanisterHttpHeader {
                    name: name.into(),
                    value: value.into(),
                })
                .collect(),
        );
        self
    }

    pub fn with_raw_request_body(self, body: &str) -> Self {
        self.with_request_body(MockJsonRequestBody::from_raw_request_unchecked(body))
    }

    pub fn with_request_body(mut self, body: impl Into<MockJsonRequestBody>) -> Self {
        self.0.request_body = Some(body.into());
        self
    }

    pub fn with_max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.0.max_response_bytes = Some(max_response_bytes);
        self
    }

    pub fn with_response_header(mut self, name: String, value: String) -> Self {
        self.0.response.headers.push(CanisterHttpHeader { name, value });
        self
    }

    pub fn build(self) -> MockOutcall {
        self.0
    }
}

impl From<MockOutcallBuilder> for MockOutcall {
    fn from(builder: MockOutcallBuilder) -> Self {
        builder.build()
    }
}

#[derive(Clone, Debug)]
pub struct MockOutcall {
    pub method: Option<CanisterHttpMethod>,
    pub url: Option<String>,
    pub request_headers: Option<Vec<CanisterHttpHeader>>,
    pub request_body: Option<MockJsonRequestBody>,
    pub max_response_bytes: Option<u64>,
    pub response: CanisterHttpReply,
}

impl MockOutcall {
    pub fn assert_matches(&self, request: &CanisterHttpRequest) {
        if let Some(ref url) = self.url {
            assert_eq!(url, &request.url);
        }
        if let Some(ref method) = self.method {
            assert_eq!(method, &request.http_method);
        }
        if let Some(ref headers) = self.request_headers {
            assert_eq!(
                headers.iter().collect::<BTreeSet<_>>(),
                request.headers.iter().collect::<BTreeSet<_>>()
            );
        }
        if let Some(ref expected_body) = self.request_body {
            let actual_body: serde_json::Value =
                serde_json::from_slice(&request.body).expect("Error: failed to parse JSON request body");
            expected_body.assert_matches(&actual_body);
        }
        if let Some(max_response_bytes) = self.max_response_bytes {
            assert_eq!(Some(max_response_bytes), request.max_response_bytes);
        }
    }
}

/// Assertions on parts of the JSON-RPC request body.
#[derive(Clone, Debug)]
pub struct MockJsonRequestBody {
    pub jsonrpc: String,
    pub method: String,
    pub id: Option<u64>,
    pub params: Option<serde_json::Value>,
}

impl MockJsonRequestBody {
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            id: None,
            params: None,
        }
    }

    pub fn builder(method: impl Into<String>) -> MockJsonRequestBuilder {
        MockJsonRequestBuilder(Self::new(method))
    }

    pub fn from_raw_request_unchecked(raw_request: &str) -> Self {
        let request: serde_json::Value =
            serde_json::from_str(raw_request).expect("Error: failed to parse JSON request");
        Self {
            jsonrpc: request["jsonrpc"]
                .as_str()
                .expect("Error: missing jsonrpc field")
                .to_string(),
            method: request["method"]
                .as_str()
                .expect("Error: missing method field")
                .to_string(),
            id: request["id"].as_u64(),
            params: request.get("params").cloned(),
        }
    }

    pub fn assert_matches(&self, request_body: &serde_json::Value) {
        assert_eq!(
            self.jsonrpc,
            request_body["jsonrpc"].as_str().expect("Error: missing jsonrpc field")
        );
        assert_eq!(
            self.method,
            request_body["method"].as_str().expect("Error: missing method field")
        );
        if let Some(id) = self.id {
            assert_eq!(id, request_body["id"].as_u64().expect("Error: missing id field"));
        }
        if let Some(expected_params) = &self.params {
            assert_eq!(
                expected_params,
                request_body.get("params").expect("Error: missing params field")
            );
        }
    }
}

#[derive(Clone, Debug)]
pub struct MockJsonRequestBuilder(MockJsonRequestBody);

impl MockJsonRequestBuilder {
    pub fn with_params(mut self, params: impl Into<serde_json::Value>) -> Self {
        self.0.params = Some(params.into());
        self
    }

    pub fn build(self) -> MockJsonRequestBody {
        self.0
    }
}

impl From<MockJsonRequestBuilder> for MockJsonRequestBody {
    fn from(builder: MockJsonRequestBuilder) -> Self {
        builder.build()
    }
}
