use std::ops::{Add, Div, Mul, Sub};

use duplicate::duplicate_item;

/// precision selected for our fixed-point decimals
const PRECISION: i32 = 6;
/// alias for u64, allows for easy swapping with other types like u128
pub type Uint = u64;

/// Scale factor of fixed-point decimals
pub const SCALE: Uint = 10u32.pow(PRECISION as u32) as Uint;

#[inline(always)]
/// floating point numbers don't support const functions right now so we need separate function to
/// calculate correct multiplier. Lu
pub fn f64_precision_multiplier() -> f64 {
    SCALE as f64
}

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
/// Token Amount in fixed-point decimal format
pub struct TokenAmount(Uint);

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
/// Staked Token Amount in fixed-point decimal format
pub struct StakedTokenAmount(Uint);

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
/// Lp Token Amount in fixed-point decimal format
pub struct LpTokenAmount(Uint);

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
/// Price of StakedToken in respect to Token in fixed-point decimal format
pub struct Price(Uint);

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
/// Percentage in fixed-point decimal format
pub struct Percentage(Uint);

impl TokenAmount {
    pub fn apply_fee(&self, fee: Percentage) -> TokenAmount {
        TokenAmount::from_raw_amount(self.0 * (SCALE - fee.raw()) / SCALE)
    }
}

impl StakedTokenAmount {
    pub fn into_token_amount(self, price: Price) -> TokenAmount {
        TokenAmount::from_raw_amount(self.raw() * price.raw() / SCALE)
    }
}

impl LpTokenAmount {
    pub fn from_token_amount(
        token_amount: TokenAmount,
        token_total: TokenAmount,
        lp_tokens_total: LpTokenAmount,
    ) -> Self {
        Self::from_raw_amount(
            lp_tokens_total.raw() * (token_amount.raw() * SCALE / token_total.raw()) / SCALE,
        )
    }
}

////////////////////
/// Common Impls ///
////////////////////

// this macro generates the same code for every item in brackets by substituting ImplName with the name from the brackets
#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl ImplName {
    /// takes value as minimal precision units (based on fixed-point decimal precision) and wraps it into appropriate struct
    pub fn from_raw_amount(value: Uint) -> Self {
        Self(value)
    }
    /// returns raw fixed point value
    pub fn raw(&self) -> Uint {
        self.0
    }
}

#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl From<Uint> for ImplName {
    fn from(value: Uint) -> Self {
        Self(value * SCALE)
    }
}

#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl From<f64> for ImplName {
    fn from(value: f64) -> Self {
        let value = value * f64_precision_multiplier();
        let u_value = value as Uint;
        Self(u_value)
    }
}

//////////////////////
/// MATH OPERATORS ///
//////////////////////

#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl Sub for ImplName {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl Add for ImplName {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl Div for ImplName {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 * SCALE / rhs.0)
    }
}

#[duplicate_item(ImplName; [TokenAmount]; [StakedTokenAmount]; [LpTokenAmount]; [Price]; [Percentage])]
impl Mul for ImplName {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0 / SCALE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_AMOUNT: Uint = 1;

    #[test]
    fn can_create_item_from_f64() {
        let token = TokenAmount::from(TEST_AMOUNT as f64);
        assert_eq!(token.0, TEST_AMOUNT * SCALE);
    }
    #[test]
    fn can_create_item_from_uint() {
        let token = TokenAmount::from(TEST_AMOUNT);
        assert_eq!(token.0, TEST_AMOUNT * SCALE);
    }
    #[test]
    fn from_uint_f64_same_token_amounts() {
        let uint_token = TokenAmount::from(2);
        let f64_token = TokenAmount::from(2.);
        assert_eq!(uint_token, f64_token);
    }
}
