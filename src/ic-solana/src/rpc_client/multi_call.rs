use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

use ic_canister_log::log;
use serde::Serialize;

use crate::{
    logs::INFO,
    rpc_client::types::{ConsensusStrategy, RpcApi, RpcError, RpcResult},
};

/// Aggregates responses of different providers to the same query.
/// Guaranteed to be non-empty.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MultiCallResults<T> {
    ok_results: BTreeMap<RpcApi, T>,
    errors: BTreeMap<RpcApi, RpcError>,
}

impl<T> MultiCallResults<T> {
    pub fn new() -> Self {
        Self {
            ok_results: BTreeMap::new(),
            errors: BTreeMap::new(),
        }
    }

    pub fn from_non_empty_iter<I: IntoIterator<Item = (RpcApi, RpcResult<T>)>>(iter: I) -> Self {
        let mut results = Self::new();
        for (provider, result) in iter {
            results.insert_once(provider, result);
        }
        if results.is_empty() {
            panic!("BUG: MultiCallResults cannot be empty!")
        }
        results
    }

    fn is_empty(&self) -> bool {
        self.ok_results.is_empty() && self.errors.is_empty()
    }

    fn insert_once(&mut self, provider: RpcApi, result: RpcResult<T>) {
        match result {
            Ok(value) => {
                assert!(!self.errors.contains_key(&provider));
                assert!(self.ok_results.insert(provider, value).is_none());
            }
            Err(error) => {
                assert!(!self.ok_results.contains_key(&provider));
                assert!(self.errors.insert(provider, error).is_none());
            }
        }
    }

    pub fn into_vec(self) -> Vec<(RpcApi, RpcResult<T>)> {
        self.ok_results
            .into_iter()
            .map(|(provider, result)| (provider, Ok(result)))
            .chain(self.errors.into_iter().map(|(provider, error)| (provider, Err(error))))
            .collect()
    }

    fn group_errors(&self) -> BTreeMap<&RpcError, BTreeSet<&RpcApi>> {
        let mut errors: BTreeMap<_, _> = BTreeMap::new();
        for (provider, error) in self.errors.iter() {
            errors.entry(error).or_insert_with(BTreeSet::new).insert(provider);
        }
        errors
    }
}

impl<T: PartialEq> MultiCallResults<T> {
    /// Expects all results to be ok or return the following error:
    /// * MultiCallError::ConsistentError: all errors are the same, and there are no ok results.
    /// * MultiCallError::InconsistentResults: in all other cases.
    fn all_ok(self) -> Result<BTreeMap<RpcApi, T>, MultiCallError<T>> {
        if self.errors.is_empty() {
            return Ok(self.ok_results);
        }
        Err(self.expect_error())
    }

    fn expect_error(self) -> MultiCallError<T> {
        let errors = self.group_errors();
        match errors.len() {
            0 => {
                panic!("BUG: errors should be non-empty")
            }
            1 if self.ok_results.is_empty() => {
                MultiCallError::ConsistentError(errors.into_keys().next().unwrap().clone())
            }
            _ => MultiCallError::InconsistentResults(self),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MultiCallError<T> {
    ConsistentError(RpcError),
    InconsistentResults(MultiCallResults<T>),
}

impl<T: Debug + PartialEq + Clone + Serialize> MultiCallResults<T> {
    pub fn reduce(self, strategy: ConsensusStrategy) -> Result<T, MultiCallError<T>> {
        match strategy {
            ConsensusStrategy::Equality => self.reduce_with_equality(),
            ConsensusStrategy::Threshold(min) => self.reduce_with_threshold(min),
        }
    }

    fn reduce_with_equality(self) -> Result<T, MultiCallError<T>> {
        let mut results = self.all_ok()?.into_iter();
        let (base_node_provider, base_result) = results
            .next()
            .expect("BUG: MultiCallResults is guaranteed to be non-empty");

        let mut inconsistent_results: Vec<_> = results.filter(|(_, result)| result != &base_result).collect();
        if !inconsistent_results.is_empty() {
            inconsistent_results.push((base_node_provider, base_result));
            let error = MultiCallError::InconsistentResults(MultiCallResults::from_non_empty_iter(
                inconsistent_results
                    .into_iter()
                    .map(|(provider, result)| (provider, Ok(result))),
            ));
            log!(INFO, "[reduce_with_equality]: inconsistent results {error:?}");
            return Err(error);
        }

        Ok(base_result)
    }

    fn reduce_with_threshold(self, min: u8) -> Result<T, MultiCallError<T>> {
        assert!(min > 0, "BUG: min must be greater than 0");
        if self.ok_results.len() < min as usize {
            // At least total >= min was queried,
            // so there is at least one error
            return Err(self.expect_error());
        }
        let distribution = ResponseDistribution::from_non_empty_iter(self.ok_results.clone());
        let (most_likely_response, providers) = distribution
            .most_frequent()
            .expect("BUG: distribution should be non-empty");
        if providers.len() >= min as usize {
            Ok(most_likely_response.clone())
        } else {
            log!(
                INFO,
                "[reduce_with_threshold]: too many inconsistent ok responses to reach threshold of {min}, results: \
                 {self:?}"
            );
            Err(MultiCallError::InconsistentResults(self))
        }
    }
}

/// Distribution of responses observed from different providers.
///
/// From the API point of view, it emulates a map from a response instance to a set of providers
/// that returned it. At the implementation level, to avoid requiring `T` to have a total order
/// (i.e., must implements `Ord` if it were to be used as keys in a `BTreeMap`) which might not
/// always be meaningful, we use as a key the hash of the serialized response instance.
struct ResponseDistribution<T> {
    hashes: BTreeMap<[u8; 32], T>,
    responses: BTreeMap<[u8; 32], BTreeSet<RpcApi>>,
}

impl<T> Default for ResponseDistribution<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ResponseDistribution<T> {
    pub fn new() -> Self {
        Self {
            hashes: BTreeMap::new(),
            responses: BTreeMap::new(),
        }
    }

    /// Returns the most frequent response and the set of providers that returned it.
    pub fn most_frequent(&self) -> Option<(&T, &BTreeSet<RpcApi>)> {
        self.responses
            .iter()
            .max_by_key(|(_hash, providers)| providers.len())
            .map(|(hash, providers)| (self.hashes.get(hash).expect("BUG: hash should be present"), providers))
    }
}

impl<T: Debug + PartialEq + Serialize> ResponseDistribution<T> {
    pub fn from_non_empty_iter<I: IntoIterator<Item = (RpcApi, T)>>(iter: I) -> Self {
        let mut distribution = Self::new();
        for (provider, result) in iter {
            distribution.insert_once(provider, result);
        }
        distribution
    }

    pub fn insert_once(&mut self, provider: RpcApi, result: T) {
        let hash = ic_sha3::Keccak256::hash(serde_json::to_vec(&result).expect("BUG: failed to serialize"));
        match self.hashes.get(&hash) {
            Some(existing_result) => {
                assert_eq!(
                    existing_result, &result,
                    "BUG: different results once serialized have the same hash"
                );
                let providers = self
                    .responses
                    .get_mut(&hash)
                    .expect("BUG: hash is guaranteed to be present");
                assert!(providers.insert(provider), "BUG: provider is already present");
            }
            None => {
                assert_eq!(self.hashes.insert(hash, result), None);
                let providers = BTreeSet::from_iter(std::iter::once(provider));
                assert_eq!(self.responses.insert(hash, providers), None);
            }
        }
    }
}
