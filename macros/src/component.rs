use std::collections::HashSet;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Error, Fields, GenericArgument, Ident, ItemStruct, LitStr, Meta, PathArguments,
    Result, Token, Type, Visibility, parse_quote, punctuated::Punctuated,
};

#[derive(Default)]
struct Options {
    into: bool,
    option: bool,
    name: Option<Ident>,
    vis: Option<Visibility>,
    option_setter: Option<Ident>,
}

impl Options {
    fn parse(attr: &Attribute) -> Result<Self> {
        let mut options = Self::default();
        if matches!(attr.meta, Meta::Path(_)) {
            return Ok(options);
        }

        let mut seen = HashSet::new();
        attr.parse_nested_meta(|meta| {
            let Some(key) = meta.path.get_ident() else {
                return Err(meta.error("expected an attr option name"));
            };
            let key = key.to_string();
            if !seen.insert(key.clone()) {
                return Err(meta.error(format!("duplicate attr option `{key}`")));
            }
            match key.as_str() {
                "into" => options.into = true,
                "option" => options.option = true,
                "name" => options.name = Some(meta.value()?.parse::<LitStr>()?.parse()?),
                "vis" => options.vis = Some(meta.value()?.parse::<LitStr>()?.parse()?),
                "option_setter" => {
                    options.option_setter = Some(meta.value()?.parse::<LitStr>()?.parse()?)
                }
                _ => {
                    return Err(meta.error(
                        "unknown attr option; expected into, option, name, vis, or option_setter",
                    ));
                }
            }
            Ok(())
        })?;

        if !options.option
            && let Some(name) = &options.option_setter
        {
            return Err(Error::new_spanned(name, "option_setter requires option"));
        }
        Ok(options)
    }
}

pub fn expand(args: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    if !args.is_empty() {
        return Err(Error::new_spanned(
            args,
            "#[component] does not accept arguments",
        ));
    }

    let mut item: ItemStruct = syn::parse2(input)?;
    let Fields::Named(fields) = &mut item.fields else {
        return Err(Error::new_spanned(
            &item,
            "#[component] requires a struct with named fields",
        ));
    };
    let mut methods = Vec::new();
    let mut parameter_name = String::from("value");
    while item
        .generics
        .const_params()
        .any(|param| param.ident.to_string().trim_start_matches("r#") == parameter_name)
    {
        parameter_name.push('_');
    }
    let parameter = Ident::new(&parameter_name, Span::mixed_site());

    for field in &mut fields.named {
        let mut options = None;
        let mut attrs = Vec::new();
        for attr in std::mem::take(&mut field.attrs) {
            if attr.path().is_ident("attr") {
                if options.is_some() {
                    return Err(Error::new_spanned(attr, "duplicate #[attr] on this field"));
                }
                options = Some(Options::parse(&attr)?);
            } else {
                attrs.push(attr);
            }
        }
        field.attrs = attrs;
        let Some(options) = options else {
            continue;
        };

        let field_name = field.ident.as_ref().expect("named field");
        let name = options.name.as_ref().unwrap_or(field_name);
        let vis = options.vis.unwrap_or_else(|| parse_quote!(pub));
        let attrs = forwarded_attrs(&field.attrs, true)?;
        let field_ty = &field.ty;
        let value_ty = if options.option {
            option_inner(field_ty)?
        } else {
            field_ty
        };
        let (param_ty, value) = if options.into {
            (
                quote!(impl ::core::convert::Into<#value_ty>),
                quote!(::core::convert::Into::into(#parameter)),
            )
        } else {
            (quote!(#value_ty), quote!(#parameter))
        };
        let value = if options.option {
            quote!(::core::option::Option::Some(#value))
        } else {
            value
        };

        methods.push(quote! {
            #(#attrs)*
            #[must_use]
            #vis fn #name(mut self, #parameter: #param_ty) -> Self {
                self.#field_name = #value;
                self
            }
        });

        if let Some(name) = options.option_setter {
            methods.push(quote! {
                #(#attrs)*
                #[must_use]
                #vis fn #name(mut self, #parameter: #field_ty) -> Self {
                    self.#field_name = #parameter;
                    self
                }
            });
        }
    }

    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let attrs = forwarded_attrs(&item.attrs, false)?;
    Ok(quote! {
        #item

        #(#attrs)*
        impl #impl_generics #name #ty_generics #where_clause {
            #(#methods)*
        }
    })
}

fn option_inner(ty: &Type) -> Result<&Type> {
    if let Type::Path(path) = ty
        && path.qself.is_none()
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Option"
        && let PathArguments::AngleBracketed(args) = &segment.arguments
        && args.args.len() == 1
        && let Some(GenericArgument::Type(inner)) = args.args.first()
    {
        return Ok(inner);
    }
    Err(Error::new_spanned(
        ty,
        "attr(option) requires a field of type Option<T>",
    ))
}

// Keep field-only attributes on the field, while mirroring its availability and
// documentation on setters. cfg_attr may contain both kinds, including nested
// cfg_attr, so only forward the relevant parts of it.
fn forwarded_attrs(attrs: &[Attribute], include_docs: bool) -> Result<Vec<TokenStream2>> {
    attrs
        .iter()
        .filter_map(|attr| match forwarded_meta(&attr.meta, include_docs) {
            Ok(Some(meta)) => Some(Ok(quote!(#[#meta]))),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn forwarded_meta(meta: &Meta, include_docs: bool) -> Result<Option<Meta>> {
    if meta.path().is_ident("cfg") || (include_docs && meta.path().is_ident("doc")) {
        return Ok(Some(meta.clone()));
    }
    if let Meta::List(list) = meta
        && list.path.is_ident("cfg_attr")
    {
        let args = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        let mut args = args.into_iter();
        let Some(predicate) = args.next() else {
            return Err(Error::new_spanned(list, "expected a cfg_attr predicate"));
        };
        let mut forwarded = Vec::new();
        for arg in args {
            if let Some(meta) = forwarded_meta(&arg, include_docs)? {
                forwarded.push(meta);
            }
        }
        if !forwarded.is_empty() {
            return Ok(Some(parse_quote!(cfg_attr(#predicate, #(#forwarded),*))));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
