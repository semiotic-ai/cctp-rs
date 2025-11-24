//! CCTP v2 message format types
//!
//! Circle's CCTP v2 introduces a structured message format with headers and
//! typed body formats for different message types (burn messages, etc.).
//!
//! Reference: <https://developers.circle.com/cctp/technical-guide>

use alloy_primitives::{Address, Bytes, FixedBytes, U256};

use super::DomainId;

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
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Returns true if this message has hook data
    pub fn has_hooks(&self) -> bool {
        !self.hook_data.is_empty()
    }

    /// Returns true if this message is configured for fast transfer (max_fee > 0)
    pub fn is_fast_transfer(&self) -> bool {
        self.max_fee > U256::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

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
}
