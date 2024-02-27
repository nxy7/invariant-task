use thiserror::Error;

use crate::types::LpTokenAmount;

#[derive(Error, Debug)]
pub enum AddLiquidityError {
    #[error("Add liquidity was called without any tokens")]
    NoTokensProvided,
}

#[derive(Error, Debug)]
pub enum RemoveLiquidityError {
    #[error("Caller wanted to withdraw {withdraw_amount:?} tokens from the pool that only has {pool_capacity:?}")]
    NotEnoughTokens {
        withdraw_amount: LpTokenAmount,
        pool_capacity: LpTokenAmount,
    },
}

#[derive(Error, Debug)]
pub enum SwapError {}
