// Parse DSL values and isolated Rust expressions, then assemble arguments.
#[doc(hidden)]
#[macro_export]
macro_rules! __uim_value {
    (@value [$($context:tt)*] # $($rest:tt)*) => {
        $crate::__uim_element!(@node [$($context)*] # $($rest)*)
    };
    (@value [$($context:tt)*] $literal:literal $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$literal] $($rest)*)
    };
    (@value [$($context:tt)*] ($($group:tt)*) $($rest:tt)*) => {
        $crate::__uim_value!(@value [@value_group [$($context)*] [$($rest)*]] $($group)* ;)
    };
    (@value [$($context:tt)*] $first:ident $($rest:tt)*) => {
        $crate::__uim_element!(@scan [@value [$($context)*]] [$first] [] $($rest)*)
    };
    (@value [$($context:tt)*] $marker:tt $($rest:tt)*) => {
        $crate::__uim_value!(@value_prefix [$($context)*] [$marker] $($rest)*)
    };
    (@value_prefix [$($context:tt)*] [$] ($expression:expr) $($rest:tt)*) => {
        $crate::__uim_element!(@stage [$($context)*] [$expression] $($rest)*)
    };
    (@value_prefix [$($context:tt)*] [$] $expression:expr, $($rest:tt)*) => {
        $crate::__uim_emit!(@emit [$($context)*] [$expression] , $($rest)*)
    };
    (@value_prefix [$($context:tt)*] [$] $expression:expr; $($rest:tt)*) => {
        $crate::__uim_emit!(@emit [$($context)*] [$expression] ; $($rest)*)
    };
    (@value_prefix [$($context:tt)*] [$marker:tt] $($rest:tt)*) => {
        compile_error!("el!: expected a DSL value or $Rust expression")
    };
    (@arguments [$($call:tt)*] [$($args:expr),*] ; $($rest:tt)*) => {
        $crate::__uim_value!(@call [$($call)*] [$($args),*] $($rest)*)
    };
    (@arguments [$($call:tt)*] [$($args:expr),*] $($rest:tt)+) => {
        $crate::__uim_value!(@value [@argument [$($call)*] [$($args),*]] $($rest)+)
    };
    (@call [@method [$element:expr] [$method:ident]] [$($args:expr),*]) => {
        ($element).$method($($args),*)
    };
    (@call [@constructor [$function:path]] [$($args:expr),*]) => {
        $function($($args),*)
    };
    (@call [$($context:tt)*] [$($args:expr),*] $($invalid:tt)+) => {
        compile_error!("el!: unexpected tokens after arguments; use ; between attributes and , between arguments")
    };
    (@call [@condition_builder [$element:expr] [$($children:tt)*] [$condition:expr]]
        [$builder:expr]) => {
        ($element).when($condition, $builder)
    };
    (@call [@condition_builder [$element:expr] [$($children:tt)*] [$condition:expr]]
        [$($invalid:tt)*] $($rest:tt)*) => {
        compile_error!("el!: a condition requires exactly one builder after :")
    };
    ($($invalid:tt)*) => {
        compile_error!("el!: invalid value syntax")
    };
}
