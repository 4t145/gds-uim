use proc_macro::TokenStream;

mod component;
mod expand;
mod parse;

#[proc_macro_attribute]
pub fn component(args: TokenStream, input: TokenStream) -> TokenStream {
    component::expand(args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn el(input: TokenStream) -> TokenStream {
    match syn::parse::<parse::Expression>(input) {
        Ok(expression) => expand::expression(&expression).into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[proc_macro]
pub fn m(input: TokenStream) -> TokenStream {
    match syn::parse::<parse::Mapper>(input) {
        Ok(mapper) => expand::mapper(&mapper).into(),
        Err(error) => error.into_compile_error().into(),
    }
}
