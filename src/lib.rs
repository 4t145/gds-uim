//! Declarative syntax for GPUI's fluent element builders.
//!
//! Import `el`, `m`, and the traits used by your builders, usually
//! `gpui::prelude::*`. These macros have no GPUI dependency and preserve builder
//! return types, including methods and wrappers that change the element type.

mod macros;

// Supply the input type before checking a macro-generated closure's body.
// A direct `(m! { ... })(value)` call cannot infer method receivers reliably.
#[doc(hidden)]
#[inline]
pub fn __uim_apply<T, U>(value: T, mapper: impl FnOnce(T) -> U) -> U {
    mapper(value)
}
