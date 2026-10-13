use std::cell::Cell;
use uim::el as ui;

// These builders use GPUI's trait signatures without requiring a window or a
// platform backend. Keep them move-only to catch accidental cloning or reuse.
#[derive(Debug, Default, PartialEq)]
struct List {
    calls: Vec<String>,
    children: Vec<Child>,
}

#[derive(Debug, PartialEq)]
enum Child {
    Element(List),
    Text(String),
}

trait IntoElement {
    fn into_element(self) -> Child;
}

impl IntoElement for List {
    fn into_element(self) -> Child {
        Child::Element(self)
    }
}

impl IntoElement for &str {
    fn into_element(self) -> Child {
        Child::Text(self.to_owned())
    }
}

impl IntoElement for String {
    fn into_element(self) -> Child {
        Child::Text(self)
    }
}

trait ParentElement: Sized {
    fn child(self, child: impl IntoElement) -> Self;
    fn children(self, children: impl IntoIterator<Item = impl IntoElement>) -> Self;
}

impl ParentElement for List {
    fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_element());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(IntoElement::into_element));
        self
    }
}

trait FluentBuilder: Sized {
    fn map<T>(self, builder: impl FnOnce(Self) -> T) -> T {
        builder(self)
    }

    fn when(self, condition: bool, builder: impl FnOnce(Self) -> Self) -> Self {
        if condition { builder(self) } else { self }
    }
}

impl<T: IntoElement> FluentBuilder for T {}

impl List {
    fn new() -> Self {
        Self {
            calls: vec!["new".into()],
            ..Self::default()
        }
    }

    fn flex(mut self) -> Self {
        self.calls.push("flex".into());
        self
    }

    fn flex_col(mut self) -> Self {
        self.calls.push("flex_col".into());
        self
    }

    fn gap(mut self, value: f32) -> Self {
        self.calls.push(format!("gap({value})"));
        self
    }

    fn label(mut self, label: &str) -> Self {
        self.calls.push(format!("label({label})"));
        self
    }

    fn pair(mut self, value: (i32, i32)) -> Self {
        self.calls.push(format!("pair({}, {})", value.0, value.1));
        self
    }

    fn set_pair(mut self, left: i32, right: i32) -> Self {
        self.calls.push(format!("set_pair({left}, {right})"));
        self
    }

    fn on_click(mut self, _listener: impl FnOnce(i32, i32, i32)) -> Self {
        self.calls.push("on_click".into());
        self
    }

    fn owned_pair(mut self, left: String, right: String) -> Self {
        self.calls.push(format!("owned_pair({left}, {right})"));
        self
    }
}

struct Button;

impl Button {
    fn with_label(label: &str) -> List {
        List {
            calls: vec![format!("with_label({label})")],
            ..List::default()
        }
    }
}

mod widgets {
    pub(super) use super::{Button, List};
}

mod argument_list {
    pub(super) struct List(pub(super) super::List);

    impl List {
        pub(super) fn new(left: i32, right: i32) -> Self {
            Self(super::List {
                calls: vec![format!("new({left}, {right})")],
                ..super::List::default()
            })
        }

        pub(super) fn flex(self) -> Self {
            Self(self.0.flex())
        }

        pub(super) fn gap(self, value: f32) -> Self {
            Self(self.0.gap(value))
        }
    }

    impl super::IntoElement for List {
        fn into_element(self) -> super::Child {
            super::IntoElement::into_element(self.0)
        }
    }
}

fn px(value: f32) -> f32 {
    value
}

#[test]
fn builds_the_documented_tree_in_source_order() {
    let condition = true;
    let rest = ["first", "second"];
    let actual = ui! {
        List {
            flex;
            flex_col;
            gap: 8.0.px;
            @condition: $|x| x.child("dynamic");
        } [
            Button with_label("confirm") {
                label: "确定"
            },
            ..rest,
        ]
    };

    assert_eq!(actual.calls, ["new", "flex", "flex_col", "gap(8)"]);
    assert_eq!(
        actual.children,
        [
            Child::Text("dynamic".into()),
            Child::Element(List {
                calls: vec!["with_label(confirm)".into(), "label(确定)".into()],
                children: vec![],
            }),
            Child::Text("first".into()),
            Child::Text("second".into()),
        ]
    );
}

#[test]
fn accepts_empty_and_omitted_sections() {
    let elements = [
        ui! { List },
        ui! { List() },
        ui! { List {} },
        ui! { List [] },
        ui! { List {} [] },
        ui! { List() {} },
        ui! { List() [] },
        ui! { List() {} [] },
    ];

    for element in elements {
        assert_eq!(element, List::new());
    }
    assert_eq!(ui! { List { flex; } }, List::new().flex());
    assert_eq!(ui! { List() { flex } }, List::new().flex());
}

#[test]
fn supports_constructor_arguments_and_qualified_paths() {
    use argument_list::List;

    let a = 3;
    let b = 5;
    let element = ui! { List(a, b) { flex } };
    assert_eq!(element.0.calls, ["new(3, 5)", "flex"]);
    assert_eq!(ui! { argument_list::List(1, 2,) }.0.calls, ["new(1, 2)"]);
    assert_eq!(ui! { widgets::List }.calls, ["new"]);
    assert_eq!(
        ui! { widgets::Button with_label("ok",) [] }.calls,
        ["with_label(ok)"]
    );
}

#[test]
fn supports_each_constructor_form_in_nested_children() {
    let element = ui! {
        List [
            List,
            List(),
            List {},
            List [],
            widgets::List {} [],
            argument_list::List(1, 2) { gap: 2.0.px },
            widgets::Button with_label("ok") [
                List { flex } ["grandchild"]
            ]
        ]
    };

    assert_eq!(element.children.len(), 7);
    for child in &element.children[..5] {
        assert_eq!(*child, Child::Element(List::new()));
    }
    assert_eq!(
        element.children[5],
        argument_list::List::new(1, 2).gap(2.0).into_element()
    );
    assert_eq!(
        element.children[6],
        Child::Element(Button::with_label("ok").child(List::new().flex().child("grandchild")))
    );
}

#[test]
fn preserves_rust_expressions_and_closure_commas_in_attributes() {
    let element = ui! {
        List {
            pair: $(1, 2);
            set_pair(3, 4,);
            on_click: $|event, window, context| {
                let _ = (event, window, context);
            };
            map: $|element| element.label(concat!("con", "firm"))
        }
    };

    assert_eq!(
        element.calls,
        [
            "new",
            "pair(1, 2)",
            "set_pair(3, 4)",
            "on_click",
            "label(confirm)"
        ]
    );
}

#[test]
fn applies_conditions_without_running_disabled_builders() {
    mod conditions {
        pub const ENABLED: bool = true;
        pub fn enabled<T, U>() -> bool {
            std::mem::size_of::<T>() <= std::mem::size_of::<U>()
        }
    }

    let skipped = Cell::new(0);
    let disabled = false;
    let moved_child = String::from("moved");
    let element = ui! {
        List {
            @disabled: $|element| {
                skipped.set(skipped.get() + 1);
                element.child("hidden")
            };
            @$conditions::ENABLED && !disabled: $|element| element.child("enabled");
            @$conditions::enabled::<u8, u16>(): $move |element| element.child(moved_child)
        } ["last"]
    };

    assert_eq!(skipped.get(), 0);
    assert_eq!(
        element.children,
        [
            Child::Text("enabled".into()),
            Child::Text("moved".into()),
            Child::Text("last".into()),
        ]
    );
}

#[test]
fn mixes_spreads_with_expression_children() {
    let dynamic = String::from("dynamic");
    let element = ui! {
        List [
            "first",
            ..$std::iter::empty::<&str>(),
            ..$["second", "third"].into_iter().map(String::from),
            $(dynamic),
            $({ String::from("block") }),
            $(if true { "conditional" } else { "unreachable" }),
            $(concat!("mac", "ro")),
            ..$std::iter::once("last")
        ]
    };

    let expected = [
        "first",
        "second",
        "third",
        "dynamic",
        "block",
        "conditional",
        "macro",
        "last",
    ];
    assert_eq!(
        element.children,
        expected.map(|value| Child::Text(value.into()))
    );
}

#[test]
fn evaluates_constructors_attributes_and_spreads_once() {
    let calls = Cell::new(0);
    let evaluate = || {
        let value = calls.get();
        calls.set(value + 1);
        value
    };

    let element = ui! {
        $({ assert_eq!(evaluate(), 0); List::new() }) {
            gap: $evaluate() as f32;
            @$evaluate() == 2: $|element| element.child("condition");
        } [
            ..$std::iter::once({ assert_eq!(evaluate(), 3); "spread" }),
            $({ assert_eq!(evaluate(), 4); List::new() }) { flex } ["nested"]
        ]
    };

    assert_eq!(calls.get(), 5);
    assert_eq!(element.calls, ["new", "gap(1)"]);
    assert_eq!(element.children.len(), 3);
    assert_eq!(
        element.children[2],
        Child::Element(List::new().flex().child("nested"))
    );
}

#[test]
fn preserves_type_changing_builder_methods() {
    struct Stateful(List);

    impl Stateful {
        fn label(self, value: &str) -> Self {
            Self(self.0.label(value))
        }
    }

    impl ParentElement for Stateful {
        fn child(self, child: impl IntoElement) -> Self {
            Self(self.0.child(child))
        }

        fn children(self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
            Self(self.0.children(children))
        }
    }

    let element: Stateful = ui! {
        List { map: Stateful; label: "stateful" } ["child", ..$["rest"]]
    };
    assert_eq!(element.0.calls, ["new", "label(stateful)"]);
    assert_eq!(element.0.children.len(), 2);
}

#[test]
fn dollar_expressions_accept_factories_fields_and_existing_values() {
    struct Holder {
        child: List,
    }
    let holder = Holder { child: List::new() };
    let factory = || List::new();
    let element = ui! {
        $factory().flex() { gap: 8.0.px } [
            $Button::with_label("confirm") { label: "确定" },
            $holder.child { flex_col },
            $factory() [],
        ]
    };

    assert_eq!(element.calls, ["new", "flex", "gap(8)"]);
    assert_eq!(
        element.children,
        [
            Child::Element(Button::with_label("confirm").label("确定")),
            Child::Element(List::new().flex_col()),
            Child::Element(List::new()),
        ]
    );

    let existing = List::new();
    assert_eq!(ui! { $existing }, List::new());
    for element in [
        ui! { $factory() },
        ui! { $factory() {} },
        ui! { $factory() [] },
        ui! { $factory() {} [] },
        ui! { $widgets::List::new() },
    ] {
        assert_eq!(element, List::new());
    }
}

#[test]
fn dollar_expressions_preserve_generics_and_grouped_rust_syntax() {
    fn factory<T, U>() -> List {
        List::new().pair((
            std::mem::size_of::<T>() as i32,
            std::mem::size_of::<U>() as i32,
        ))
    }
    trait Factory {
        fn make() -> List;
    }
    impl Factory for List {
        fn make() -> List {
            List::new().flex()
        }
    }

    let enabled = true;
    let element = ui! {
        List [
            $factory::<u8, u16>(),
            $factory::<Vec<Vec<u8>>, Vec<(u32, u64)>>() { flex },
            $<List as Factory>::make(),
            $(if enabled { List::new() } else { List::new().flex() }) { flex_col },
            $(["indexed"][0]),
        ]
    };
    assert_eq!(
        element.children,
        [
            Child::Element(factory::<u8, u16>()),
            Child::Element(factory::<Vec<Vec<u8>>, Vec<(u32, u64)>>().flex()),
            Child::Element(<List as Factory>::make()),
            Child::Element(List::new().flex_col()),
            Child::Text("indexed".into()),
        ]
    );
}

#[test]
fn dollar_expressions_are_evaluated_once_in_source_order() {
    let calls = Cell::new(0);
    let factory = |expected| {
        assert_eq!(calls.get(), expected);
        calls.set(calls.get() + 1);
        List::new()
    };
    let enabled = true;

    let element = ui! {
        $ factory(0).flex() {
            gap: ${ factory(1); px(8.0) };
        } [
            $ factory(2) { label: "nested" } [$ factory(3)],
            ..$std::iter::once({ factory(4); "spread" }),
            $ (if enabled { factory(5) } else { unreachable!() }),
        ]
    };

    assert_eq!(calls.get(), 6);
    assert_eq!(element.calls, ["new", "flex", "gap(8)"]);
    assert_eq!(
        element.children,
        [
            Child::Element(List::new().label("nested").child(List::new())),
            Child::Text("spread".into()),
            Child::Element(List::new()),
        ]
    );
}

#[test]
fn hash_factories_accept_optional_arguments_sections_and_module_paths() {
    let factory = || List::new();
    for element in [
        ui! { #factory },
        ui! { #factory() },
        ui! { #factory {} },
        ui! { #factory [] },
        ui! { #factory {} [] },
        ui! { #factory() {} },
        ui! { #factory() [] },
        ui! { #factory() {} [] },
        ui! { #widgets::List::new },
        ui! { #widgets::List::new() },
    ] {
        assert_eq!(element, List::new());
    }

    let existing = List::new().label("existing");
    let condition = true;
    let element = ui! {
        #factory { flex; @condition: $|list| list.child("condition") } [
            #factory,
            #factory(),
            #widgets::List::new { flex_col },
            #Button::with_label("factory",) { label: "确定" },
            Button with_label("constructor"),
            $existing,
            ..$["spread"],
        ]
    };
    assert_eq!(element.calls, ["new", "flex"]);
    assert_eq!(
        element.children,
        [
            Child::Text("condition".into()),
            Child::Element(List::new()),
            Child::Element(List::new()),
            Child::Element(List::new().flex_col()),
            Child::Element(Button::with_label("factory").label("确定")),
            Child::Element(Button::with_label("constructor")),
            Child::Element(List::new().label("existing")),
            Child::Text("spread".into()),
        ]
    );
}

#[test]
fn hash_factories_preserve_type_const_and_lifetime_arguments() {
    fn factory<T, const N: usize>() -> List {
        List::new().pair((std::mem::size_of::<T>() as i32, N as i32))
    }
    fn borrowed<'a: 'static>(label: &'a str) -> List {
        List::new().label(label)
    }

    let element = ui! {
        #factory::<u8, 3> [
            #factory::<u16, 4>,
            #factory::<Vec<Vec<u8>>, 5>(),
            #factory::<fn(i32) -> i32, 6> { flex },
            #factory::<fn(i32) -> i32, 7>() { flex_col },
            #borrowed::<'static>("lifetime"),
        ]
    };
    assert_eq!(element.calls, factory::<u8, 3>().calls);
    assert_eq!(
        element.children,
        [
            Child::Element(factory::<u16, 4>()),
            Child::Element(factory::<Vec<Vec<u8>>, 5>()),
            Child::Element(factory::<fn(i32) -> i32, 6>().flex()),
            Child::Element(factory::<fn(i32) -> i32, 7>().flex_col()),
            Child::Element(borrowed("lifetime")),
        ]
    );
}

#[test]
fn hash_factories_consume_move_only_values_once_in_source_order() {
    let calls = Cell::new(0);
    let factory = |expected, label: String| {
        assert_eq!(calls.get(), expected);
        calls.set(calls.get() + 1);
        List::new().child(label)
    };
    let owned = String::from("once");
    let once = || List::new().child(owned);

    let element = ui! {
        #factory(0, $String::from("root")) { flex } [
            #factory(1, $String::from("nested")) [#once],
            ..$std::iter::once({ assert_eq!(calls.get(), 2); "spread" }),
        ]
    };

    assert_eq!(calls.get(), 2);
    assert_eq!(element.calls, ["new", "flex"]);
    assert_eq!(
        element.children,
        [
            Child::Text("root".into()),
            Child::Element(List::new().child("nested").child(List::new().child("once"))),
            Child::Text("spread".into()),
        ]
    );
}

#[test]
fn distinguishes_dsl_application_from_rust_fields_and_calculations() {
    struct Source {
        b: f32,
    }
    fn b(source: Source) -> f32 {
        source.b * 2.0
    }
    mod units {
        pub(super) fn double(value: f32) -> f32 {
            value * 2.0
        }
    }
    fn offset<T, U, const N: usize>(value: f32) -> f32 {
        let _ = (std::mem::size_of::<T>(), std::mem::size_of::<U>());
        value + N as f32
    }

    let a = Source { b: 3.0 };
    let raw = Source { b: 5.0 };
    // The macro's own local builder bindings must not capture this variable.
    let element = 7.0;
    let actual = ui! {
        List {
            gap: a.b;
            gap: $raw.b;
            gap: $(raw.b + 1.0).px.units::double;
            gap: -3.5.px.offset::<Vec<Vec<u8>>, fn(i32, i32) -> i32, 2>;
            gap: element;
        }
    };
    assert_eq!(
        actual.calls,
        ["new", "gap(6)", "gap(5)", "gap(12)", "gap(-1.5)", "gap(7)"]
    );
}

#[test]
fn suffixes_wrap_complete_trees_and_preserve_wrapper_types() {
    // Deliberately has no ParentElement implementation: children must already
    // be attached to the List when the wrapper is called.
    struct Frame(List);
    impl Frame {
        fn label(self, value: &str) -> Self {
            Self(self.0.label(value))
        }
    }
    fn frame(mut element: List) -> Frame {
        element
            .calls
            .push(format!("frame({})", element.children.len()));
        Frame(element)
    }
    fn finish(mut wrapped: Frame) -> List {
        wrapped.0.calls.push("finish".into());
        wrapped.0
    }
    fn identity<T>(value: T) -> T {
        value
    }

    let existing = List::new().label("existing");
    let actual = ui! {
        #widgets::List::new { flex; } [
            Button with_label("child") ["nested"].frame {
                label: "wrapper";
            }.finish,
            $(existing).identity::<List>,
            ..$std::iter::once("spread"),
        ].frame { label: "root"; }.finish
    };
    assert_eq!(
        actual.calls,
        ["new", "flex", "frame(3)", "label(root)", "finish"]
    );
    let Child::Element(child) = &actual.children[0] else {
        panic!("expected wrapped child");
    };
    assert_eq!(
        child.calls,
        ["with_label(child)", "frame(1)", "label(wrapper)", "finish"]
    );
    assert_eq!(child.children, [Child::Text("nested".into())]);
    assert_eq!(
        actual.children[1],
        Child::Element(List::new().label("existing"))
    );
    assert_eq!(actual.children[2], Child::Text("spread".into()));
    assert_eq!(ui! { List.frame }.0.calls, ["new", "frame(0)"]);
    assert_eq!(
        ui! { #widgets::List::new().frame }.0.calls,
        ["new", "frame(0)"]
    );
}

#[test]
fn multiple_arguments_and_fn_once_suffixes_are_evaluated_once_in_order() {
    let calls = Cell::new(0);
    let evaluate = |expected| {
        assert_eq!(calls.get(), expected);
        calls.set(expected + 1);
        expected
    };
    let payload = String::from("once");
    let once = |value: f32| {
        assert_eq!(evaluate(4), 4);
        drop(payload);
        value
    };
    let actual = ui! {
        List {
            owned_pair: $evaluate(0).to_string(), $evaluate(1).to_string();
            set_pair: $(evaluate(2)), $(evaluate(3)),;
            gap: 6.0.once;
            pair: $(7, 8);
            on_click: $|event, window, context| {
                let _ = (event, window, context);
            }
        }
    };
    assert_eq!(calls.get(), 5);
    assert_eq!(
        actual.calls,
        [
            "new",
            "owned_pair(0, 1)",
            "set_pair(2, 3)",
            "gap(6)",
            "pair(7, 8)",
            "on_click"
        ]
    );

    fn plus_one(value: i32) -> i32 {
        value + 1
    }
    let constructor = ui! { argument_list::List(1.plus_one, $(2 + 1)) };
    assert_eq!(constructor.0.calls, ["new(2, 3)"]);
}
