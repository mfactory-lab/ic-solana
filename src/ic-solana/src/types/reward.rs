use {
    candid::{CandidType, Deserialize},
    serde::Serialize,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy, CandidType)]
pub enum RewardType {
    Fee,
    Rent,
    Staking,
    Voting,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
#[serde(rename_all = "camelCase")]
pub struct Reward {
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: u64, // Account balance in lamports after `lamports` was applied
    pub reward_type: Option<RewardType>,
    pub commission: Option<u8>, // Vote account commission when the reward was credited, only present for voting and staking rewards
}

pub type Rewards = Vec<Reward>;
