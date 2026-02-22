//! Merchant entrypoints: withdraw_merchant_funds.
//!
//! **PRs that only change merchant payouts should edit this file only.**

use crate::safe_math::validate_non_negative;
use crate::types::Error;
use soroban_sdk::{Address, Env};

pub fn withdraw_merchant_funds(_env: &Env, merchant: Address, amount: i128) -> Result<(), Error> {
    merchant.require_auth();
    validate_non_negative(amount)?;
    // TODO: load merchant's accumulated balance from storage, use safe_sub_balance, transfer token to merchant
    Ok(())
}
