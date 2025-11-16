use alloy_primitives::{Address, U256};
use bon::Builder;

/// Parameters for bridging USDC
#[derive(Builder, Debug, Clone)]
pub struct BridgeParams {
    from_address: Address,
    recipient: Address,
    token_address: Address,
    amount: U256,
}

impl BridgeParams {
    pub fn from_address(&self) -> Address {
        self.from_address
    }

    pub fn recipient(&self) -> Address {
        self.recipient
    }

    pub fn token_address(&self) -> Address {
        self.token_address
    }

    pub fn amount(&self) -> U256 {
        self.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};

    #[test]
    fn test_bridge_params_builder() {
        let params = BridgeParams::builder()
            .from_address(Address::ZERO)
            .recipient(Address::ZERO)
            .token_address(Address::ZERO)
            .amount(U256::from(1000))
            .build();

        assert_eq!(params.from_address(), Address::ZERO);
        assert_eq!(params.recipient(), Address::ZERO);
        assert_eq!(params.token_address(), Address::ZERO);
        assert_eq!(params.amount(), U256::from(1000));
    }
}
