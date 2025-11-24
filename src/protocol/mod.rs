//! CCTP protocol types and definitions
//!
//! This module contains core protocol-level types used in Circle's Cross-Chain
//! Transfer Protocol (CCTP), including domain identifiers and attestation responses.

mod attestation;
mod domain_id;

pub use attestation::{AttestationBytes, AttestationResponse, AttestationStatus};
pub use domain_id::DomainId;
