//! # Workflow Domain
//!
//! This crate implements the Workflow domain following Domain-Driven Design (DDD) principles
//! with event sourcing and CQRS patterns.
//!
//! ## Architecture
//!
//! The domain is organized into the following modules:
//! - `aggregate` - Domain aggregates (Workflow)
//! - `value_objects` - Value objects (WorkflowId, StepId, etc.)
//! - `commands` - Command objects for CQRS
//! - `events` - Domain events for event sourcing
//! - `handlers` - Command and query handlers
//! - `queries` - Query objects
//! - `projections` - Read model projections
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_workflow::{Workflow, WorkflowStatus, StepType};
//! use std::collections::HashMap;
//!
//! // Create a new workflow
//! let (mut workflow, events) = Workflow::new(
//!     "Order Processing".to_string(),
//!     "Process customer orders".to_string(),
//!     HashMap::new(),
//!     Some("admin".to_string()),
//! ).unwrap();
//!
//! // Add steps to the workflow
//! let events = workflow.add_step(
//!     "Validate Order".to_string(),
//!     "Validate order details".to_string(),
//!     StepType::Manual,
//!     HashMap::new(),
//!     Vec::new(),
//!     Some(30),
//!     Some("validator".to_string()),
//!     Some("admin".to_string()),
//! ).unwrap();
//! ```

// Re-export the main types from the old structure for compatibility
pub use aggregate::*;
pub use commands::*;
pub use domain_events::*;
pub use events::*;
pub use value_objects::*;

// Main exports - use these types from their respective modules

// Domain modules following DDD structure
pub mod aggregate;
pub mod commands;
pub mod domain_events;
pub mod events;
pub mod handlers;
pub mod projections;
pub mod queries;
pub mod state_machine;
pub mod value_objects;

// New consolidated architecture modules
pub mod core;
pub mod primitives;
pub mod composition;
pub mod messaging;
pub mod algebra;

// Backward compatibility layer
pub mod compatibility;

// Legacy workflow engine removed - replaced with proper DDD structure 