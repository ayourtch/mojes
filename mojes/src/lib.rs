// #![doc = include_str!("../README.md")]

//! # Mojes - Rust to JavaScript Transpiler
//!
//! A complete solution for transpiling Rust code to JavaScript with full DOM API support.

// Re-export all the derive macros
pub use mojes_derive::*;

// Re-export core transpilation functionality
pub use mojes_mojo::*;

// Re-export DOM API under a clean namespace
pub use mojes_dom_api as dom;

// Re-export linkme for distributed slices
pub use linkme::{DistributedSlice, distributed_slice};

/// Convenient prelude that imports commonly used items
pub mod prelude {
    pub use crate::dom::{alert, confirm, console, document, prompt, window};
    pub use crate::{distributed_slice, js_type, to_js};
}

/// Type aliases for common patterns
pub type JsSlice = &'static [&'static str];

/// Helper macro for declaring the JS distributed slice
#[macro_export]
macro_rules! js_slice {
    () => {
        #[mojes::distributed_slice]
        pub static JS: [&str];
    };
}

// Optional: Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
