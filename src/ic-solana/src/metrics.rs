use std::{cell::RefCell, collections::HashMap};

use candid::{CandidType, Deserialize};

use crate::request::RpcRequest;

thread_local! {
    pub static METRICS: RefCell<Metrics> = RefCell::default();
}

#[derive(Clone, Debug, Default, PartialEq, Eq, CandidType, Deserialize)]
pub struct Metrics {
    pub requests: HashMap<(MetricRpcMethod, MetricRpcHost), u64>,
    pub responses: HashMap<(MetricRpcMethod, MetricRpcHost, MetricHttpStatusCode), u64>,
    #[serde(rename = "inconsistentResponses")]
    pub inconsistent_responses: HashMap<(MetricRpcMethod, MetricRpcHost), u64>,
    #[serde(rename = "cyclesCharged")]
    pub cycles_charged: HashMap<(MetricRpcMethod, MetricRpcHost), u128>,
    #[serde(rename = "cyclesWithdrawn")]
    pub cycles_withdrawn: u128,
    #[serde(rename = "auths")]
    pub auths: HashMap<MetricAuth, u128>,
    #[serde(rename = "errNoPermission")]
    pub err_no_permission: u64,
    #[serde(rename = "errUnauthorized")]
    pub err_unauthorized: HashMap<MetricAuth, u128>,
    #[serde(rename = "errHttpOutcall")]
    pub err_http_outcall: HashMap<(MetricRpcMethod, MetricRpcHost), u64>,
    #[serde(rename = "errHostNotAllowed")]
    pub err_host_not_allowed: HashMap<MetricRpcHost, u64>,
}

pub fn encode_metrics(w: &mut ic_metrics_encoder::MetricsEncoder<Vec<u8>>) -> std::io::Result<()> {
    METRICS.with(|m| {
        let m = m.borrow();

        w.gauge_vec("cycle_balance", "Cycle balance of this canister")?.value(
            &[("canister", "sol")],
            ic_cdk::api::canister_balance128().metric_value(),
        )?;
        w.encode_gauge(
            "sol_canister_version",
            ic_cdk::api::canister_version().metric_value(),
            "Canister version",
        )?;
        w.encode_gauge(
            "sol_stable_memory_pages",
            ic_cdk::api::stable::stable_size().metric_value(),
            "Size of the stable memory allocated by this canister measured in 64-bit Wasm pages",
        )?;
        w.counter_entries(
            "sol_cycles_charged",
            &m.cycles_charged,
            "Number of cycles charged for RPC calls",
        );
        w.encode_counter(
            "sol_cycles_withdrawn",
            m.cycles_withdrawn.metric_value(),
            "Number of accumulated cycles withdrawn by RPC providers",
        )?;
        w.gauge_entries(
            "sol_auths",
            &m.auths,
            "Number of active authorizations for canister methods",
        );
        w.counter_entries("sol_requests", &m.requests, "Number of JSON-RPC requests");
        w.counter_entries("sol_responses", &m.responses, "Number of JSON-RPC responses");
        w.counter_entries(
            "sol_inconsistent_responses",
            &m.inconsistent_responses,
            "Number of inconsistent RPC responses",
        );
        w.counter_entries(
            "sol_err_http_outcall",
            &m.err_http_outcall,
            "Number of unsuccessful HTTP outcalls",
        );
        w.counter_entries(
            "sol_err_unauthorized",
            &m.err_unauthorized,
            "Number of unauthorized errors for canister methods",
        );
        w.counter_entries(
            "sol_err_host_not_allowed",
            &m.err_host_not_allowed,
            "Number of HostNotAllowed errors",
        );
        w.encode_counter(
            "sol_err_no_permission",
            m.err_no_permission.metric_value(),
            "Number of NoPermission errors",
        )?;

        Ok(())
    })
}

pub fn read_metrics<R>(f: impl FnOnce(&Metrics) -> R) -> R {
    METRICS.with(|metrics| f(&metrics.borrow()))
}

pub trait MetricValue {
    fn metric_value(&self) -> f64;
}

impl MetricValue for u32 {
    fn metric_value(&self) -> f64 {
        *self as f64
    }
}

impl MetricValue for u64 {
    fn metric_value(&self) -> f64 {
        *self as f64
    }
}

impl MetricValue for u128 {
    fn metric_value(&self) -> f64 {
        *self as f64
    }
}

pub trait MetricLabels {
    fn metric_labels(&self) -> Vec<(&str, &str)>;
}

impl<A: MetricLabels, B: MetricLabels> MetricLabels for (A, B) {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        [self.0.metric_labels(), self.1.metric_labels()].concat()
    }
}

impl<A: MetricLabels, B: MetricLabels, C: MetricLabels> MetricLabels for (A, B, C) {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        [self.0.metric_labels(), self.1.metric_labels(), self.2.metric_labels()].concat()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct MetricMethod(pub String);

impl MetricLabels for MetricMethod {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("method", &self.0)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct MetricAuth(pub String);

impl MetricLabels for MetricAuth {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("auth", &self.0)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct MetricRpcMethod(pub String);

impl From<RpcRequest> for MetricRpcMethod {
    fn from(req: RpcRequest) -> Self {
        MetricRpcMethod(req.to_string())
    }
}

impl MetricLabels for MetricRpcMethod {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("method", &self.0)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct MetricRpcHost(pub String);

impl From<String> for MetricRpcHost {
    fn from(hostname: String) -> Self {
        MetricRpcHost(hostname)
    }
}

impl From<&str> for MetricRpcHost {
    fn from(hostname: &str) -> Self {
        MetricRpcHost(hostname.to_string())
    }
}

impl MetricLabels for MetricRpcHost {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("host", &self.0)]
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, CandidType, Deserialize)]
pub struct MetricHttpStatusCode(pub String);

impl From<u16> for MetricHttpStatusCode {
    fn from(value: u16) -> Self {
        MetricHttpStatusCode(value.to_string())
    }
}

impl MetricLabels for MetricHttpStatusCode {
    fn metric_labels(&self) -> Vec<(&str, &str)> {
        vec![("status", &self.0)]
    }
}

trait EncoderExtensions {
    fn counter_entries<K: MetricLabels, V: MetricValue>(&mut self, name: &str, map: &HashMap<K, V>, help: &str);

    fn gauge_entries<K: MetricLabels, V: MetricValue>(&mut self, name: &str, map: &HashMap<K, V>, help: &str);
}

impl EncoderExtensions for ic_metrics_encoder::MetricsEncoder<Vec<u8>> {
    fn counter_entries<K: MetricLabels, V: MetricValue>(&mut self, name: &str, map: &HashMap<K, V>, help: &str) {
        map.iter().for_each(|(k, v)| {
            self.counter_vec(name, help)
                .and_then(|m| {
                    m.value(&k.metric_labels(), v.metric_value())?;
                    Ok(())
                })
                .unwrap_or(());
        })
    }

    fn gauge_entries<K: MetricLabels, V: MetricValue>(&mut self, name: &str, map: &HashMap<K, V>, help: &str) {
        map.iter().for_each(|(k, v)| {
            self.gauge_vec(name, help)
                .and_then(|m| {
                    m.value(&k.metric_labels(), v.metric_value())?;
                    Ok(())
                })
                .unwrap_or(());
        })
    }
}

#[macro_export]
macro_rules! add_metric {
    ($metric:ident, $amount:expr) => {{
        $crate::metrics::METRICS.with_borrow_mut(|m| m.$metric += $amount);
    }};
}

#[macro_export]
macro_rules! add_metric_entry {
    ($metric:ident, $key:expr, $amount:expr) => {{
        $crate::metrics::METRICS.with_borrow_mut(|m| {
            let amount = $amount;
            if amount != 0 {
                m.$metric
                    .entry($key)
                    .and_modify(|counter| *counter += amount)
                    .or_insert(amount);
            }
        });
    }};
}

#[macro_export]
macro_rules! sub_metric_entry {
    ($metric:ident, $key:expr, $amount:expr) => {{
        $crate::metrics::METRICS.with_borrow_mut(|m| {
            let amount = $amount;
            if amount != 0 {
                m.$metric
                    .entry($key)
                    .and_modify(|counter| *counter = counter.saturating_sub(amount))
                    .or_insert(0);
            }
        });
    }};
}
