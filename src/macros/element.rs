/// Build an element using constructors, attributes, children, and unary wrappers.
///
/// - `List` and `List(args)` call `List::new(...)`.
/// - `Button with_label("confirm")` calls `Button::with_label("confirm")`.
/// - `#div` and `#div()` call `div()`; `#img(source)` calls `img(source)`.
/// - Separate attributes with `;` and an attribute's arguments with `,`.
/// - `flex;` calls `.flex()`; `gap: 6.0.px;` calls `.gap(px(6.0))`.
/// - `method: a, b;` and `method(a, b);` call `.method(a, b)`.
/// - `@condition: builder;` calls `.when(condition, builder)`.
/// - `[child, ..items]` calls `.child(child).children(items)`.
/// - `#div [children].deferred` calls `deferred(div().children(...))`.
///
/// Bare values are DSL literals, variable names, or module paths. `a.b`
/// applies `b(a)`; successive suffixes run from left to right. Function paths
/// support modules and generics. Wrap Rust expressions in `$`: `$theme.surface`
/// reads a field, `$move |event, window, cx| { ... }` is a Rust closure, and
/// `$(theme.spacing).px` applies `px` to a Rust expression's result.
///
/// Element bodies are optional. Attributes run before children, and a suffix
/// wraps the complete preceding tree. Attributes following a suffix configure
/// that wrapper. Bare child names denote constructors; use `$child` for an
/// existing element. Constructor arguments use the same DSL values as attributes.
///
/// `$expression` extends to the next argument/child delimiter. Use `$(...)`
/// to establish a boundary before DSL suffixes or element bodies. Rust blocks,
/// tuples, calculations, indexing, calls, and macros also need `$`.
///
/// ```rust
/// use gpui::{deferred, div, prelude::*, px};
/// use uim::el;
///
/// let show_details = true;
/// let rest = vec![div(), div()];
/// let element = el! {
///     #div {
///         flex;
///         flex_col;
///         gap: 8.0.px;
///         @show_details: $|element| element.child("Details");
///     } [
///         #div { flex; } ["Confirm"],
///         ..rest,
///     ].deferred { priority: 1; }
/// };
/// ```
#[macro_export]
macro_rules! el {
    () => {
        compile_error!("el!: expected an element constructor, #factory, or $Rust expression")
    };
    ($($tokens:tt)+) => {
        $crate::__uim_element!(@node [@root] $($tokens)+ ,)
    };
}

// Parse constructors, element bodies, paths, and unary suffixes.
#[doc(hidden)]
#[macro_export]
macro_rules! __uim_element {
    (@node [$($context:tt)*] # $($path:ident)::+ ($($args:tt)*) $($rest:tt)*) => {
        $crate::__uim_element!(@construct [$($context)*] [$($path)::+] [$($args)*] [$($rest)*])
    };
    (@node [$($context:tt)*] # $($path:ident)::+ { $($attrs:tt)* } $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$($path)::+()] { $($attrs)* } $($rest)*)
    };
    (@node [$($context:tt)*] # $($path:ident)::+ [ $($children:tt)* ] $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$($path)::+()] [ $($children)* ] $($rest)*)
    };
    (@node [$($context:tt)*] # $($path:ident)::+ . $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$($path)::+()] . $($rest)*)
    };
    (@node [$($context:tt)*] # $($path:ident)::+ , $($rest:tt)*) => {
        $crate::__uim_emit!(@emit [$($context)*] [$($path)::+()] , $($rest)*)
    };
    (@node [$($context:tt)*] # $($rest:tt)*) => {
        $crate::__uim_element!(@scan [@factory [$($context)*]] [] [] $($rest)*)
    };
    (@node [$($context:tt)*] $literal:literal $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$literal] $($rest)*)
    };
    (@node [$($context:tt)*] ($($group:tt)*) $($rest:tt)*) => {
        $crate::__uim_element!(@node [@node_group [$($context)*] [$($rest)*]] $($group)* ,)
    };
    (@node [$($context:tt)*] $($path:ident)::+ $constructor:ident
        ($($args:tt)*) $($rest:tt)*) => {
        $crate::__uim_element!(@construct [$($context)*] [$($path)::+::$constructor]
            [$($args)*] [$($rest)*])
    };
    (@node [$($context:tt)*] $($path:ident)::+ ($($args:tt)*) $($rest:tt)*) => {
        $crate::__uim_element!(@construct [$($context)*] [$($path)::+::new]
            [$($args)*] [$($rest)*])
    };
    (@node [$($context:tt)*] $first:ident $($rest:tt)*) => {
        $crate::__uim_element!(@node_path [$($context)*] [$first] $($rest)*)
    };
    (@node_path [$($context:tt)*] [$($path:ident)::+] :: $next:ident $($rest:tt)*) => {
        $crate::__uim_element!(@node_path [$($context)*] [$($path)::+::$next] $($rest)*)
    };
    (@node_path [$($context:tt)*] [$($path:ident)::+] $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$($path)::+::new()] $($rest)*)
    };
    (@node [$($context:tt)*] $marker:tt $($rest:tt)*) => {
        $crate::__uim_element!(@node_prefix [$($context)*] [$marker] $($rest)*)
    };
    (@node_prefix [$($context:tt)*] [$] ($expression:expr) $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$expression] $($rest)*)
    };
    (@node_prefix [$($context:tt)*] [$] $($rest:tt)*) => {
        $crate::__uim_element!(@scan [@rust_node [$($context)*]] [] [] $($rest)*)
    };
    (@node_prefix [$($context:tt)*] [$marker:tt] $($rest:tt)*) => {
        compile_error!("el!: expected a constructor, #factory, literal, or $Rust expression")
    };
    (@construct [$($context:tt)*] [$function:path] [] [$($rest:tt)*]) => {
        $crate::__uim_element!(@stage [$($context)*] [$function()] $($rest)*)
    };
    (@construct [$($context:tt)*] [$function:path] [$($args:tt)+] [$($rest:tt)*]) => {
        $crate::__uim_element!(@stage [$($context)*]
            [$crate::__uim_element!(@constructor_value [$function] [$($args)+])] $($rest)*)
    };
    (@constructor_value [$function:path] [$($args:tt)*]) => {
        $crate::__uim_value!(@arguments [@constructor [$function]] [] $($args)* ;)
    };
    (@stage [$($context:tt)*] [$element:expr]
        { $($attrs:tt)* } [ $($children:tt)* ] $($rest:tt)*) => {
        $crate::__uim_element!(@suffix [$($context)*]
            [$crate::__uim_attributes!(@attributes [$element] [$($children)*] $($attrs)* ;)]
            $($rest)*)
    };
    (@stage [$($context:tt)*] [$element:expr] { $($attrs:tt)* } $($rest:tt)*) => {
        $crate::__uim_element!(@suffix [$($context)*]
            [$crate::__uim_attributes!(@attributes [$element] [] $($attrs)* ;)] $($rest)*)
    };
    (@stage [$($context:tt)*] [$element:expr] [ $($children:tt)* ] $($rest:tt)*) => {
        $crate::__uim_element!(@suffix [$($context)*]
            [$crate::__uim_children!(@children [$element] $($children)* ,)] $($rest)*)
    };
    (@stage [$($context:tt)*] [$element:expr] $($rest:tt)*) => {
        $crate::__uim_element!(@suffix [$($context)*] [$element] $($rest)*)
    };
    (@suffix [$($context:tt)*] [$element:expr] . $($rest:tt)*) => {
        $crate::__uim_element!(@scan [@unary [$($context)*] [$element]] [] [] $($rest)*)
    };
    (@suffix [$($context:tt)*] [$element:expr] $($rest:tt)*) => {
        $crate::__uim_emit!(@emit [$($context)*] [$element] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [@ $($depth:tt)*] < $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* <] [@ @ $($depth)*] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [@] > $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* >] [] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [@ @ $($depth:tt)*] > $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* >] [@ $($depth)*] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [@ @] >> $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* >>] [] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [@ @ @ $($depth:tt)*] >> $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* >>] [@ $($depth)*] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [@ $($depth:tt)*] $next:tt $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* $next] [@ $($depth)*] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [] :: < $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* :: <] [@] $($rest)*)
    };
    (@scan [$($context:tt)*] [] [] < $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [<] [@] $($rest)*)
    };
    (@scan [@rust_node [$($context:tt)*]] [$($tokens:tt)+] []
        ! $body:tt $($rest:tt)*) => {
        $crate::__uim_element!(@scan [@rust_node [$($context)*]]
            [$($tokens)+ ! $body] [] $($rest)*)
    };
    (@scan [@factory [$($context:tt)*]] [$($function:tt)+] []
        ($($args:tt)*) $($rest:tt)*) => {
        $crate::__uim_element!(@factory_args [$($context)*] [$($function)+]
            [$($args)*] [$($rest)*])
    };
    (@scan [@unary [$($context:tt)*] [$element:expr]] [$($function:tt)+] []
        ($($args:tt)*) $($rest:tt)*) => {
        compile_error!("el!: suffix functions take only the preceding value; write .function without an argument list")
    };
    (@scan [@value [$($context:tt)*]] [$($value:tt)+] []
        ($($args:tt)*) $($rest:tt)*) => {
        compile_error!("el!: prefix Rust calls with $ or use #factory(args) for a DSL call")
    };
    (@scan [@factory [$($context:tt)*]] [$($function:tt)*] [] . $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [@factory [$($context)*]] [$($function)*] . $($rest)*)
    };
    (@scan [@unary [$($context:tt)*] [$element:expr]] [$($function:tt)*] []
        . $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [@unary [$($context)*] [$element]]
            [$($function)*] . $($rest)*)
    };
    (@scan [@value [$($context:tt)*]] [$($value:tt)*] [] . $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [@value [$($context)*]] [$($value)*] . $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [] { $($attrs:tt)* } $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [$($context)*] [$($tokens)*] { $($attrs)* } $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [] [ $($children:tt)* ] $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [$($context)*] [$($tokens)*] [ $($children)* ] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [] , $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [$($context)*] [$($tokens)*] , $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [] ; $($rest:tt)*) => {
        $crate::__uim_element!(@scan_end [$($context)*] [$($tokens)*] ; $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [] $next:tt $($rest:tt)*) => {
        $crate::__uim_element!(@scan [$($context)*] [$($tokens)* $next] [] $($rest)*)
    };
    (@scan [$($context:tt)*] [$($tokens:tt)*] [$($depth:tt)*]) => {
        compile_error!("el!: incomplete value or function path")
    };
    (@factory_args [$($context:tt)*] [$function:path] [$($args:tt)*] [$($rest:tt)*]) => {
        $crate::__uim_element!(@construct [$($context)*] [$function] [$($args)*] [$($rest)*])
    };
    (@factory_args [$($context:tt)*] [$($invalid:tt)*] [$($args:tt)*] [$($rest:tt)*]) => {
        compile_error!("el!: expected a function path after #")
    };
    (@scan_end [@factory [$($context:tt)*]] [$function:path] $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$function()] $($rest)*)
    };
    (@scan_end [@unary [$($context:tt)*] [$element:expr]]
        [$function:path] $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$function($element)] $($rest)*)
    };
    (@scan_end [@value [$($context:tt)*]] [$value:path] $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$value] $($rest)*)
    };
    (@scan_end [@rust_node [$($context:tt)*]] [$value:expr] $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$value] $($rest)*)
    };
    (@scan_end [@unary [$($context:tt)*] [$element:expr]]
        [$($invalid:tt)*] $($rest:tt)*) => {
        compile_error!("el!: expected a unary function path after .; suffix functions receive the preceding value")
    };
    (@scan_end [$($context:tt)*] [$($invalid:tt)*] $($rest:tt)*) => {
        compile_error!("el!: expected a DSL literal or path; prefix Rust expressions with $")
    };
    ($($invalid:tt)*) => {
        compile_error!("el!: invalid element syntax")
    };
}
