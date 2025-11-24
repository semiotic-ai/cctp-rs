//! CCTP v2 contract bindings
//!
//! This module contains contract bindings for Circle's CCTP v2 contracts,
//! which add Fast Transfer, programmable hooks, and support for 26+ chains.

mod message_transmitter_v2;
mod token_messenger_v2;

// These will be used in upcoming v2 bridge implementation
#[allow(unused_imports)]
pub use message_transmitter_v2::MessageTransmitterV2Contract;
#[allow(unused_imports)]
pub use token_messenger_v2::TokenMessengerV2Contract;
