use std::error::Error;

use crate::error::*;
use crate::types::*;

#[derive(Debug)]
struct LpPool {
    price: Price,
    token_amount: TokenAmount,
    st_token_amount: StakedTokenAmount,
    lp_token_amount: LpTokenAmount,
    liquidity_target: TokenAmount,
    min_fee: Percentage,
    max_fee: Percentage,
}

impl LpPool {
    pub fn init(
        price: Price,
        min_fee: Percentage,
        max_fee: Percentage,
        liquidity_target: TokenAmount,
    ) -> Result<Self, Box<dyn Error>> {
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

    pub fn add_liquidity(
        &mut self,
        token_amount: TokenAmount,
    ) -> Result<LpTokenAmount, AddLiquidityError> {
        if token_amount.raw() == 0 {
            return Err(AddLiquidityError::NoTokensProvided);
        }
        let lp_tokens_raw_amount = match self.lp_token_amount.raw() {
            0 => token_amount.raw(),
            lp_amount => lp_amount * token_amount.raw() / self.total_val().raw(),
        };
        let lp_amount = LpTokenAmount::from_raw_amount(lp_tokens_raw_amount);
        dbg!(lp_amount);

        self.token_amount = self.token_amount + token_amount;
        self.lp_token_amount = self.lp_token_amount + lp_amount;

        Ok(lp_amount)
    }

    pub fn remove_liquidity(
        &mut self,
        lp_token_amount: LpTokenAmount,
    ) -> Result<(TokenAmount, StakedTokenAmount), Box<dyn Error>> {
        if lp_token_amount > self.lp_token_amount {
            return Err("pool doesn't have enough LP tokens".into());
        }

        let calculate_raw_out =
            |raw_amount: Uint| raw_amount * lp_token_amount.raw() / self.lp_token_amount.raw();

        let token_out = TokenAmount::from_raw_amount(calculate_raw_out(self.token_amount.raw()));
        let staked_out =
            StakedTokenAmount::from_raw_amount(calculate_raw_out(self.st_token_amount.raw()));

        self.token_amount = self.token_amount - token_out;
        self.st_token_amount = self.st_token_amount - staked_out;

        Ok((token_out, staked_out))
    }

    pub fn swap(
        &mut self,
        staked_amount_out: StakedTokenAmount,
    ) -> Result<TokenAmount, Box<dyn Error>> {
        let amount_out_before_fees = staked_amount_out.into_token_amount(self.price);
        let fee = self.fee(self.token_amount - amount_out_before_fees);

        let amount_out = amount_out_before_fees.apply_fee(fee);

        self.token_amount = self.token_amount - amount_out;
        self.st_token_amount = self.st_token_amount + staked_amount_out;

        Ok(amount_out)
    }

    /// total pool value as TokenAmount
    fn total_val(&self) -> TokenAmount {
        let staked_value =
            TokenAmount::from_raw_amount(self.st_token_amount.raw() * self.price.raw() / SCALE);
        self.token_amount + staked_value
    }

    // returns information on how much Token is each LpToken worth
    fn new_lp_token_value(&self, tokens_added: TokenAmount) -> TokenAmount {
        let assets_total = match (self.total_val() + tokens_added).raw() {
            0 => return TokenAmount::from(1),
            v => v,
        };
        let lp_tokens = self.lp_token_amount.raw();
        TokenAmount::from_raw_amount(lp_tokens * SCALE / assets_total)
    }

    /// returns pool Percentage fee
    fn fee(&self, amount_after: TokenAmount) -> Percentage {
        // UNSTAKE FORMULA
        // unstake_fee = max_fee - (max_fee - min_fee) * amount_after / target
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
    fn pool_1() -> LpPool {
        LpPool {
            price: 1.into(),
            token_amount: 0.into(),
            st_token_amount: 0.into(),
            lp_token_amount: 0.into(),
            liquidity_target: 100.into(),
            min_fee: 0.0.into(),
            max_fee: 0.09.into(),
        }
    }

    #[fixture]
    fn pool_2() -> LpPool {
        LpPool {
            price: 1.into(),
            token_amount: 0.into(),
            st_token_amount: 0.into(),
            lp_token_amount: 0.into(),
            liquidity_target: 100.into(),
            min_fee: 0.1.into(),
            max_fee: 0.2.into(),
        }
    }

    #[rstest]
    fn can_calculate_fees(pool_1: LpPool, pool_2: LpPool) {
        assert_eq!(pool_1.fee(0.into()).raw(), Percentage::from(0.09).raw());
        assert_eq!(pool_1.fee(100.into()).raw(), Percentage::from(0.0).raw());
        assert_eq!(pool_1.fee(50.into()).raw(), Percentage::from(0.045).raw());

        assert_eq!(pool_2.fee(0.into()).raw(), Percentage::from(0.2).raw());
        assert_eq!(pool_2.fee(100.into()).raw(), Percentage::from(0.1).raw());
        assert_eq!(pool_2.fee(50.into()).raw(), Percentage::from(0.15).raw());
    }

    #[rstest]
    fn can_add_liquidity() {}
    #[rstest]
    fn errors_on_add_liquidity() {}

    #[rstest]
    fn can_remove_liquidity() {}

    #[rstest]
    fn can_execute_swap() {}

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
