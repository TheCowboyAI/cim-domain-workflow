//! Workflow Event Algebra Implementation
//! 
//! This module implements the mathematical foundation for workflow event processing,
//! providing type-safe algebraic operations over workflow events based on the
//! 7-tuple Workflow Event Algebra: 𝒲 = (𝔼, 𝔾, 𝒯, ℂ, ⊕, ⊗, →)

pub mod event_algebra;
pub mod subject_algebra;
pub mod operations;

pub use event_algebra::*;
pub use subject_algebra::*;
pub use operations::*;