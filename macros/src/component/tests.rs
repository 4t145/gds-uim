use super::*;

#[test]
fn rejects_invalid_attr_configuration() {
    let cases = [
        (
            quote!(
                struct Button {
                    #[attr(unknown)]
                    text: String,
                }
            ),
            "unknown attr option",
        ),
        (
            quote!(
                struct Button {
                    #[attr(into, into)]
                    text: String,
                }
            ),
            "duplicate attr option `into`",
        ),
        (
            quote!(
                struct Panel {
                    #[attr(any_element, any_element)]
                    content: AnyElement,
                }
            ),
            "duplicate attr option `any_element`",
        ),
        (
            quote!(
                struct Panel {
                    #[attr(any_element, into)]
                    content: AnyElement,
                }
            ),
            "any_element and into are mutually exclusive",
        ),
        (
            quote!(
                struct Panel {
                    #[attr(into, option, any_element)]
                    content: Option<AnyElement>,
                }
            ),
            "any_element and into are mutually exclusive",
        ),
        (
            quote!(
                struct Button {
                    #[attr(name = "a", name = "b")]
                    text: String,
                }
            ),
            "duplicate attr option `name`",
        ),
        (
            quote!(
                struct Button {
                    #[attr(vis = "pub", vis = "pub(crate)")]
                    text: String,
                }
            ),
            "duplicate attr option `vis`",
        ),
        (
            quote!(
                struct Button {
                    #[attr]
                    #[attr(into)]
                    text: String,
                }
            ),
            "duplicate #[attr]",
        ),
        (
            quote!(
                struct Button {
                    #[attr(option)]
                    text: String,
                }
            ),
            "requires a field of type Option<T>",
        ),
        (
            quote!(
                struct Button {
                    #[attr(option_setter = "text_opt")]
                    text: Option<String>,
                }
            ),
            "option_setter requires option",
        ),
        (
            quote!(
                struct Button(String);
            ),
            "requires a struct with named fields",
        ),
        (
            quote!(
                struct Button;
            ),
            "requires a struct with named fields",
        ),
    ];

    for (input, expected) in cases {
        let error = expand(TokenStream2::new(), input.clone()).unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "{input}: expected {expected:?}, got {error}"
        );
    }
}

#[test]
fn rejects_invalid_component_options() {
    for (args, expected) in [
        (quote!(default), "unknown component option; expected gpui"),
        (
            quote!(gpui = gpui_kit, gpui = gpui_pre),
            "duplicate component option `gpui`",
        ),
        (quote!(gpui), "expected `=`"),
        (quote!(gpui = "gpui_kit"), "expected identifier"),
    ] {
        let error = expand(
            args,
            quote!(
                struct Button {}
            ),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains(expected),
            "expected {expected:?}, got {error}"
        );
    }
}

#[test]
fn forwards_conditional_docs_and_cfg_without_field_only_attributes() {
    let output = expand(
        TokenStream2::new(),
        quote! {
            #[derive(Debug)]
            #[cfg_attr(feature = "example", cfg(unix), derive(Clone))]
            struct Button {
                /// The button label.
                #[cfg_attr(feature = "example", cfg_attr(unix, cfg(test), doc = "Conditional docs", field_only))]
                #[attr(into)]
                text: String,
            }
        },
    )
    .unwrap();
    let file: syn::File = syn::parse2(output).unwrap();
    let syn::Item::Struct(item) = &file.items[0] else {
        panic!("expected the original struct");
    };
    let syn::Item::Impl(implementation) = &file.items[1] else {
        panic!("expected the setter impl");
    };
    let syn::ImplItem::Fn(setter) = &implementation.items[0] else {
        panic!("expected the setter");
    };
    let field = item.fields.iter().next().unwrap();
    let field_attrs = &field.attrs;
    let impl_attrs = &implementation.attrs;
    let method_attrs = &setter.attrs;
    let field_text = quote!(#(#field_attrs)*).to_string();
    let impl_text = quote!(#(#impl_attrs)*).to_string();
    let method_text = quote!(#(#method_attrs)*).to_string();

    assert!(field_text.contains("field_only"));
    assert!(!field_text.contains("attr (into)"));
    assert!(impl_text.contains("cfg (unix)"));
    assert!(!impl_text.contains("derive"));
    assert!(method_text.contains("cfg (test)"));
    assert!(method_text.contains("The button label."));
    assert!(method_text.contains("Conditional docs"));
    assert!(!method_text.contains("field_only"));
}
