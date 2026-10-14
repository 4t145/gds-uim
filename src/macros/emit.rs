// Resume the surrounding parser after completing a value or node.
#[doc(hidden)]
#[macro_export]
macro_rules! __uim_emit {
    (@emit [@root] [$element:expr] , $(,)?) => { $element };
    (@emit [@child [$parent:expr]] [$element:expr] , $(,)?) => {
        ($parent).child($element)
    };
    (@emit [@spread [$parent:expr]] [$elements:expr] , $(,)?) => {
        ($parent).children($elements)
    };
    (@emit [@child [$parent:expr]] [$element:expr] , $($rest:tt)*) => {
        $crate::__uim_children!(@children [($parent).child($element)] $($rest)*)
    };
    (@emit [@spread [$parent:expr]] [$elements:expr] , $($rest:tt)*) => {
        $crate::__uim_children!(@children [($parent).children($elements)] $($rest)*)
    };
    (@emit [@argument [$($call:tt)*] [$($args:expr),*]] [$value:expr] , $($rest:tt)*) => {
        $crate::__uim_value!(@arguments [$($call)*] [$($args,)* $value] $($rest)*)
    };
    (@emit [@argument [$($call:tt)*] [$($args:expr),*]] [$value:expr] ; $($rest:tt)*) => {
        $crate::__uim_value!(@call [$($call)*] [$($args,)* $value] $($rest)*)
    };
    (@emit [@node_group [$($context:tt)*] [$($tail:tt)*]] [$value:expr] , $(,)?) => {
        $crate::__uim_element!(@stage [$($context)*] [$value] $($tail)*)
    };
    (@emit [@value_group [$($context:tt)*] [$($tail:tt)*]] [$value:expr] ; $(;)?) => {
        $crate::__uim_element!(@stage [$($context)*] [$value] $($tail)*)
    };
    (@emit [@condition_value [$element:expr] [$($children:tt)*] [$($tail:tt)*]]
        [$condition:expr] ;) => {
        $crate::__uim_value!(@arguments [@condition_builder [$element] [$($children)*] [$condition]]
            [] $($tail)*)
    };
    (@emit [@condition_map [$element:expr] [$builder:expr]] [$condition:expr] ;) => {
        ($element).when($condition, $builder)
    };
    (@emit [$($context:tt)*] [$value:expr] $($invalid:tt)*) => {
        compile_error!("el!: unexpected tokens after a value; use .function for unary application, ; between attributes, and , between arguments or children; Rust expressions require $")
    };
    ($($invalid:tt)*) => {
        compile_error!("el!: invalid emit syntax")
    };
}
