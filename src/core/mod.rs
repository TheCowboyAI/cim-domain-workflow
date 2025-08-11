//! Core workflow engine components
//! 
//! This module contains the foundational components for the unified workflow domain,
//! implementing the abstract interfaces that all domain extensions will use.

pub mod engine;
pub mod state_machine;
pub mod template_engine;

pub use engine::*;
pub use state_machine::*;
pub use template_engine::*;