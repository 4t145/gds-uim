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
    (@attribute [$element:expr] $method:ident => { $($attrs:tt)* } ;) => {
        ($element).$method($crate::m! { $($attrs)* })
    };
    (@attribute [$element:expr] $method:ident => move { $($attrs:tt)* } ;) => {
        ($element).$method($crate::m!(move { $($attrs)* }))
    };
    (@attribute [$element:expr] $method:ident => $($invalid:tt)*) => {
        compile_error!("el!: expected an attribute block after =>; use method => { attributes }; or method => move { attributes };")
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
        compile_error!("el!: expected method; method: args; method => { attributes }; @condition: builder; or @condition => { attributes };")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*] :: $($rest:tt)*) => {
        $crate::__uim_attributes!(@condition [$element] [$($children)*] [$($condition)* ::] $($rest)*)
    };
    (@condition [$element:expr] [$($children:tt)*] [] : $($rest:tt)*) => {
        compile_error!("el!: expected a condition after @")
    };
    (@condition [$element:expr] [$($children:tt)*] [] => $($rest:tt)*) => {
        compile_error!("el!: expected a condition after @")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)+]
        => { $($attrs:tt)* } ;) => {
        $crate::__uim_value!(@value [@condition_map [$element] [$crate::m! { $($attrs)* }]]
            $($condition)+ ;)
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)+]
        => move { $($attrs:tt)* } ;) => {
        $crate::__uim_value!(@value [@condition_map [$element] [$crate::m!(move { $($attrs)* })]]
            $($condition)+ ;)
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)+] => $($invalid:tt)*) => {
        compile_error!("el!: expected an attribute block after =>; use @condition => { attributes }; or @condition => move { attributes };")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)+] : $($rest:tt)*) => {
        $crate::__uim_value!(@value [@condition_value [$element] [$($children)*] [$($rest)*]]
            $($condition)+ ;)
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*] ; $($rest:tt)*) => {
        compile_error!("el!: expected : builder or => { attributes } after the condition")
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*] $next:tt $($rest:tt)*) => {
        $crate::__uim_attributes!(@condition [$element] [$($children)*] [$($condition)* $next] $($rest)*)
    };
    (@condition [$element:expr] [$($children:tt)*] [$($condition:tt)*]) => {
        compile_error!("el!: expected : builder or => { attributes } after the condition")
    };
    ($($invalid:tt)*) => {
        compile_error!("el!: invalid attributes syntax")
    };
}
