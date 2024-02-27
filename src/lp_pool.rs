use std::convert::Infallible;

use crate::error::*;
use crate::types::*;

#[derive(Debug)]
/// Unstake Liquidity Pool following marinade protocol
pub struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Percentage,
    max_fee: Percentage,
}

impl LpPool {
    /// Initialized and returns LpPool instance.
    /// Right now init doesn't have any extra logic so it's
    /// effectively infallible function.
    pub fn init(
        price: Price,
        min_fee: Percentage,
        max_fee: Percentage,
        liquidity_target: TokenAmount,
    ) -> Result<Self, Infallible> {
        Ok(Self {
            price,
            token_amount: TokenAmount::from(0),
            st_token_amount: StakedTokenAmount::from(0),
            lp_token_amount: LpTokenAmount::from(0),
            min_fee,
            max_fee,
            liquidity_target,
        })
    }

    /// Returns Amount of LP tokens granted to the caller.
    ///
    /// # Arguments
    ///
    /// * `token_amount_in` - amount of 'unstaked' tokens provided by the caller
    pub fn add_liquidity(
        &mut self,
        token_amount_in: TokenAmount,
    ) -> Result<LpTokenAmount, AddLiquidityError> {
        if token_amount_in.raw() == 0 {
            return Err(AddLiquidityError::NoTokensProvided);
        }

        let lp_tokens_raw_amount = match self.lp_token_amount.raw() {
            0 => token_amount_in.raw(),
            lp_amount => {
                let Some(checked_mul) = lp_amount.checked_mul(token_amount_in.raw()) else {
                    return Err(AddLiquidityError::TokenAmountTooBig);
                };
                checked_mul / self.total_val().raw()
            }
        };
        let lp_amount = LpTokenAmount::from_raw_amount(lp_tokens_raw_amount);

        self.token_amount = self.token_amount + token_amount_in;
        self.lp_token_amount = self.lp_token_amount + lp_amount;

        Ok(lp_amount)
    }

    /// Returns tuple consisting of unstaked and staked token amounts withdrawn from the pool.
    ///
    /// # Arguments
    ///
    /// * `lp_amount_out` - lp token amount that the caller wants to withdraw from the pool
    pub fn remove_liquidity(
        &mut self,
        lp_amount_out: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), RemoveLiquidityError> {
        if lp_amount_out > self.lp_token_amount {
            return Err(RemoveLiquidityError::NotEnoughTokens {
                withdraw_amount: lp_amount_out,
                pool_capacity: self.lp_token_amount,
            });
        }

        let calculate_raw_out = |raw_amount: Uint| {
            let Some(checked_mul) = raw_amount.checked_mul(lp_amount_out.raw()) else {
                return Err(RemoveLiquidityError::WithdrawCalculationOverflow);
            };
            Ok(checked_mul / self.lp_token_amount.raw())
        };

        let token_out = TokenAmount::from_raw_amount(calculate_raw_out(self.token_amount.raw())?);
        let staked_out =
            StakedTokenAmount::from_raw_amount(calculate_raw_out(self.st_token_amount.raw())?);

        self.token_amount = self.token_amount - token_out;
        self.st_token_amount = self.st_token_amount - staked_out;
        self.lp_token_amount = self.lp_token_amount - lp_amount_out;

        Ok((token_out, staked_out))
    }

    /// Returns amount of tokens granted to the person executing swap.
    ///
    /// # Arguments
    ///
    /// * `swap_amount` - amount of staked tokens in incoming swap
    pub fn swap(&mut self, swap_amount: StakedTokenAmount) -> Result<TokenAmount, SwapError> {
        if swap_amount.raw() == 0 {
            return Err(SwapError::ZeroTokensAsArgument);
        }

        let amount_out_before_fees = swap_amount.into_token_amount(self.price);
        if amount_out_before_fees > self.token_amount {
            return Err(SwapError::PoolNotEnoughTokens {
                token_amount: amount_out_before_fees,
                pool_capacity: self.token_amount,
            });
        }

        let fee = self.fee(self.token_amount - amount_out_before_fees);

        let amount_out = amount_out_before_fees.apply_fee(fee);

        self.token_amount = self.token_amount - amount_out;
        self.st_token_amount = self.st_token_amount + swap_amount;

        Ok(amount_out)
    }

    /// Returns total value stored inside the pool (tokens + staked tokens) as `TokenAmount`
    fn total_val(&self) -> TokenAmount {
        let staked_value =
            TokenAmount::from_raw_amount(self.st_token_amount.raw() * self.price.raw() / SCALE);
        self.token_amount + staked_value
    }

    /// Returns pool swap percentage fee.
    ///
    /// # Arguments
    ///
    /// * `amount_after` - Token amount after operation
    fn fee(&self, amount_after: TokenAmount) -> Percentage {
        // FEE FORMULA
        // fee = max_fee - (max_fee - min_fee) * amount_after / target
        let rhs =
            (self.max_fee - self.min_fee).raw() * amount_after.raw() / self.liquidity_target.raw();
        let rhs = rhs.min(self.max_fee.raw());

        // we're capping rhs to max_fee so there's no need to check if current_percentage is over it later on
        // and we avoid overflows
        let current_percentage = (self.max_fee.raw() - rhs).max(self.min_fee.raw());
        Percentage::from_raw_amount(current_percentage)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use rstest::{fixture, rstest};

    use super::*;

    #[fixture]
    fn story_example_pool() -> LpPool {
        LpPool {
            price: 1.5.into(),
            token_amount: 0.into(),
            st_token_amount: 0.into(),
            lp_token_amount: 0.into(),
            liquidity_target: 90.into(),
            min_fee: 0.001.into(),
            max_fee: 0.09.into(),
        }
    }

    #[fixture]
    fn empty_pool() -> LpPool {
        LpPool {
            price: 2.into(),
            token_amount: 0.into(),
            st_token_amount: 0.into(),
            lp_token_amount: 0.into(),
            liquidity_target: 100.into(),
            min_fee: 0.0.into(),
            max_fee: 0.09.into(),
        }
    }

    #[fixture]
    fn non_empty_pool() -> LpPool {
        LpPool {
            price: 5.into(),
            token_amount: (2 as Uint).pow(20).into(),
            st_token_amount: 30.into(),
            lp_token_amount: 250.into(),
            liquidity_target: 100.into(),
            min_fee: 0.1.into(),
            max_fee: 0.2.into(),
        }
    }

    #[rstest]
    fn can_calculate_fees(empty_pool: LpPool, non_empty_pool: LpPool) {
        assert_eq!(empty_pool.fee(0.into()).raw(), Percentage::from(0.09).raw());
        assert_eq!(
            empty_pool.fee(100.into()).raw(),
            Percentage::from(0.0).raw()
        );
        assert_eq!(
            empty_pool.fee(50.into()).raw(),
            Percentage::from(0.045).raw()
        );

        assert_eq!(
            non_empty_pool.fee(0.into()).raw(),
            Percentage::from(0.2).raw()
        );
        assert_eq!(
            non_empty_pool.fee(100.into()).raw(),
            Percentage::from(0.1).raw()
        );
        assert_eq!(
            non_empty_pool.fee(50.into()).raw(),
            Percentage::from(0.15).raw()
        );
    }

    #[rstest]
    fn can_add_liquidity(mut empty_pool: LpPool) -> Result<(), Box<dyn Error>> {
        let added = empty_pool.add_liquidity(TokenAmount::from(20))?;
        assert_eq!(
            added,
            LpTokenAmount::from(20),
            "initial liquidity added should match token amount added"
        );

        Ok(())
    }

    #[rstest]
    fn errors_on_empty_add_liquidity(mut story_example_pool: LpPool) {
        assert!(
            story_example_pool
                .add_liquidity(TokenAmount::from_raw_amount(0))
                .is_err(),
            "adding zero liquidity should fail"
        )
    }

    #[rstest]
    fn can_remove_liquidity(mut non_empty_pool: LpPool) -> Result<(), Box<dyn Error>> {
        let res = non_empty_pool.remove_liquidity(LpTokenAmount::from(10))?;
        assert_ne!(res.0, TokenAmount::from(0), "removing liquidity from the pool consisting of both assets should not yield zero value");
        assert_ne!(res.1, StakedTokenAmount::from(0), "removing liquidity from the pool consisting of both assets should not yield zero value");
        Ok(())
    }

    #[rstest]
    fn errors_on_remove_liquidity_bigger_than_pool(mut empty_pool: LpPool) {
        let res = empty_pool.remove_liquidity(LpTokenAmount::from(1000));
        assert!(res.is_err());
    }

    #[rstest]
    fn can_execute_swap(mut non_empty_pool: LpPool) -> Result<(), Box<dyn Error>> {
        let swap_result = non_empty_pool.swap(StakedTokenAmount::from(3))?;
        assert_ne!(
            swap_result,
            TokenAmount::from(0),
            "successful swap should result in non-zero token amount granted to the caller"
        );
        Ok(())
    }

    #[rstest]
    fn swap_errors_on_not_enough_tokens(mut empty_pool: LpPool) {
        let swap_result = empty_pool.swap(StakedTokenAmount::from(3));
        assert!(
            swap_result.is_err(),
            "swap on empty pool should always yield err"
        );
    }

    #[rstest]
    fn swap_errors_on_zero_token_argument(mut empty_pool: LpPool) {
        let swap_result = empty_pool.swap(StakedTokenAmount::from(0));
        assert!(
            swap_result.is_err(),
            "swap on empty pool should always yield err"
        );
    }

    #[rstest]
    fn story_example(mut story_example_pool: LpPool) -> Result<(), Box<dyn Error>> {
        assert_eq!(
            story_example_pool.add_liquidity(TokenAmount::from(100))?,
            LpTokenAmount::from(100),
            "initial add liquidity"
        );
        assert_eq!(
            story_example_pool.swap(StakedTokenAmount::from(6))?,
            TokenAmount::from(8.991),
            "first swap"
        );
        assert_eq!(
            story_example_pool.add_liquidity(TokenAmount::from(10))?,
            LpTokenAmount::from(9.9991),
            "second add liquidity"
        );
        assert_eq!(
            story_example_pool.swap(StakedTokenAmount::from(30))?,
            TokenAmount::from(43.44237),
            "second swap"
        );

        let remove_liquidity_result =
            story_example_pool.remove_liquidity(LpTokenAmount::from(109.9991))?;
        assert_eq!(
            remove_liquidity_result.0,
            TokenAmount::from(57.56663),
            "withdraw"
        );
        assert_eq!(
            remove_liquidity_result.1,
            StakedTokenAmount::from(36),
            "withdraw"
        );
        Ok(())
    }
}
