use std::collections::HashSet;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::Ident;

use crate::parse::{
    Attribute, Attributes, Body, Child, Children, Control, ControlKind, Expression, Head, Mapper,
    Section, Value,
};

pub fn expression(expr: &Expression) -> TokenStream {
    let head = match &expr.head {
        Head::Value(val) => value(val),
        Head::Call(args, callable) => apply(args.iter().map(expression).collect(), callable),
    };
    let mut result = body(head, &expr.body);
    for (callable, next_body) in &expr.suffixes {
        result = body(apply(vec![result], callable), next_body);
    }
    result
}

fn value(val: &Value) -> TokenStream {
    match val {
        Value::Rust(expr) => quote! { #expr },
        Value::Clone(tilde, inner) => {
            let inner = value(inner);
            quote_spanned! { tilde.span => (#inner).clone() }
        }
        Value::Mapper(mapping) => mapper(mapping),
        Value::Closure(closure) => {
            let capture = &closure.capture;
            let inputs = &closure.inputs;
            let body = expression(&closure.body);
            quote! { #capture |#inputs| { #body } }
        }
        Value::Control(branches) => control(branches, expression, TokenStream::new()),
    }
}

fn apply(args: Vec<TokenStream>, callable: &Value) -> TokenStream {
    let span = callable.span();
    // Preserve Rust's contextual typing for generic factories taking callbacks
    // (including higher-ranked lifetimes), and its borrowing of Fn/FnMut paths.
    if let Value::Rust(expr) = callable
        && matches!(expr.as_ref(), syn::Expr::Path(_))
    {
        return quote_spanned! { span => #expr(#(#args),*) };
    }
    let callable = value(callable);
    if args.is_empty() {
        return quote_spanned! { span => (#callable)() };
    }
    let types: Vec<_> = (0..args.len()).map(|i| format_ident!("A{i}")).collect();
    let names: Vec<_> = (0..args.len()).map(|i| format_ident!("arg{i}")).collect();
    // A concrete FnOnce bound supplies closure parameter types before checking
    // their bodies. The tuple also evaluates every input before the callable.
    let helper = helper_name(&args, &callable);
    quote_spanned! { span =>
        {
            #[inline]
            fn #helper<#(#types,)* R>(
                args: (#(#types,)*),
                callable: impl ::core::ops::FnOnce(#(#types),*) -> R,
            ) -> R {
                let (#(#names,)*) = args;
                callable(#(#names),*)
            }
            #helper((#(#args,)*), #callable)
        }
    }
}

// Mixed-site spans protect local bindings, but not item names. Avoid capturing
// a caller's function or variable when introducing the local helper function.
fn helper_name(args: &[TokenStream], callable: &TokenStream) -> Ident {
    fn collect(tokens: TokenStream, names: &mut HashSet<String>) {
        for token in tokens {
            match token {
                TokenTree::Ident(ident) => {
                    names.insert(ident.to_string());
                }
                TokenTree::Group(group) => collect(group.stream(), names),
                _ => {}
            }
        }
    }
    let mut names = HashSet::new();
    for tokens in args.iter().chain(std::iter::once(callable)) {
        collect(tokens.clone(), &mut names);
    }
    let mut name = String::from("__uim_apply");
    while names.contains(&name) {
        name.push('_');
    }
    Ident::new(&name, Span::mixed_site())
}

fn binding() -> Ident {
    Ident::new("__uim_value", Span::mixed_site())
}

fn body(mut result: TokenStream, body: &Body) -> TokenStream {
    for section in &body.0 {
        result = match section {
            Section::Attributes(attrs) => attributes(result, attrs),
            Section::Children(items) => children(result, items),
        };
    }
    result
}

// Keep ordinary child calls in one expression so temporary inputs live through
// the complete builder chain. Control fragments bind their parent exactly once.
fn children(mut result: TokenStream, items: &Children) -> TokenStream {
    for child in &items.0 {
        result = match child {
            Child::Element {
                spread,
                expression: expr,
            } => {
                let expr = expression(expr);
                if let Some(spread) = spread {
                    quote_spanned! { spread.spans[0] => (#result).children(#expr) }
                } else {
                    quote! { (#result).child(#expr) }
                }
            }
            Child::Control(branches) => {
                let receiver = binding();
                let selected = control(
                    branches,
                    |items| children(quote! { #receiver }, items),
                    quote! { #receiver },
                );
                quote! {{
                    let #receiver = #result;
                    #selected
                }}
            }
        };
    }
    result
}

fn attributes(initial: TokenStream, attrs: &Attributes) -> TokenStream {
    if attrs.0.is_empty() {
        return initial;
    }
    let receiver = binding();
    let statements = attrs.0.iter().map(|attr| {
        let call = match attr {
            Attribute::Method(method, args) => {
                let args = args.iter().map(expression);
                quote! { #receiver.#method(#(#args),*) }
            }
            Attribute::Control(branches) => control(
                branches,
                |attrs| attributes(quote! { #receiver }, attrs),
                quote! { #receiver },
            ),
            Attribute::Id(hash, expr) => {
                let expr = expression(expr);
                quote_spanned! { hash.span => #receiver.id(#expr) }
            }
            Attribute::DebugSelector(hash, capture, expr) => {
                let expr = expression(expr);
                quote_spanned! { hash.span => #receiver.debug_selector(#capture || { #expr }) }
            }
        };
        quote! { let #receiver = #call; }
    });
    quote! {{
        let #receiver = #initial;
        #(#statements)*
        #receiver
    }}
}

// Fragment controls update the existing builder in the selected arm. Keeping
// the match subject intact preserves Rust's borrowing and place-expression
// semantics, and allows different child types without erasing them ourselves.
fn control<T>(
    branches: &Control<T>,
    mut expand: impl FnMut(&T) -> TokenStream,
    fallback: TokenStream,
) -> TokenStream {
    match &branches.kind {
        ControlKind::If {
            branches,
            otherwise,
        } => {
            let mut result = otherwise.as_ref().map_or(fallback, |branch| expand(branch));
            for (condition, branch) in branches.iter().rev() {
                let branch = expand(branch);
                result = quote! { if (#condition) { #branch } else { #result } };
            }
            result
        }
        ControlKind::Match { value, arms } => {
            let arms = arms.iter().map(|arm| {
                let pattern = &arm.pattern;
                let guard = arm.guard.as_ref().map(|guard| quote! { if #guard });
                let body = expand(&arm.body);
                quote! { #pattern #guard => { #body }, }
            });
            quote! { match (#value) { #(#arms)* } }
        }
    }
}

pub fn mapper(mapping: &Mapper) -> TokenStream {
    let capture = &mapping.capture;
    let receiver = binding();
    let result = attributes(quote! { #receiver }, &mapping.attributes);
    quote! { #capture |#receiver| #result }
}
