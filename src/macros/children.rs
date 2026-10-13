// Expand child macros as siblings so their parser depth does not
// accumulate with the preceding children.
#[doc(hidden)]
#[macro_export]
macro_rules! __uim_children {
    (@children [$element:expr]) => { $element };
    (@children [$element:expr] ,) => { $element };
    (@children [$element:expr] $marker:tt $($name:ident)::+ ! $body:tt , $($rest:tt)*) => {
        $crate::__uim_children!(@macro_child [$element] [$marker] [$($name)::+!$body] $($rest)*)
    };
    (@macro_child [$element:expr] [$] [$child:expr] $($rest:tt)*) => {{
        let element = $element;
        let child = $child;
        let element = element.child(child);
        $crate::__uim_children!(@children [element] $($rest)*)
    }};
    (@macro_child [$element:expr] [$marker:tt] [$child:expr] $($rest:tt)*) => {
        compile_error!("el!: prefix Rust macro expressions with $")
    };
    (@children [$element:expr] .. $($rest:tt)*) => {
        $crate::__uim_value!(@value [@spread [$element]] $($rest)*)
    };
    (@children [$element:expr] $($rest:tt)+) => {
        $crate::__uim_element!(@node [@child [$element]] $($rest)+)
    };
    ($($invalid:tt)*) => {
        compile_error!("el!: invalid children syntax")
    };
}
