// SPDX-FileCopyrightText: 2025 Semiotic AI, Inc.
//
// SPDX-License-Identifier: Apache-2.0
//! CCTP v2 message format types
//!
//! Circle's CCTP v2 introduces a structured message format with headers and
//! typed body formats for different message types (burn messages, etc.).
//!
//! Reference: <https://developers.circle.com/cctp/technical-guide>

use alloy_primitives::{Address, Bytes, FixedBytes, U256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::DomainId;
use crate::FinalityThreshold;

fn push_address_word(bytes: &mut Vec<u8>, address: Address) {
    bytes.extend_from_slice(&[0u8; 12]);
    bytes.extend_from_slice(address.as_slice());
}

fn decode_address_word(bytes: &[u8]) -> Option<Address> {
    (bytes.len() == 32).then(|| Address::from_slice(&bytes[12..32]))
}

fn bytes_is_empty(bytes: &Bytes) -> bool {
    bytes.is_empty()
}

/// Error returned when parsing a canonical CCTP v2 message fails.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("invalid CCTP v2 message: {reason}")]
pub struct ParseMessageError {
    reason: String,
}

impl ParseMessageError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// CCTP v2 Message Header
///
/// The message header contains metadata about cross-chain messages,
/// including source/destination domains, finality requirements, and routing.
///
/// # Format
///
/// - version: uint32 (4 bytes)
/// - sourceDomain: uint32 (4 bytes)
/// - destinationDomain: uint32 (4 bytes)
/// - nonce: bytes32 (32 bytes) - unique identifier assigned by Circle
/// - sender: bytes32 (32 bytes) - message sender address
/// - recipient: bytes32 (32 bytes) - message recipient address
/// - destinationCaller: bytes32 (32 bytes) - authorized caller on destination
/// - minFinalityThreshold: uint32 (4 bytes) - minimum required finality
/// - finalityThresholdExecuted: uint32 (4 bytes) - actual finality level
///
/// Total fixed size: 4 + 4 + 4 + 32 + 32 + 32 + 32 + 4 + 4 = 148 bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Message format version
    pub version: u32,
    /// Source blockchain domain ID
    pub source_domain: DomainId,
    /// Destination blockchain domain ID
    pub destination_domain: DomainId,
    /// Unique message nonce assigned by Circle
    pub nonce: FixedBytes<32>,
    /// Address that sent the message (padded to 32 bytes)
    pub sender: FixedBytes<32>,
    /// Address that will receive the message (padded to 32 bytes)
    pub recipient: FixedBytes<32>,
    /// Address authorized to call receiveMessage on destination (0 = anyone)
    pub destination_caller: FixedBytes<32>,
    /// Minimum finality threshold required (1000 = Fast, 2000 = Standard)
    pub min_finality_threshold: u32,
    /// Actual finality threshold when message was attested
    pub finality_threshold_executed: u32,
}

impl MessageHeader {
    /// Size of the message header in bytes
    pub const SIZE: usize = 148;

    /// Creates a new message header
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        version: u32,
        source_domain: DomainId,
        destination_domain: DomainId,
        nonce: FixedBytes<32>,
        sender: FixedBytes<32>,
        recipient: FixedBytes<32>,
        destination_caller: FixedBytes<32>,
        min_finality_threshold: u32,
        finality_threshold_executed: u32,
    ) -> Self {
        Self {
            version,
            source_domain,
            destination_domain,
            nonce,
            sender,
            recipient,
            destination_caller,
            min_finality_threshold,
            finality_threshold_executed,
        }
    }

    /// Encodes the message header to bytes
    ///
    /// The encoding follows Circle's v2 message format specification.
    pub fn encode(&self) -> Bytes {
        let mut bytes = Vec::with_capacity(Self::SIZE);

        // version (4 bytes)
        bytes.extend_from_slice(&self.version.to_be_bytes());
        // sourceDomain (4 bytes)
        bytes.extend_from_slice(&self.source_domain.as_u32().to_be_bytes());
        // destinationDomain (4 bytes)
        bytes.extend_from_slice(&self.destination_domain.as_u32().to_be_bytes());
        // nonce (32 bytes)
        bytes.extend_from_slice(self.nonce.as_slice());
        // sender (32 bytes)
        bytes.extend_from_slice(self.sender.as_slice());
        // recipient (32 bytes)
        bytes.extend_from_slice(self.recipient.as_slice());
        // destinationCaller (32 bytes)
        bytes.extend_from_slice(self.destination_caller.as_slice());
        // minFinalityThreshold (4 bytes)
        bytes.extend_from_slice(&self.min_finality_threshold.to_be_bytes());
        // finalityThresholdExecuted (4 bytes)
        bytes.extend_from_slice(&self.finality_threshold_executed.to_be_bytes());

        Bytes::from(bytes)
    }

    /// Decodes a message header from bytes
    ///
    /// Returns `None` if the bytes are not at least [`MessageHeader::SIZE`] bytes long
    /// or if domain IDs are invalid.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }

        let version = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let source_domain = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let source_domain = DomainId::from_u32(source_domain)?;

        let destination_domain = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let destination_domain = DomainId::from_u32(destination_domain)?;

        let nonce = FixedBytes::from_slice(&bytes[12..44]);
        let sender = FixedBytes::from_slice(&bytes[44..76]);
        let recipient = FixedBytes::from_slice(&bytes[76..108]);
        let destination_caller = FixedBytes::from_slice(&bytes[108..140]);

        let min_finality_threshold =
            u32::from_be_bytes([bytes[140], bytes[141], bytes[142], bytes[143]]);
        let finality_threshold_executed =
            u32::from_be_bytes([bytes[144], bytes[145], bytes[146], bytes[147]]);

        Some(Self {
            version,
            source_domain,
            destination_domain,
            nonce,
            sender,
            recipient,
            destination_caller,
            min_finality_threshold,
            finality_threshold_executed,
        })
    }

    /// Parses a message header and returns a descriptive error on failure.
    pub fn parse(bytes: &[u8]) -> std::result::Result<Self, ParseMessageError> {
        if bytes.len() < Self::SIZE {
            return Err(ParseMessageError::new(format!(
                "header requires at least {} bytes, got {}",
                Self::SIZE,
                bytes.len()
            )));
        }

        Self::decode(bytes).ok_or_else(|| ParseMessageError::new("failed to decode header"))
    }

    /// Returns true when the nonce is still the placeholder zero value from the on-chain event.
    pub fn has_placeholder_nonce(&self) -> bool {
        self.nonce.as_slice().iter().all(|byte| *byte == 0)
    }

    /// Returns the EVM sender address encoded in the 32-byte sender field.
    ///
    /// This helper assumes the source domain uses the EVM trailing-20-byte
    /// convention for `bytes32` addresses. For non-EVM domains, the raw
    /// [`Self::sender`] field is authoritative.
    #[must_use]
    pub fn sender_address(&self) -> Address {
        Address::from_slice(&self.sender.as_slice()[12..32])
    }

    /// Returns the EVM recipient address encoded in the 32-byte recipient field.
    ///
    /// This helper assumes the destination domain uses the EVM trailing-20-byte
    /// convention for `bytes32` addresses. For non-EVM domains, the raw
    /// [`Self::recipient`] field is authoritative.
    #[must_use]
    pub fn recipient_address(&self) -> Address {
        Address::from_slice(&self.recipient.as_slice()[12..32])
    }

    /// Returns the destination caller as an EVM address if the message is not permissionless.
    ///
    /// This helper assumes the destination domain uses the EVM trailing-20-byte
    /// convention for `bytes32` addresses. For non-EVM domains, the raw
    /// [`Self::destination_caller`] field is authoritative.
    #[must_use]
    pub fn destination_caller_address(&self) -> Option<Address> {
        (!self.is_permissionless())
            .then(|| Address::from_slice(&self.destination_caller.as_slice()[12..32]))
    }

    /// Returns true when the message can be relayed by anyone.
    pub fn is_permissionless(&self) -> bool {
        self.destination_caller
            .as_slice()
            .iter()
            .all(|byte| *byte == 0)
    }

    /// Returns the requested finality threshold when it matches a known CCTP mode.
    #[must_use]
    pub fn requested_finality(&self) -> Option<FinalityThreshold> {
        FinalityThreshold::from_u32(self.min_finality_threshold)
    }

    /// Returns the finality threshold that Circle actually used for the attestation.
    #[must_use]
    pub fn attested_finality(&self) -> Option<FinalityThreshold> {
        FinalityThreshold::from_u32(self.finality_threshold_executed)
    }
}

/// CCTP v2 Burn Message Body
///
/// The burn message body contains information about a token burn operation
/// for cross-chain USDC transfers.
///
/// # Format
///
/// - version: uint32 (4 bytes)
/// - burnToken: bytes32 (32 bytes) - address of token being burned
/// - mintRecipient: bytes32 (32 bytes) - address to receive minted tokens
/// - amount: uint256 (32 bytes) - amount being transferred
/// - messageSender: bytes32 (32 bytes) - original sender address
/// - maxFee: uint256 (32 bytes) - maximum fee willing to pay
/// - feeExecuted: uint256 (32 bytes) - actual fee charged
/// - expirationBlock: uint256 (32 bytes) - block number when message expires
/// - hookData: dynamic bytes - arbitrary data for destination chain hooks
///
/// Total fixed size: 4 + 32 + 32 + 32 + 32 + 32 + 32 + 32 = 228 bytes + dynamic hookData
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BurnMessageV2 {
    /// Message body version
    pub version: u32,
    /// Address of the token being burned (USDC contract)
    pub burn_token: Address,
    /// Address that will receive minted tokens on destination chain
    pub mint_recipient: Address,
    /// Amount of tokens being transferred (in wei/smallest unit)
    pub amount: U256,
    /// Address of the original message sender
    pub message_sender: Address,
    /// Maximum fee the sender is willing to pay (for Fast Transfers)
    pub max_fee: U256,
    /// Actual fee that was charged
    pub fee_executed: U256,
    /// Block number after which the message expires (anti-replay protection)
    pub expiration_block: U256,
    /// Optional hook data for programmable transfers
    pub hook_data: Bytes,
}

impl BurnMessageV2 {
    /// Minimum size of the burn message body in bytes (without hookData)
    pub const MIN_SIZE: usize = 228;

    /// Creates a new burn message with standard settings (no fast transfer, no hooks)
    pub fn new(
        burn_token: Address,
        mint_recipient: Address,
        amount: U256,
        message_sender: Address,
    ) -> Self {
        Self {
            version: 1,
            burn_token,
            mint_recipient,
            amount,
            message_sender,
            max_fee: U256::ZERO,
            fee_executed: U256::ZERO,
            expiration_block: U256::ZERO,
            hook_data: Bytes::new(),
        }
    }

    /// Creates a new burn message with fast transfer settings
    pub fn new_with_fast_transfer(
        burn_token: Address,
        mint_recipient: Address,
        amount: U256,
        message_sender: Address,
        max_fee: U256,
    ) -> Self {
        Self {
            version: 1,
            burn_token,
            mint_recipient,
            amount,
            message_sender,
            max_fee,
            fee_executed: U256::ZERO,
            expiration_block: U256::ZERO,
            hook_data: Bytes::new(),
        }
    }

    /// Creates a new burn message with hook data
    pub fn new_with_hooks(
        burn_token: Address,
        mint_recipient: Address,
        amount: U256,
        message_sender: Address,
        hook_data: Bytes,
    ) -> Self {
        Self {
            version: 1,
            burn_token,
            mint_recipient,
            amount,
            message_sender,
            max_fee: U256::ZERO,
            fee_executed: U256::ZERO,
            expiration_block: U256::ZERO,
            hook_data,
        }
    }

    /// Sets the hook data for this message
    pub fn with_hook_data(mut self, hook_data: Bytes) -> Self {
        self.hook_data = hook_data;
        self
    }

    /// Sets the maximum fee for fast transfer
    pub fn with_max_fee(mut self, max_fee: U256) -> Self {
        self.max_fee = max_fee;
        self
    }

    /// Sets the expiration block
    pub fn with_expiration_block(mut self, expiration_block: U256) -> Self {
        self.expiration_block = expiration_block;
        self
    }

    /// Encodes the burn message body to bytes.
    pub fn encode(&self) -> Bytes {
        let mut bytes = Vec::with_capacity(Self::MIN_SIZE + self.hook_data.len());

        bytes.extend_from_slice(&self.version.to_be_bytes());
        push_address_word(&mut bytes, self.burn_token);
        push_address_word(&mut bytes, self.mint_recipient);
        bytes.extend_from_slice(&self.amount.to_be_bytes::<32>());
        push_address_word(&mut bytes, self.message_sender);
        bytes.extend_from_slice(&self.max_fee.to_be_bytes::<32>());
        bytes.extend_from_slice(&self.fee_executed.to_be_bytes::<32>());
        bytes.extend_from_slice(&self.expiration_block.to_be_bytes::<32>());
        bytes.extend_from_slice(&self.hook_data);

        Bytes::from(bytes)
    }

    /// Decodes a burn message body from bytes.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::MIN_SIZE {
            return None;
        }

        Some(Self {
            version: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            burn_token: decode_address_word(&bytes[4..36])?,
            mint_recipient: decode_address_word(&bytes[36..68])?,
            amount: U256::from_be_slice(&bytes[68..100]),
            message_sender: decode_address_word(&bytes[100..132])?,
            max_fee: U256::from_be_slice(&bytes[132..164]),
            fee_executed: U256::from_be_slice(&bytes[164..196]),
            expiration_block: U256::from_be_slice(&bytes[196..228]),
            hook_data: Bytes::copy_from_slice(&bytes[228..]),
        })
    }

    /// Parses a burn message body and returns a descriptive error on failure.
    pub fn parse(bytes: &[u8]) -> std::result::Result<Self, ParseMessageError> {
        if bytes.len() < Self::MIN_SIZE {
            return Err(ParseMessageError::new(format!(
                "burn message body requires at least {} bytes, got {}",
                Self::MIN_SIZE,
                bytes.len()
            )));
        }

        Self::decode(bytes)
            .ok_or_else(|| ParseMessageError::new("failed to decode burn message body"))
    }

    /// Returns true if this message has hook data
    pub fn has_hooks(&self) -> bool {
        !self.hook_data.is_empty()
    }

    /// Returns true if this message is configured for fast transfer (`max_fee` > 0)
    pub fn is_fast_transfer(&self) -> bool {
        self.max_fee > U256::ZERO
    }
}

/// Parsed representation of a canonical CCTP v2 transfer message.
///
/// This combines the fixed-size message header with the burn message body and
/// can be serialized directly for agent or tool responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedV2Message {
    pub header: MessageHeader,
    pub body: BurnMessageV2,
}

impl ParsedV2Message {
    /// Encodes the full CCTP v2 message.
    pub fn encode(&self) -> Bytes {
        let mut bytes = self.header.encode().to_vec();
        bytes.extend_from_slice(&self.body.encode());
        Bytes::from(bytes)
    }

    /// Decodes a full CCTP v2 message.
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let header = MessageHeader::decode(bytes)?;
        let body = BurnMessageV2::decode(&bytes[MessageHeader::SIZE..])?;
        Some(Self { header, body })
    }

    /// Parses a full CCTP v2 message and returns a descriptive error on failure.
    pub fn parse(bytes: &[u8]) -> std::result::Result<Self, ParseMessageError> {
        let header = MessageHeader::parse(bytes)?;
        let body = BurnMessageV2::parse(&bytes[MessageHeader::SIZE..])?;
        Ok(Self { header, body })
    }

    /// Returns the keccak256 message hash used by the destination contract.
    #[must_use]
    pub fn message_hash(&self) -> FixedBytes<32> {
        alloy_primitives::keccak256(self.encode())
    }

    /// Returns a compact summary that is convenient to serialize from tools.
    ///
    /// Address-like fields in the summary use the SDK's current EVM-oriented
    /// interpretation of `bytes32` address words. For non-EVM domains, use the
    /// raw header fields in [`Self::header`] as the canonical source of truth.
    #[must_use]
    pub fn summary(&self) -> ParsedV2MessageSummary {
        let encoded = self.encode();
        let message_hash = alloy_primitives::keccak256(&encoded);
        let message_len_bytes = encoded.len();

        ParsedV2MessageSummary {
            message_hash,
            message_len_bytes,
            source_domain: self.header.source_domain,
            destination_domain: self.header.destination_domain,
            message_version: self.header.version,
            body_version: self.body.version,
            nonce: self.header.nonce,
            has_placeholder_nonce: self.header.has_placeholder_nonce(),
            sender: self.header.sender_address(),
            recipient: self.header.recipient_address(),
            destination_caller: self.header.destination_caller_address(),
            permissionless_relay: self.header.is_permissionless(),
            requested_finality: self.header.requested_finality(),
            attested_finality: self.header.attested_finality(),
            burn_token: self.body.burn_token,
            mint_recipient: self.body.mint_recipient,
            amount: self.body.amount,
            message_sender: self.body.message_sender,
            max_fee: self.body.max_fee,
            fee_executed: self.body.fee_executed,
            expiration_block: self.body.expiration_block,
            hook_data: self.body.hook_data.clone(),
            hook_data_len_bytes: self.body.hook_data.len(),
            has_hooks: self.body.has_hooks(),
            is_fast_transfer: self.body.is_fast_transfer(),
        }
    }
}

/// JSON-friendly summary of a canonical CCTP v2 transfer message.
///
/// `DomainId` values serialize as `snake_case` strings. Future crate releases may
/// add new domain variants, so older versions of the crate may reject summaries
/// containing unknown domain names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedV2MessageSummary {
    pub message_hash: FixedBytes<32>,
    pub message_len_bytes: usize,
    pub source_domain: DomainId,
    pub destination_domain: DomainId,
    pub message_version: u32,
    pub body_version: u32,
    pub nonce: FixedBytes<32>,
    pub has_placeholder_nonce: bool,
    pub sender: Address,
    pub recipient: Address,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_caller: Option<Address>,
    pub permissionless_relay: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_finality: Option<FinalityThreshold>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attested_finality: Option<FinalityThreshold>,
    pub burn_token: Address,
    pub mint_recipient: Address,
    pub amount: U256,
    pub message_sender: Address,
    pub max_fee: U256,
    pub fee_executed: U256,
    pub expiration_block: U256,
    #[serde(default, skip_serializing_if = "bytes_is_empty")]
    pub hook_data: Bytes,
    pub hook_data_len_bytes: usize,
    pub has_hooks: bool,
    pub is_fast_transfer: bool,
}

impl ParsedV2MessageSummary {
    /// Parses and summarizes a canonical CCTP v2 transfer message.
    pub fn parse(bytes: &[u8]) -> std::result::Result<Self, ParseMessageError> {
        ParsedV2Message::parse(bytes).map(|message| message.summary())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, hex};

    #[test]
    fn test_message_header_size() {
        assert_eq!(MessageHeader::SIZE, 148);
    }

    #[test]
    fn test_message_header_encode_decode() {
        let header = MessageHeader::new(
            1,
            DomainId::Ethereum,
            DomainId::Arbitrum,
            FixedBytes::from([1u8; 32]),
            FixedBytes::from([2u8; 32]),
            FixedBytes::from([3u8; 32]),
            FixedBytes::from([0u8; 32]),
            1000,
            1000,
        );

        let encoded = header.encode();
        assert_eq!(encoded.len(), MessageHeader::SIZE);

        let decoded = MessageHeader::decode(&encoded).expect("should decode");
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_message_header_decode_too_short() {
        let short_bytes = vec![0u8; 100];
        assert!(MessageHeader::decode(&short_bytes).is_none());
    }

    #[test]
    fn test_message_header_decode_invalid_domain() {
        let mut bytes = vec![0u8; MessageHeader::SIZE];
        // Set invalid source domain ID (999)
        bytes[4..8].copy_from_slice(&999u32.to_be_bytes());
        assert!(MessageHeader::decode(&bytes).is_none());
    }

    #[test]
    fn test_burn_message_v2_new() {
        let burn_token = address!("A2d2a41577ce14e20a6c2de999A8Ec2BD9fe34aF");
        let mint_recipient = address!("742d35Cc6634C0532925a3b844Bc9e7595f8fA0d");
        let amount = U256::from(1000000u64);
        let sender = address!("1234567890abcdef1234567890abcdef12345678");

        let msg = BurnMessageV2::new(burn_token, mint_recipient, amount, sender);

        assert_eq!(msg.version, 1);
        assert_eq!(msg.burn_token, burn_token);
        assert_eq!(msg.mint_recipient, mint_recipient);
        assert_eq!(msg.amount, amount);
        assert_eq!(msg.message_sender, sender);
        assert_eq!(msg.max_fee, U256::ZERO);
        assert_eq!(msg.fee_executed, U256::ZERO);
        assert_eq!(msg.expiration_block, U256::ZERO);
        assert!(msg.hook_data.is_empty());
        assert!(!msg.has_hooks());
        assert!(!msg.is_fast_transfer());
    }

    #[test]
    fn test_burn_message_v2_fast_transfer() {
        let burn_token = address!("A2d2a41577ce14e20a6c2de999A8Ec2BD9fe34aF");
        let mint_recipient = address!("742d35Cc6634C0532925a3b844Bc9e7595f8fA0d");
        let amount = U256::from(1000000u64);
        let sender = address!("1234567890abcdef1234567890abcdef12345678");
        let max_fee = U256::from(100u64);

        let msg = BurnMessageV2::new_with_fast_transfer(
            burn_token,
            mint_recipient,
            amount,
            sender,
            max_fee,
        );

        assert_eq!(msg.max_fee, max_fee);
        assert!(msg.is_fast_transfer());
        assert!(!msg.has_hooks());
    }

    #[test]
    fn test_burn_message_v2_with_hooks() {
        let burn_token = address!("A2d2a41577ce14e20a6c2de999A8Ec2BD9fe34aF");
        let mint_recipient = address!("742d35Cc6634C0532925a3b844Bc9e7595f8fA0d");
        let amount = U256::from(1000000u64);
        let sender = address!("1234567890abcdef1234567890abcdef12345678");
        let hook_data = Bytes::from(vec![1, 2, 3, 4]);

        let msg = BurnMessageV2::new_with_hooks(
            burn_token,
            mint_recipient,
            amount,
            sender,
            hook_data.clone(),
        );

        assert_eq!(msg.hook_data, hook_data);
        assert!(msg.has_hooks());
        assert!(!msg.is_fast_transfer());
    }

    #[test]
    fn test_burn_message_v2_builder() {
        let burn_token = address!("A2d2a41577ce14e20a6c2de999A8Ec2BD9fe34aF");
        let mint_recipient = address!("742d35Cc6634C0532925a3b844Bc9e7595f8fA0d");
        let amount = U256::from(1000000u64);
        let sender = address!("1234567890abcdef1234567890abcdef12345678");

        let msg = BurnMessageV2::new(burn_token, mint_recipient, amount, sender)
            .with_max_fee(U256::from(100u64))
            .with_hook_data(Bytes::from(vec![1, 2, 3]))
            .with_expiration_block(U256::from(1000u64));

        assert!(msg.is_fast_transfer());
        assert!(msg.has_hooks());
        assert_eq!(msg.expiration_block, U256::from(1000u64));
    }

    #[test]
    fn test_burn_message_v2_encode_decode_roundtrip() {
        let message = BurnMessageV2::new_with_fast_transfer(
            address!("75FaF114EAFb1bdbE2f0316Df893Fd58ce46AA4D"),
            address!("7F7D081724F0240c64C9E01CDe4626602f9a0192"),
            U256::from(1_000_000u64),
            address!("1234567890abcdef1234567890abcdef12345678"),
            U256::from(100u64),
        )
        .with_hook_data(Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]))
        .with_expiration_block(U256::from(12345u64));

        let encoded = message.encode();
        let decoded = BurnMessageV2::decode(&encoded).expect("burn message should decode");

        assert_eq!(decoded, message);
    }

    #[test]
    fn test_message_header_permissionless_helpers() {
        let header = MessageHeader::new(
            1,
            DomainId::Ethereum,
            DomainId::Base,
            FixedBytes::from([0u8; 32]),
            address!("75FaF114EAFb1bdbE2f0316Df893Fd58ce46AA4D").into_word(),
            address!("7F7D081724F0240c64C9E01CDe4626602f9a0192").into_word(),
            FixedBytes::ZERO,
            FinalityThreshold::Fast.as_u32(),
            FinalityThreshold::Standard.as_u32(),
        );

        assert!(header.has_placeholder_nonce());
        assert!(header.is_permissionless());
        assert_eq!(
            header.sender_address(),
            address!("75FaF114EAFb1bdbE2f0316Df893Fd58ce46AA4D")
        );
        assert_eq!(
            header.recipient_address(),
            address!("7F7D081724F0240c64C9E01CDe4626602f9a0192")
        );
        assert_eq!(header.requested_finality(), Some(FinalityThreshold::Fast));
        assert_eq!(
            header.attested_finality(),
            Some(FinalityThreshold::Standard)
        );
        assert_eq!(header.destination_caller_address(), None);
    }

    #[test]
    fn test_parsed_v2_message_from_real_circle_message() {
        let raw_message = hex::decode("0000000100000003000000062f3cb13cf4a6103f9e3b256495b08c4e05630fcba639565d199ed420a5f2be010000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa0000000000000000000000008fe6b999dc680ccfdd5bf7eb0974218be2542daa0000000000000000000000000000000000000000000000000000000000000000000007d0000007d00000000100000000000000000000000075faf114eafb1bdbe2f0316df893fd58ce46aa4d0000000000000000000000007f7d081724f0240c64c9e01cde4626602f9a019200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000007f7d081724f0240c64c9e01cde4626602f9a0192000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

        let parsed = ParsedV2Message::parse(&raw_message).expect("message should parse");
        let summary = parsed.summary();

        assert_eq!(parsed.header.source_domain, DomainId::Arbitrum);
        assert_eq!(parsed.header.destination_domain, DomainId::Base);
        assert!(!parsed.header.has_placeholder_nonce());
        assert_eq!(
            parsed.header.requested_finality(),
            Some(FinalityThreshold::Standard)
        );
        assert_eq!(
            parsed.header.attested_finality(),
            Some(FinalityThreshold::Standard)
        );
        assert_eq!(
            parsed.body.burn_token,
            address!("75FaF114EAFb1bdbE2f0316Df893Fd58ce46AA4D")
        );
        assert_eq!(
            parsed.body.mint_recipient,
            address!("7F7D081724F0240c64C9E01CDe4626602f9a0192")
        );
        assert_eq!(parsed.body.amount, U256::from(1_000_000u64));
        assert_eq!(
            parsed.body.message_sender,
            address!("7F7D081724F0240c64C9E01CDe4626602f9a0192")
        );
        assert_eq!(parsed.body.max_fee, U256::ZERO);
        assert_eq!(parsed.body.fee_executed, U256::ZERO);
        assert_eq!(parsed.body.expiration_block, U256::ZERO);
        assert!(parsed.body.hook_data.is_empty());
        assert_eq!(parsed.encode().as_ref(), raw_message.as_slice());
        assert_eq!(
            parsed.message_hash(),
            alloy_primitives::keccak256(&raw_message)
        );
        assert_eq!(
            summary.message_hash,
            alloy_primitives::keccak256(&raw_message)
        );
        assert!(summary.permissionless_relay);
        assert!(!summary.has_hooks);
        assert!(!summary.is_fast_transfer);
    }

    #[test]
    fn test_parsed_v2_message_summary_omits_empty_optionals() {
        let summary = ParsedV2MessageSummary {
            message_hash: FixedBytes::from([0x11; 32]),
            message_len_bytes: 376,
            source_domain: DomainId::Ethereum,
            destination_domain: DomainId::Base,
            message_version: 1,
            body_version: 1,
            nonce: FixedBytes::from([0x22; 32]),
            has_placeholder_nonce: false,
            sender: address!("75FaF114EAFb1bdbE2f0316Df893Fd58ce46AA4D"),
            recipient: address!("7F7D081724F0240c64C9E01CDe4626602f9a0192"),
            destination_caller: None,
            permissionless_relay: true,
            requested_finality: Some(FinalityThreshold::Standard),
            attested_finality: Some(FinalityThreshold::Standard),
            burn_token: address!("75FaF114EAFb1bdbE2f0316Df893Fd58ce46AA4D"),
            mint_recipient: address!("7F7D081724F0240c64C9E01CDe4626602f9a0192"),
            amount: U256::from(1_000_000u64),
            message_sender: address!("7F7D081724F0240c64C9E01CDe4626602f9a0192"),
            max_fee: U256::ZERO,
            fee_executed: U256::ZERO,
            expiration_block: U256::ZERO,
            hook_data: Bytes::new(),
            hook_data_len_bytes: 0,
            has_hooks: false,
            is_fast_transfer: false,
        };

        let json = serde_json::to_value(summary).expect("summary should serialize");
        assert!(json.get("destination_caller").is_none());
        assert!(json.get("hook_data").is_none());
    }
}
