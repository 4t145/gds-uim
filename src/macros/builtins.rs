/// Construct a GPUI div. A flat body contains attributes; an explicit
/// `{ attributes } [ children ]` body uses the same syntax as `el!`.
///
/// ```rust
/// use gpui::{prelude::*, px};
/// use uim::div;
///
/// let element = div! {
///     { flex; flex_col; gap: 6.0.px; }
///     ["Title", $div! { child: "Content"; }]
/// };
/// ```
#[macro_export]
macro_rules! div {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@unit [::gpui::div] $($tokens)*)
    };
}

/// Construct a GPUI SVG: `svg! { path: "icon.svg"; size: 16.0.px; }`.
#[macro_export]
macro_rules! svg {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@unit [::gpui::svg] $($tokens)*)
    };
}

/// Construct a GPUI anchored element: `anchored! { [#gpui::div] }`.
#[macro_export]
macro_rules! anchored {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@unit [::gpui::anchored] $($tokens)*)
    };
}

/// Construct a GPUI image: `img!(source)` or
/// `img! { (source) { size: 32.0.px; } }`.
#[macro_export]
macro_rules! img {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::img] $($tokens)*)
    };
}

/// Construct a GPUI canvas. Rust callbacks use `$`, as in `el!`.
#[macro_export]
macro_rules! canvas {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::canvas] $($tokens)*)
    };
}

/// Construct a GPUI deferred element: `deferred!(#gpui::div)`.
#[macro_export]
macro_rules! deferred {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::deferred] $($tokens)*)
    };
}

/// Construct a GPUI image-cache element: `image_cache! { (cache) [children] }`.
#[macro_export]
macro_rules! image_cache {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::image_cache] $($tokens)*)
    };
}

/// Construct a GPUI list: `list!(state, $render_item)`.
#[macro_export]
macro_rules! list {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::list] $($tokens)*)
    };
}

/// Construct a GPUI uniform list: `uniform_list!(id, count, $render_items)`.
#[macro_export]
macro_rules! uniform_list {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::uniform_list] $($tokens)*)
    };
}

/// Construct a GPUI surface: `surface!(source)`. Sources depend on the platform.
#[macro_export]
macro_rules! surface {
    ($($tokens:tt)*) => {
        $crate::__uim_builtin!(@args [::gpui::surface] $($tokens)*)
    };
}

// Built-in shortcuts use absolute GPUI paths so callers only need to import
// macros and the builder traits, without importing the factory functions.
#[doc(hidden)]
#[macro_export]
macro_rules! __uim_builtin {
    (@unit [$($function:tt)+] { $($attrs:tt)* } $($rest:tt)*) => {
        $crate::el! { #$($function)+ { $($attrs)* } $($rest)* }
    };
    (@unit [$($function:tt)+] [ $($children:tt)* ] $($rest:tt)*) => {
        $crate::el! { #$($function)+ [ $($children)* ] $($rest)* }
    };
    (@unit [$($function:tt)+] . $($rest:tt)*) => {
        $crate::el! { #$($function)+ . $($rest)* }
    };
    (@unit [$($function:tt)+] $($attrs:tt)*) => {
        $crate::el! { #$($function)+ { $($attrs)* } }
    };
    (@args [$($function:tt)+] :: $($rest:tt)*) => {
        $crate::el! { #$($function)+ :: $($rest)* }
    };
    (@args [$($function:tt)+] ($($args:tt)*) $($rest:tt)*) => {
        $crate::el! { #$($function)+ ($($args)*) $($rest)* }
    };
    (@args [$($function:tt)+] $($args:tt)*) => {
        $crate::el! { #$($function)+ ($($args)*) }
    };
}
