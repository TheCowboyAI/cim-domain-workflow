//! Event messaging and correlation system
//! 
//! This module implements CIM-compliant event publishing with correlation/causation
//! tracking for cross-domain workflow coordination.

pub mod correlation;
pub mod publishers;

pub use correlation::*;
pub use publishers::*;