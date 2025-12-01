// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! Core CCTP bridge implementation
//!
//! This module provides the primary types and functionality for bridging USDC across
//! chains using Circle's Cross-Chain Transfer Protocol (CCTP).

mod bridge_trait;
mod cctp;
mod config;
mod v2;

pub use bridge_trait::CctpBridge;
pub use cctp::Cctp;
pub use config::PollingConfig;
pub use v2::{CctpV2, MintResult};
