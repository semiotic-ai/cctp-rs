// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! CCTP contract bindings
//!
//! This module contains Alloy-generated contract bindings for interacting with
//! Circle's Cross-Chain Transfer Protocol smart contracts.
//!
//! - v1: Original CCTP contracts (7 chains)
//! - v2: Enhanced contracts with Fast Transfer, hooks, and 26+ chains
//!
//! ## Public API
//!
//! Contract wrappers provide type-safe, instrumented interfaces to CCTP contracts:
//!
//! - v1: [`TokenMessengerContract`](token_messenger::TokenMessengerContract), [`MessageTransmitterContract`](message_transmitter::MessageTransmitterContract)
//! - v2: [`TokenMessengerV2Contract`](v2::TokenMessengerV2Contract), [`MessageTransmitterV2Contract`](v2::MessageTransmitterV2Contract)
//! - ERC20: [`Erc20Contract`](erc20::Erc20Contract) for approval and allowance operations

pub mod erc20;
pub mod message_transmitter;
pub mod token_messenger;
pub mod v2;
