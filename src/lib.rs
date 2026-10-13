//! Declarative syntax for GPUI's fluent element builders.
//!
//! Import `el` and the traits used by your builders, usually
//! `gpui::prelude::*`. The macro has no GPUI dependency and preserves builder
//! return types, including methods and wrappers that change the element type.

mod macros;
