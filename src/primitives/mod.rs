//! Core primitives for the unified workflow domain
//! 
//! This module provides the foundational types and abstractions used across
//! all domain extensions, ensuring consistency and interoperability.

pub mod identifiers;
pub mod context;

pub use identifiers::*;
pub use context::*;