//! CCTP contract bindings
//!
//! This module contains Alloy-generated contract bindings for interacting with
//! Circle's Cross-Chain Transfer Protocol smart contracts.
//!
//! - v1: Original CCTP contracts (7 chains)
//! - v2: Enhanced contracts with Fast Transfer, hooks, and 26+ chains

pub mod message_transmitter;
pub mod token_messenger;
pub mod v2;
