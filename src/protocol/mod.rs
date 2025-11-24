//! CCTP protocol types and definitions
//!
//! This module contains core protocol-level types used in Circle's Cross-Chain
//! Transfer Protocol (CCTP), including domain identifiers, attestation responses,
//! and v2-specific types like finality thresholds and message formats.

mod attestation;
mod domain_id;
mod finality;
mod message;

pub use attestation::{AttestationBytes, AttestationResponse, AttestationStatus};
pub use domain_id::DomainId;
pub use finality::FinalityThreshold;
pub use message::{BurnMessageV2, MessageHeader};
