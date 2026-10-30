use std::borrow::Cow;

use gds_uim::{component, el, m};

struct MoveOnly(String);

impl AsRef<str> for MoveOnly {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[component]
struct Button {
    id: MoveOnly,
    #[attr]
    width: f32,
    #[attr(into)]
    height: f32,
    #[attr(into)]
    text: String,
    #[attr(option)]
    badge: std::option::Option<MoveOnly>,
    #[attr(option, into, option_setter = "icon_opt")]
    icon: ::core::option::Option<String>,
    #[attr]
    raw: Option<MoveOnly>,
    #[attr(into)]
    raw_into: Option<String>,
    #[attr]
    disabled: bool,
    listener: Option<Box<dyn Fn() -> usize>>,
}

impl Button {
    fn new(id: MoveOnly) -> Self {
        Self {
            id,
            width: 0.0,
            height: 0.0,
            text: String::new(),
            badge: None,
            icon: None,
            raw: None,
            raw_into: None,
            disabled: false,
            listener: None,
        }
    }

    fn on_click(mut self, listener: impl Fn() -> usize + 'static) -> Self {
        self.listener = Some(Box::new(listener));
        self
    }
}

#[test]
fn uses_generated_setters_in_el_with_a_manual_constructor_and_callback() {
    let id = MoveOnly("button-id".to_owned());
    let badge = MoveOnly("badge".to_owned());
    let raw = MoveOnly("raw".to_owned());
    let captured = String::from("abc");
    let button = el! {
        id.Button::new {
            width: 120.0;
            height: 32_u16;
            text: "Confirm";
            badge: badge;
            icon: "check";
            raw: $Some(raw);
            raw_into: $String::from("optional");
            disabled: true;
            on_click: $move || captured.len();
        }
    };

    assert_eq!(button.id.0, "button-id");
    assert_eq!(button.width, 120.0);
    assert_eq!(button.height, 32.0);
    assert_eq!(button.text, "Confirm");
    assert_eq!(button.badge.as_ref().unwrap().0, "badge");
    assert_eq!(button.icon.as_deref(), Some("check"));
    assert_eq!(button.raw.as_ref().unwrap().0, "raw");
    assert_eq!(button.raw_into.as_deref(), Some("optional"));
    assert!(button.disabled);
    assert_eq!(button.listener.as_ref().unwrap()(), 3);

    let button = el! {
        button.m! { text: "Updated"; disabled: false; icon_opt: None }
    };
    assert_eq!(button.text, "Updated");
    assert!(!button.disabled);
    assert!(button.icon.is_none());

    let icon = Some(String::from("forwarded"));
    let button = button.icon_opt(icon).raw(None).raw_into(None);
    assert_eq!(button.icon.as_deref(), Some("forwarded"));
    assert!(button.raw.is_none());
    assert!(button.raw_into.is_none());
}

#[component]
struct Generic<'a, T = MoveOnly, const N: usize = 1>
where
    T: AsRef<str>,
{
    #[attr]
    values: [T; N],
    #[attr(option)]
    selected: Option<T>,
    #[attr(into)]
    caption: Cow<'a, str>,
    #[attr(name = "r#type")]
    kind: &'a str,
}

#[test]
fn preserves_borrowed_values_generics_defaults_and_where_clauses() {
    let source = String::from("borrowed");
    let generic = Generic {
        values: [MoveOnly("initial".to_owned())],
        selected: None,
        caption: Cow::Borrowed("initial"),
        kind: "initial",
    }
    .values([MoveOnly("replacement".to_owned())])
    .selected(MoveOnly("selected".to_owned()))
    .caption(source.as_str())
    .r#type(source.as_str());

    assert_eq!(generic.values[0].as_ref(), "replacement");
    assert_eq!(generic.selected.as_ref().unwrap().as_ref(), "selected");
    assert!(matches!(generic.caption, Cow::Borrowed("borrowed")));
    assert_eq!(generic.kind, "borrowed");
}

#[test]
#[allow(non_upper_case_globals)]
fn setter_parameters_do_not_shadow_const_generics() {
    #[component]
    struct Label<const r#value: usize, const value_: usize> {
        #[attr(option, into, option_setter = "text_opt")]
        text: Option<String>,
    }

    let label = Label::<3, 4> { text: None }.text("Hello");
    assert_eq!(label.text.as_deref(), Some("Hello"));
    assert!(label.text_opt(None).text.is_none());
}

mod private_fields {
    use super::component;

    #[component]
    pub struct Label {
        #[attr(into, name = "text")]
        content: String,
        #[attr(vis = "pub(crate)")]
        disabled: bool,
    }

    impl Label {
        pub fn new() -> Self {
            Self {
                content: String::new(),
                disabled: false,
            }
        }

        pub fn into_parts(self) -> (String, bool) {
            (self.content, self.disabled)
        }
    }
}

#[test]
fn exposes_renamed_setters_independently_of_field_visibility() {
    let label = private_fields::Label::new().text("public").disabled(true);
    assert_eq!(label.into_parts(), (String::from("public"), true));
}

#[component]
#[derive(Default, Debug)]
struct Conditional {
    #[cfg(any())]
    #[attr(into)]
    missing: MissingType,
    #[cfg_attr(all(), cfg(any()))]
    #[attr(option, option_setter = "missing_opt")]
    missing_option: Option<MissingType>,
    #[cfg_attr(all(), cfg_attr(all(), cfg(any())))]
    #[attr]
    nested_missing: MissingType,
    /// The visible value.
    #[cfg_attr(all(), doc = "Available in this build.")]
    #[attr()]
    value: usize,
}

#[component]
#[cfg_attr(all(), cfg(any()))]
struct Removed {
    #[attr]
    value: MissingType,
}

#[test]
fn preserves_derives_docs_and_conditional_fields() {
    let conditional = Conditional::default().value(42);
    assert_eq!(conditional.value, 42);
    assert_eq!(format!("{conditional:?}"), "Conditional { value: 42 }");
}
