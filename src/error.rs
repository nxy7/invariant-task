use thiserror::Error;

use crate::types::{LpTokenAmount, TokenAmount};

#[derive(Error, Debug)]
pub enum General {}

#[derive(Error, Debug)]
pub enum AddLiquidityError {
    #[error("Add liquidity was called without any tokens")]
    NoTokensProvided,
    #[error("Provided token amount was too big and would cause overflow")]
    TokenAmountTooBig,
}

#[derive(Error, Debug)]
pub enum RemoveLiquidityError {
    #[error("Caller wanted to withdraw {withdraw_amount:?} tokens from the pool that only has {pool_capacity:?}")]
    NotEnoughTokens {
        withdraw_amount: LpTokenAmount,
        pool_capacity: LpTokenAmount,
    },
    #[error("Calculating withdraw amount caused overflow, try using smaller withdraw amount")]
    WithdrawCalculationOverflow,
}

#[derive(Error, Debug)]
pub enum SwapError {
    #[error(
        "Swap call would require {token_amount:?} but pool can only provide {pool_capacity:?}"
    )]
    PoolNotEnoughTokens {
        token_amount: TokenAmount,
        pool_capacity: TokenAmount,
    },
    #[error("Zero tokens were passed as swap argument")]
    ZeroTokensAsArgument,
}
