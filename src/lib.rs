//! Declarative syntax for GPUI's fluent element builders.
//!
//! Import `el`, `m`, and the traits used by your builders, usually
//! `gpui::prelude::*`. These macros have no GPUI dependency and preserve builder
//! return types, including methods and wrappers that change the element type.

/// Build a value or element tree with postfix function application.
///
/// `.div` calls `div()`, `(a, b).f` calls `f(a, b)`, and `value.f` calls
/// `f(value)`. Bare paths denote existing values. Rust expressions use `$(...)`.
/// Attribute blocks and child lists can alternate or repeat; each suffix receives
/// the complete preceding result. Inputs are evaluated once, in source order. Computed callables are
/// evaluated after their inputs; paths preserve Rust's normal call semantics.
/// Bare `{ attrs }` and `move { attrs }` values create builder closures.
/// Named closures use `item => expr`, `(a, b) => expr`, or `() => expr`.
/// `#? expr` calls `.debug_selector(|| expr)`; `#? move expr` moves captures.
/// `@if (...) ... @else ... @end` and `@match (...) @case (...) ... @end`
/// select values or conditionally apply attribute and children fragments.
///
/// ```rust
/// use gds_uim::el;
/// use gpui::{div, deferred, prelude::*, px};
///
/// let selected = true;
/// let element = el! {
///     .div { # "example";
///         #? "example" { into };
///         flex;
///         gap: 8.0.px;
///         when: selected, { border_1 };
///     } ["Hello", .div ["World"]]
///     .deferred { priority: 1 }
/// };
/// ```
pub use gds_uim_macros::el;

/// Make a builder closure using the same attribute syntax as [`el!`].
///
/// Inside `el!`, `method: { ... }` passes a bare builder closure. Use
/// `method: move { ... }` or `m!(move { ... })` for move captures. Expressions are evaluated
/// when the closure is called. A suffix can use a macro producing a callable.
///
/// ```rust
/// use gds_uim::{el, m};
/// use gpui::{div, prelude::*, px, rgb};
///
/// let padded = div().map(m! { p: 8.0.px; rounded_lg });
/// let element = el! {
///     .div { # "button"; hover: { bg: 0x818cf8.rgb } }
///     ["Button"]
///     .m! { p: 12.0.px }
/// };
/// ```
pub use gds_uim_macros::m;
