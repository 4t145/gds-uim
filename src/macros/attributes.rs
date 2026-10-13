// Expand attributes independently, with local builder bindings that
// keep nested trees within the default macro expansion limit.
#[doc(hidden)]
#[macro_export]
macro_rules! __uim_attributes {
    (@attributes [$element:expr] [$($children:tt)*]) => {
        $crate::__uim_children!(@children [$element] $($children)* ,)
    };
    (@attributes [$element:expr] [$($children:tt)*] ; $($rest:tt)*) => {
        $crate::__uim_attributes!(@attributes [$element] [$($children)*] $($rest)*)
    };
    (@attributes [$element:expr] [$($children:tt)*]
        $method:ident ; $($rest:tt)*) => {{
        let element = ($element).$method();
        $crate::__uim_attributes!(@attributes [element] [$($children)*] $($rest)*)
    }};
    (@attributes [$element:expr] [$($children:tt)*]
        $method:ident : $literal:literal . $($function:ident)::+ ; $($rest:tt)*) => {{
        let element = ($element).$method($($function)::+($literal));
        $crate::__uim_attributes!(@attributes [element] [$($children)*] $($rest)*)
    }};
    (@attributes [$element:expr] [$($children:tt)*]
        $method:ident : $literal:literal ; $($rest:tt)*) => {{
        let element = ($element).$method($literal);
        $crate::__uim_attributes!(@attributes [element] [$($children)*] $($rest)*)
    }};
    (@attributes [$element:expr] [$($children:tt)*]
        $method:ident : $($value:ident)::+ ; $($rest:tt)*) => {{
        let element = ($element).$method($($value)::+);
        $crate::__uim_attributes!(@attributes [element] [$($children)*] $($rest)*)
    }};
    (@attributes [$element:expr] [$($children:tt)*] $($rest:tt)+) => {
        $crate::__uim_attributes!(@attribute_tokens [$element] [$($children)*] [] $($rest)+)
    };
    (@attribute_tokens [$element:expr] [$($children:tt)*] [$($attr:tt)*]
        ; $($rest:tt)*) => {{
        let element = $element;
        let element = $crate::__uim_attributes!(@attribute [element] $($attr)* ;);
        $crate::__uim_attributes!(@attributes [element] [$($children)*] $($rest)*)
    }};
    (@attribute_tokens [$element:expr] [$($children:tt)*] [$($attr:tt)*]
        $first:tt ; $($rest:tt)*) => {
        $crate::__uim_attributes!(@attribute_tokens [$element] [$($children)*]
            [$($attr)* $first] ; $($rest)*)
    };
    (@attribute_tokens [$element:expr] [$($children:tt)*] [$($attr:tt)*]
        $first:tt $second:tt ; $($rest:tt)*) => {
        $crate::__uim_attributes!(@attribute_tokens [$element] [$($children)*]
            [$($attr)* $first $second] ; $($rest)*)
    };
    (@attribute_tokens [$element:expr] [$($children:tt)*] [$($attr:tt)*]
        $first:tt $second:tt $third:tt $($rest:tt)*) => {
        $crate::__uim_attributes!(@attribute_tokens [$element] [$($children)*]
            [$($attr)* $first $second $third] $($rest)*)
    };
    (@attribute_tokens [$element:expr] [$($children:tt)*] [$($attr:tt)*] $($invalid:tt)*) => {
        compile_error!("el!: separate attributes with ; and method arguments with ,")
    };
    (@attribute [$element:expr] $method:ident : ;) => {
        compile_error!("el!: expected at least one argument after :; use method; for a zero-argument method")
    };
    (@attribute [$element:expr] $method:ident : $($args:tt)*) => {
        $crate::__uim_value!(@arguments [@method [$element] [$method]] [] $($args)*)
    };
    (@attribute [$element:expr] $method:ident ($($args:tt)*) ;) => {
        $crate::__uim_value!(@arguments [@method [$element] [$method]] [] $($args)* ;)
    };
    (@attribute [$element:expr] $method:ident ;) => {
        ($element).$method()
    };
    (@attribute [$element:expr] @ $($rest:tt)*) => {
        $crate::__uim_attributes!(@condition [$element] [] [] $($rest)*)
    };
    (@attribute [$element:expr] $($invalid:tt)*) => {
        compile_error!("el!: expected method; method: args; or @condition: builder;")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*] :: $($rest:tt)*) => {
        $crate::__uim_attributes!(@condition [$element] [$($children)*] [$($condition)* ::] $($rest)*)
    };
    (@condition [$element:expr] [$($children:tt)*] [] : $($rest:tt)*) => {
        compile_error!("el!: expected a condition after @")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)+] : $($rest:tt)*) => {
        $crate::__uim_value!(@value [@condition_value [$element] [$($children)*] [$($rest)*]]
            $($condition)+ ;)
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*] ; $($rest:tt)*) => {
        compile_error!("el!: expected : builder after the condition")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*] $next:tt $($rest:tt)*) => {
        $crate::__uim_attributes!(@condition [$element] [$($children)*] [$($condition)* $next] $($rest)*)
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*]) => {
        compile_error!("el!: expected : builder after the condition")
    };
    ($($invalid:tt)*) => {
        compile_error!("el!: invalid attributes syntax")
    };
}
