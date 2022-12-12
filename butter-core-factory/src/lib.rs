use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde_json::json;
use near_sdk::{env, near_bindgen, AccountId, Gas, Promise};

const BUTTER_CORE_BINARY: &'static [u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/butter_core.wasm");

/// This gas spent on the call & account creation, the rest goes to the `new` call.
const CREATE_CALL_GAS: Gas = Gas(200_000_000_000_000);

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct Factory {}

#[near_bindgen]
impl Factory {
    #[payable]
    pub fn create_butter_core(
        &mut self,
        name: String,
        controller: AccountId,
        ref_exchanger: AccountId,
        wrapped_token: AccountId,
        owner: AccountId,
    ) -> Promise {
        let account_id = format!("{}.{}", name, env::current_account_id());
        Promise::new(account_id.parse().unwrap())
            .create_account()
            .deploy_contract(BUTTER_CORE_BINARY.to_vec())
            .transfer(env::attached_deposit())
            .function_call(
                "new".to_string(),
                json!({
                    "controller": controller,
                    "ref_exchanger": ref_exchanger,
                    "wrapped_token": wrapped_token,
                    "owner": owner})
                .to_string()
                .as_bytes()
                .to_vec(),
                0,
                env::prepaid_gas() - CREATE_CALL_GAS,
            )
    }
}
