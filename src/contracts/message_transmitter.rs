//! MessageTransmitter contract bindings
//!
//! This module contains the Alloy-generated contract bindings for the CCTP
//! MessageTransmitter contract, which handles cross-chain message verification
//! and processing.

use alloy_sol_types::sol;

sol!(
    #[allow(clippy::too_many_arguments)]
    #[allow(missing_docs)]
    #[sol(rpc)]
    MessageTransmitter,
    "abis/v1_message_transmitter.json"
);
