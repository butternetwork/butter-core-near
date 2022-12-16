mod types;

use crate::types::{Action, CoreSwapMessage, TokenReceiverMessage};
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::panic_str;
use near_sdk::json_types::U128;
use near_sdk::{
    env, ext_contract, log, near_bindgen, serde_json, AccountId, Balance, Gas, PanicOnDefault,
    Promise, PromiseOrValue, PromiseResult,
};

/// Gas to call ft_transfer_call method.
const FT_TRANSFER_CALL_REF_GAS: Gas = Gas(86_000_000_000_000);
/// Gas to call ft_transfer_call method.
const FT_TRANSFER_CALL_MOS_GAS: Gas = Gas(35_000_000_000_000);
/// Gas to call ft_transfer_call method.
// const FT_TRANSFER_CALL_LOST_FOUND_GAS: Gas = Gas(35_000_000_000_000);
/// Gas to call ft_transfer method.
const FT_TRANSFER_GAS: Gas = Gas(4_000_000_000_000);
/// Gas to call ft_balance_of method.
const FT_BALANCE_OF_GAS: Gas = Gas(4_000_000_000_000);
/// Gas to call near_withdraw on wrap near contract
const NEAR_WITHDRAW_GAS: Gas = Gas(4_000_000_000_000);
/// Gas to call near_deposit on wrap near contract
const NEAR_DEPOSIT_GAS: Gas = Gas(7_000_000_000_000);
/// Gas to call callback_return_value on wrap near contract
const CALLBACK_RETURN_VALUE_GAS: Gas = Gas(3_000_000_000_000);
/// Gas to call callback_get_amount_out method, not include gas used in cross contract call.
const CALLBACK_GET_AMOUNT_OUT_GAS: Gas = Gas(10_000_000_000_000);
/// Gas to call callback_transfer_to_target_account method.
const CALLBACK_TRANSFER_TO_TARGET_ACCOUNT_SWAP_IN_GAS: Gas =
    Gas(10_000_000_000_000 + NEAR_WITHDRAW_GAS.0 + CALLBACK_CHECK_TRANSFER_GAS.0);
const CALLBACK_TRANSFER_TO_TARGET_ACCOUNT_SWAP_OUT_GAS: Gas =
    Gas(10_000_000_000_000 + FT_TRANSFER_CALL_MOS_GAS.0 + CALLBACK_RETURN_VALUE_GAS.0);
/// Gas to call callback_check_transfer method.
const CALLBACK_CHECK_TRANSFER_GAS: Gas = Gas(8_000_000_000_000 + FT_TRANSFER_GAS.0);

#[ext_contract(ext_wnear_token)]
pub trait ExtWNearToken {
    fn near_deposit(&mut self);
    fn near_withdraw(&mut self, amount: U128) -> Promise;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ButterCore {
    pub controller: AccountId,
    pub ref_exchange: AccountId,
    pub wrapped_token: AccountId,
    pub owner: AccountId,
}

#[near_bindgen]
impl ButterCore {
    #[init]
    pub fn new(
        controller: AccountId,
        ref_exchange: AccountId,
        wrapped_token: AccountId,
        owner: AccountId,
    ) -> Self {
        Self {
            controller,
            ref_exchange,
            wrapped_token,
            owner,
        }
    }

    pub fn get_controller(&self) -> AccountId {
        self.controller.clone()
    }

    pub fn set_controller(&mut self, controller: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller"
        );
        self.controller = controller;
    }

    pub fn get_ref_exchange(&self) -> AccountId {
        self.ref_exchange.clone()
    }

    pub fn set_ref_exchange(&mut self, ref_exchange: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller"
        );
        self.ref_exchange = ref_exchange;
    }

    pub fn get_wrapped_token(&self) -> AccountId {
        self.wrapped_token.clone()
    }

    pub fn set_wrapped_token(&mut self, wrapped_token: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller"
        );
        self.wrapped_token = wrapped_token;
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn set_owner(&mut self, owner: AccountId) {
        assert_eq!(
            self.owner,
            env::predecessor_account_id(),
            "unexpected caller"
        );
        self.owner = owner;
    }

    fn do_swap(
        &self,
        token: AccountId,
        amount: U128,
        token_out: AccountId,
        token_receiver_msg: TokenReceiverMessage,
        target_account: AccountId,
        target_token: Option<AccountId>,
        direct_call: bool,
    ) -> Promise {
        let msg = serde_json::to_string(&token_receiver_msg).unwrap();

        ext_ft_core::ext(token.clone())
            .with_static_gas(FT_TRANSFER_CALL_REF_GAS)
            .with_attached_deposit(1)
            .ft_transfer_call(self.ref_exchange.clone(), amount, None, msg)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(if target_token.is_none() {
                        Gas(CALLBACK_GET_AMOUNT_OUT_GAS.0
                            + FT_BALANCE_OF_GAS.0
                            + CALLBACK_TRANSFER_TO_TARGET_ACCOUNT_SWAP_OUT_GAS.0)
                    } else {
                        Gas(CALLBACK_GET_AMOUNT_OUT_GAS.0
                            + FT_BALANCE_OF_GAS.0
                            + CALLBACK_TRANSFER_TO_TARGET_ACCOUNT_SWAP_IN_GAS.0)
                    })
                    .callback_get_amount_out(
                        token,
                        amount,
                        token_out,
                        target_account,
                        target_token,
                        direct_call,
                    ),
            )
    }

    #[private]
    pub fn callback_get_amount_out(
        &self,
        token_in: AccountId,
        amount: U128,
        token_out: AccountId,
        target_account: AccountId,
        target_token: Option<AccountId>,
        direct_call: bool,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(x) => {
                let used_amount = serde_json::from_slice::<U128>(&x).unwrap();
                if amount != used_amount {
                    log!("used amount is unexpected, swap in ref exchange failed, expected: {:?}, actual: {:?}!", amount, used_amount);
                    if direct_call {
                        ext_ft_core::ext(token_in)
                            .with_static_gas(FT_TRANSFER_GAS)
                            .with_attached_deposit(1)
                            .ft_transfer(
                                self.controller.clone(),
                                U128(amount.0 - used_amount.0),
                                None,
                            )
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(CALLBACK_RETURN_VALUE_GAS)
                                    .callback_return_value(used_amount),
                            )
                            .into()
                    } else {
                        PromiseOrValue::Value(U128(amount.0 - used_amount.0))
                    }
                } else {
                    ext_ft_core::ext(token_out.clone())
                        .with_static_gas(FT_BALANCE_OF_GAS)
                        .ft_balance_of(env::current_account_id())
                        .then(
                            Self::ext(env::current_account_id())
                                .with_static_gas(if target_token.is_none() {
                                    CALLBACK_TRANSFER_TO_TARGET_ACCOUNT_SWAP_OUT_GAS
                                } else {
                                    CALLBACK_TRANSFER_TO_TARGET_ACCOUNT_SWAP_IN_GAS
                                })
                                .callback_transfer_to_target_account(
                                    token_out,
                                    target_account,
                                    target_token,
                                    amount,
                                    direct_call,
                                ),
                        )
                        .into()
                }
            }
            PromiseResult::Failed => panic_str("call ref exchange failed"),
        }
    }

    #[private]
    pub fn callback_return_value(&self, amount: U128) -> U128 {
        amount
    }

    #[private]
    pub fn callback_transfer_to_target_account(
        &self,
        token_out: AccountId,
        target_account: AccountId,
        target_token_opt: Option<AccountId>,
        amount_in: U128,
        direct_call: bool,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(x) => {
                let amount_out = serde_json::from_slice::<U128>(&x).unwrap();
                if amount_out.0 == 0 {
                    log!("!!!caution: amount out should not be zero!!!");
                    return PromiseOrValue::Value(if direct_call { amount_in } else { U128(0) });
                }
                if let Some(_target_token) = target_token_opt {
                    // swap in
                    // native token
                    if self.wrapped_token == token_out {
                        // near_withdraw() won't fail because the core account has been registered and it has a positive "amount_out" token
                        ext_wnear_token::ext(self.wrapped_token.clone())
                            .with_static_gas(NEAR_WITHDRAW_GAS)
                            .with_attached_deposit(1)
                            .near_withdraw(amount_out)
                            .then(
                                Promise::new(target_account.clone())
                                    .transfer(Balance::from(amount_out)),
                            )
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(CALLBACK_CHECK_TRANSFER_GAS)
                                    .callback_check_transfer(
                                        token_out,
                                        target_account,
                                        amount_in,
                                        amount_out,
                                        true,
                                    ),
                            )
                            .into()
                    } else {
                        ext_ft_core::ext(token_out.clone())
                            .with_static_gas(FT_TRANSFER_GAS)
                            .with_attached_deposit(1)
                            .ft_transfer(target_account.clone(), amount_out, None)
                            .then(
                                Self::ext(env::current_account_id())
                                    .with_static_gas(CALLBACK_CHECK_TRANSFER_GAS)
                                    .callback_check_transfer(
                                        token_out,
                                        target_account,
                                        amount_in,
                                        amount_out,
                                        false,
                                    ),
                            )
                            .into()
                    }
                } else {
                    // swap out
                    // // always succeed because we give enough gas and MOS has been registered in token_out
                    ext_ft_core::ext(token_out)
                        .with_static_gas(FT_TRANSFER_CALL_MOS_GAS)
                        .with_attached_deposit(1)
                        .ft_transfer_call(target_account, amount_out, None, "".to_string())
                        .then(
                            Self::ext(env::current_account_id())
                                .with_static_gas(CALLBACK_RETURN_VALUE_GAS)
                                .callback_return_value(if direct_call {
                                    amount_in
                                } else {
                                    U128(0)
                                }),
                        )
                        .into()
                }
            }
            // actually get balance won't fail if we give enough gas
            PromiseResult::Failed => panic_str("get token_out balance of core failed"),
        }
    }

    #[private]
    pub fn callback_check_transfer(
        &self,
        token: AccountId,
        account: AccountId,
        amount_in: U128,
        amount: U128,
        is_native: bool,
    ) -> U128 {
        assert_eq!(
            1,
            env::promise_results_count(),
            "promise has too many results"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_x) => {}
            PromiseResult::Failed => {
                // if transfer to user failed, transfer to mos
                if is_native {
                    // ext_wnear_token::ext(self.wrapped_token.clone())
                    //     .with_static_gas(NEAR_DEPOSIT_GAS)
                    //     .with_attached_deposit(amount.0)
                    //     .near_deposit()
                    //     .then(
                    //         ext_ft_core::ext(token)
                    //             .with_static_gas(FT_TRANSFER_GAS)
                    //             .with_attached_deposit(1)
                    //             .ft_transfer(self.controller.clone(), amount, Some(memo)),
                    //     );
                    log!(format!(
                        "transfer NEAR to user {} failed, transfer to mos instead",
                        account
                    ));
                    Promise::new(self.controller.clone()).transfer(amount.0);
                } else {
                    let memo = format!(
                        "transfer {} to user {} failed, transfer to mos instead",
                        token, account
                    );
                    ext_ft_core::ext(token)
                        .with_static_gas(FT_TRANSFER_GAS)
                        .with_attached_deposit(1)
                        .ft_transfer(self.controller.clone(), amount, Some(memo));
                }
            }
        }
        amount_in
    }

    pub fn swap(&mut self, amount: U128, core_swap_msg: CoreSwapMessage) -> PromiseOrValue<U128> {
        assert_eq!(
            self.controller,
            env::predecessor_account_id(),
            "unexpected caller, caller should be {}",
            self.controller
        );

        let Action::Swap(first_swap_action) = core_swap_msg.actions.first().unwrap().clone();
        let Action::Swap(last_swap_action) = core_swap_msg.actions.last().unwrap().clone();
        let token_receiver_msg = TokenReceiverMessage::Execute {
            referral_id: None,
            actions: core_swap_msg.actions,
        };

        PromiseOrValue::from(self.do_swap(
            first_swap_action.token_in,
            amount,
            last_swap_action.token_out,
            token_receiver_msg,
            core_swap_msg.target_account,
            core_swap_msg.target_token,
            true,
        ))
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for ButterCore {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            self.controller, sender_id,
            "unexpected caller, caller should be {}",
            self.controller
        );

        let core_swap_msg =
            serde_json::from_str::<CoreSwapMessage>(&msg).expect("unexpected core swap msg format");
        let token = env::predecessor_account_id();

        let Action::Swap(swap_action) = core_swap_msg.actions.last().unwrap().clone();
        let token_receiver_msg = TokenReceiverMessage::Execute {
            referral_id: None,
            actions: core_swap_msg.actions,
        };

        PromiseOrValue::from(self.do_swap(
            token,
            amount,
            swap_action.token_out.clone(),
            token_receiver_msg,
            core_swap_msg.target_account,
            core_swap_msg.target_token,
            false,
        ))
    }
}
