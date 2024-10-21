use {
    crate::rpc_client::{RpcError, RpcResult},
    candid::{CandidType, Deserialize},
    ic_canister_log::log,
    serde::Serialize,
};

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
pub enum MultiRpcResult<T> {
    Consistent(RpcResult<T>),
    Inconsistent(Vec<(String, RpcResult<T>)>),
}

impl<T> MultiRpcResult<T> {
    pub fn map<R>(self, mut f: impl FnMut(T) -> R) -> MultiRpcResult<R> {
        match self {
            MultiRpcResult::Consistent(result) => MultiRpcResult::Consistent(result.map(f)),
            MultiRpcResult::Inconsistent(results) => MultiRpcResult::Inconsistent(
                results
                    .into_iter()
                    .map(|(service, result)| {
                        (
                            service,
                            match result {
                                Ok(ok) => Ok(f(ok)),
                                Err(err) => Err(err),
                            },
                        )
                    })
                    .collect(),
            ),
        }
    }

    pub fn consistent(self) -> Option<RpcResult<T>> {
        match self {
            MultiRpcResult::Consistent(result) => Some(result),
            MultiRpcResult::Inconsistent(_) => None,
        }
    }

    pub fn inconsistent(self) -> Option<Vec<(String, RpcResult<T>)>> {
        match self {
            MultiRpcResult::Consistent(_) => None,
            MultiRpcResult::Inconsistent(results) => Some(results),
        }
    }

    pub fn expect_consistent(self) -> RpcResult<T> {
        self.consistent().expect("expected consistent results")
    }

    pub fn expect_inconsistent(self) -> Vec<(String, RpcResult<T>)> {
        self.inconsistent().expect("expected inconsistent results")
    }
}

// #[derive(Debug, PartialEq, Eq)]
// pub enum MultiCallError<T> {
//     ConsistentError(RpcError),
//     InconsistentResults(MultiCallResults<T>),
// }
//
// impl<T: Debug + PartialEq + Clone + Serialize> MultiCallResults<T> {
//     pub fn reduce(self, strategy: ConsensusStrategy) -> anyhow::Result<T, MultiCallError<T>> {
//         match strategy {
//             ConsensusStrategy::Equality => self.reduce_with_equality(),
//             ConsensusStrategy::Threshold { total: _, min } => self.reduce_with_threshold(min),
//         }
//     }
//
//     fn reduce_with_equality(self) -> anyhow::Result<T, MultiCallError<T>> {
//         let mut results = self.all_ok()?.into_iter();
//         let (base_node_provider, base_result) = results
//             .next()
//             .expect("BUG: MultiCallResults is guaranteed to be non-empty");
//         let mut inconsistent_results: Vec<_> = results.filter(|(_provider, result)| result != &base_result).collect();
//         if !inconsistent_results.is_empty() {
//             inconsistent_results.push((base_node_provider, base_result));
//             let error = MultiCallError::InconsistentResults(MultiCallResults::from_non_empty_iter(
//                 inconsistent_results
//                     .into_iter()
//                     .map(|(provider, result)| (provider, Ok(result))),
//             ));
//             log!(INFO, "[reduce_with_equality]: inconsistent results {error:?}");
//             return Err(error);
//         }
//         Ok(base_result)
//     }
//
//     fn reduce_with_threshold(self, min: u8) -> anyhow::Result<T, MultiCallError<T>> {
//         assert!(min > 0, "BUG: min must be greater than 0");
//         if self.ok_results.len() < min as usize {
//             // At least total >= min were queried,
//             // so there is at least one error
//             return Err(self.expect_error());
//         }
//         let distribution = ResponseDistribution::from_non_empty_iter(self.ok_results.clone());
//         let (most_likely_response, providers) = distribution
//             .most_frequent()
//             .expect("BUG: distribution should be non-empty");
//         if providers.len() >= min as usize {
//             Ok(most_likely_response.clone())
//         } else {
//             log!(
//                 INFO,
//                 "[reduce_with_threshold]: too many inconsistent ok responses to reach threshold of {min}, results: {self:?}"
//             );
//             Err(MultiCallError::InconsistentResults(self))
//         }
//     }
// }
