use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddLiquidityError {
    #[error("Add liquidity was called without any tokens")]
    NoTokensProvided,
}

// #[derive(Error, Debug)]
// pub enum AddLiquidityError {
//     #[error("Add liquidity was called without any tokens")]
//     NoTokensProvided,
// }

// #[derive(Error, Debug)]
// pub enum AddLiquidityError {
//     #[error("Add liquidity was called without any tokens")]
//     NoTokensProvided,
// }

// #[derive(Error, Debug)]
// pub enum AddLiquidityError {
//     #[error("Add liquidity was called without any tokens")]
//     NoTokensProvided,
// }
