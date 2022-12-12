use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;

pub type Address = [u8; 20];

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Execute {
        referral_id: Option<AccountId>,
        /// List of sequential actions.
        actions: Vec<Action>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CoreSwapMessage {
    /// List of sequential actions.
    pub actions: Vec<Action>,
    pub target_account: AccountId,
    pub target_token: Option<AccountId>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LostFoundMessage {
    pub account: AccountId,
    pub is_native: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum Action {
    Swap(SwapAction),
}

/// Single swap action.
#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapParam {
    pub amount_in: U128,
    pub min_amount_out: U128,
    pub path: Vec<u8>,
    pub router_index: U64,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapData {
    pub swap_param: Vec<SwapParam>,
    pub target_token: Vec<u8>,
    pub to_address: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapMsg {
    pub map_target_token: Address,
    pub to_chain: U128,
    pub swap_data_0: SwapData,
    pub swap_data_1: SwapData,
}
