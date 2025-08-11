//! Composition and extensibility framework
//! 
//! This module enables domain-specific workflow extensions through composition
//! rather than inheritance, following Category Theory principles.

pub mod extensions;
pub mod templates;
pub mod template_library;

pub use extensions::*;
pub use templates::*;
pub use template_library::*;