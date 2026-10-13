/// Map a builder through the same attributes as `el!`.
///
/// `m! { bg: 0x818cf8.rgb }` expands to a closure equivalent to
/// `|value| value.bg(rgb(0x818cf8))`. The receiving API infers its parameter
/// type, so it works with style refinements, elements, and custom builders.
/// Import the traits and value conversion functions used by the attributes.
///
/// Use it directly with `hover`, `active`, `focus`, `when`, or `map`. Inside
/// attribute blocks, `method => { ... }` is shorthand for `method: $m! { ... }`.
/// Likewise, `@condition => { ... }` applies a mapping only when the condition is true.
/// You can also pass `$m! { ... }` explicitly, including to multi-argument methods.
/// Attributes support conditions, multiple arguments, DSL suffixes, and Rust
/// expressions. An empty body returns the input unchanged.
///
/// ```rust
/// use gpui::{div, prelude::*, px, rgb};
/// use gds_uim::{el, m};
///
/// let highlighted = true;
/// let element = el! {
///     #div {
///         id: "button";
///         hover => { bg: 0x818cf8.rgb; text_color: 0xffffff.rgb };
///         active => { bg: 0x6366f1.rgb };
///         @highlighted => { border_1; border_color: 0x818cf8.rgb };
///     }
/// };
/// let padded = div().map(m! { p: 8.0.px; rounded_lg });
/// ```
///
/// As an element suffix, `.m! { ... }` receives the complete preceding tree.
/// It can change the builder type and be followed by attributes or more suffixes.
/// Import `m`, use an alias, or write the qualified `.gds_uim::m! { ... }` form.
///
/// ```rust
/// use gpui::{deferred, div, prelude::*, rgb};
/// use gds_uim::{el, m};
///
/// let element = el! {
///     #div { id: "reset"; } ["Reset"]
///         .m! { rounded_lg; hover => { bg: 0x475569.rgb } }
///         .deferred
///         .m! { priority: 1 }
/// };
/// ```
///
/// Attributes are evaluated when the closure is called. Captures follow Rust's
/// normal closure rules; use `m!(move { ... })` or `method => move { ... }`
/// to move captures into the closure.
///
/// ```rust
/// use gpui::{Div, SharedString, div, prelude::*};
/// use gds_uim::m;
///
/// fn with_font(font: SharedString) -> impl FnOnce(Div) -> Div + 'static {
///     m!(move { font_family: font })
/// }
/// let element = div().map(with_font("sans-serif".into()));
/// ```
#[macro_export]
macro_rules! m {
    (move { $($attrs:tt)* }) => {
        move |value| $crate::__uim_attributes!(@attributes [value] [] $($attrs)* ;)
    };
    ($($attrs:tt)*) => {
        |value| $crate::__uim_attributes!(@attributes [value] [] $($attrs)* ;)
    };
}
