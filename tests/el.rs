use gds_uim::el as ui;
use std::cell::Cell;

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
        .List::new {
            flex;
            flex_col;
            gap: 8.0.px;
            when: condition, $|x| x.child("dynamic");
        } [
            "confirm".Button::with_label {
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
        ui! { .List::new },
        ui! { ().List::new },
        ui! { .List::new {} },
        ui! { .List::new [] },
        ui! { .List::new {} [] },
        ui! { ().List::new {} },
        ui! { ().List::new [] },
        ui! { ().List::new {} [] },
    ];

    for element in elements {
        assert_eq!(element, List::new());
    }
    assert_eq!(ui! { .List::new { flex; } }, List::new().flex());
    assert_eq!(ui! { ().List::new { flex } }, List::new().flex());
}

#[test]
fn supports_constructor_arguments_and_qualified_paths() {
    use argument_list::List;

    let a = 3;
    let b = 5;
    let element = ui! { (a, b).List::new { flex } };
    assert_eq!(element.0.calls, ["new(3, 5)", "flex"]);
    assert_eq!(
        ui! { (1, 2,).argument_list::List::new }.0.calls,
        ["new(1, 2)"]
    );
    assert_eq!(ui! { .widgets::List::new }.calls, ["new"]);
    assert_eq!(
        ui! { ("ok",).widgets::Button::with_label [] }.calls,
        ["with_label(ok)"]
    );
}

#[test]
fn supports_each_constructor_form_in_nested_children() {
    let element = ui! {
        .List::new [
            .List::new,
            ().List::new,
            .List::new {},
            .List::new [],
            .widgets::List::new {} [],
            (1, 2).argument_list::List::new { gap: 2.0.px },
            "ok".widgets::Button::with_label [
                .List::new { flex } ["grandchild"]
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
        .List::new {
            pair: $(1, 2);
            set_pair: 3, 4,;
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
        .List::new {
            when: disabled, $|element| {
                skipped.set(skipped.get() + 1);
                element.child("hidden")
            };
            when: $conditions::ENABLED && !disabled, $|element| element.child("enabled");
            when: $conditions::enabled::<u8, u16>(), $move |element| element.child(moved_child)
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
fn when_mappers_support_dsl_values_rust_conditions_and_move_captures() {
    mod conditions {
        pub const ENABLED: bool = true;
        pub fn enabled<T, U>() -> bool {
            std::mem::size_of::<T>() <= std::mem::size_of::<U>()
        }
    }
    fn invert(value: bool) -> bool {
        !value
    }

    let disabled = false;
    let value = 8.0;
    let owned = String::from("moved");
    let element = ui! {
        .List::new {
            when: conditions::ENABLED, {
                flex;
                when: disabled.invert, { gap: value.px };
            };
            when: $conditions::enabled::<u8, u16>() && !disabled, move { child: owned };
            when: true, {};
            when: true, move {}
        } ["last"]
    };
    assert_eq!(element.calls, ["new", "flex", "gap(8)"]);
    assert_eq!(
        element.children,
        [Child::Text("moved".into()), Child::Text("last".into())]
    );
}

#[test]
fn when_mappers_evaluate_conditions_once_and_skip_inactive_blocks() {
    let conditions = Cell::new(0);
    let attributes = Cell::new(0);
    let evaluate = |enabled| {
        conditions.set(conditions.get() + 1);
        enabled
    };

    let element = List::new().map(gds_uim::m! {
        when: $evaluate(false), {
            when: $evaluate(true), { gap: ${ attributes.set(100); 0.0 } };
        };
        when: $(evaluate(true)), {
            gap: ${ attributes.set(attributes.get() + 1); 6.0 };
        };
    });
    assert_eq!(conditions.get(), 2);
    assert_eq!(attributes.get(), 1);
    assert_eq!(element.calls, ["new", "gap(6)"]);
}

#[test]
fn mixes_spreads_with_expression_children() {
    let dynamic = String::from("dynamic");
    let element = ui! {
        .List::new [
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
fn grouped_control_flow_and_indexing_are_rust_children() {
    for selected in [false, true] {
        let evaluated = Cell::new(0);
        let labels = ["indexed"];
        let index = || {
            evaluated.set(evaluated.get() + 1);
            0
        };
        let expected = if selected { "selected" } else { "other" };
        let element = ui! {
            .List::new {
                label: $match selected { true => "selected", false => "other" };
            } [
                $(match selected {
                    true => "selected",
                    false => "other",
                }),
                $(labels[index()]),
                $(if selected { "selected" } else { "other" }),
                $({ evaluated.set(evaluated.get() + 1); "block" }),
            ]
        };

        assert_eq!(evaluated.get(), 2);
        assert_eq!(element.calls, ["new", &format!("label({expected})")]);
        assert_eq!(
            element.children,
            [expected, "indexed", expected, "block"].map(|text| Child::Text(text.to_owned()))
        );
    }
}

#[test]
fn grouped_match_can_be_followed_by_dsl_stages() {
    let finish = |element: List| element.flex_col();
    let selected = Some(List::new());
    let element = ui! {
        .List::new [
            $(match selected {
                Some(element) => element,
                None => panic!("unselected branch must not run"),
            }) { label: "match"; } ["child"].finish { gap: 4.0.px; },
        ]
    };

    assert_eq!(
        element.children,
        [Child::Element(
            List::new()
                .label("match")
                .child("child")
                .flex_col()
                .gap(4.0)
        )]
    );
}

#[test]
fn preserves_temporary_borrows_across_children_and_spreads() {
    struct Borrowed<'a>(Vec<&'a str>);

    impl<'a> Borrowed<'a> {
        fn new() -> Self {
            Self(Vec::new())
        }

        fn child(mut self, text: &'a str) -> Self {
            self.0.push(text);
            self
        }

        fn children(mut self, values: impl IntoIterator<Item = &'a str>) -> Self {
            self.0.extend(values);
            self
        }

        fn into_string(self) -> String {
            self.0.join("/")
        }
    }

    fn borrowed(text: &str) -> Borrowed<'_> {
        Borrowed(vec![text])
    }

    let text = ui! {
        $(&String::from("constructor")).borrowed ["child"]
    }
    .into_string();
    assert_eq!(text, "constructor/child");

    let text = ui! {
        $(&String::from("empty-attributes")).borrowed {} ["child"]
    }
    .into_string();
    assert_eq!(text, "empty-attributes/child");

    let text = ui! {
        .Borrowed::new [
            $String::from("child").as_str(),
            ..$[String::from("spread").as_str()],
        ]
    }
    .into_string();
    assert_eq!(text, "child/spread");

    let text = ui! {
        .Borrowed::new {} [
            $String::from("child").as_str(),
            ..$[String::from("spread").as_str()],
        ]
    }
    .into_string();
    assert_eq!(text, "child/spread");
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
            when: $evaluate() == 2, { child: "condition" };
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
        .List::new { map: Stateful; label: "stateful" } ["child", ..$["rest"]]
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
            $(factory()) [],
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
        ui! { $(factory()) [] },
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
        .List::new [
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
fn postfix_calls_accept_optional_arguments_sections_and_module_paths() {
    let factory = || List::new();
    for element in [
        ui! { .factory },
        ui! { ().factory },
        ui! { .factory {} },
        ui! { .factory [] },
        ui! { .factory {} [] },
        ui! { ().factory {} },
        ui! { ().factory [] },
        ui! { ().factory {} [] },
        ui! { .widgets::List::new },
        ui! { ().widgets::List::new },
    ] {
        assert_eq!(element, List::new());
    }

    let existing = List::new().label("existing");
    let condition = true;
    let element = ui! {
        .factory { flex; when: condition, $|list| list.child("condition") } [
            .factory,
            ().factory,
            .widgets::List::new { flex_col },
            ("factory",).Button::with_label { label: "确定" },
            "constructor".Button::with_label,
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
fn postfix_calls_preserve_type_const_and_lifetime_arguments() {
    fn factory<T, const N: usize>() -> List {
        List::new().pair((std::mem::size_of::<T>() as i32, N as i32))
    }
    fn borrowed<'a: 'static>(label: &'a str) -> List {
        List::new().label(label)
    }

    let element = ui! {
        .factory::<u8, 3> [
            .factory::<u16, 4>,
            ().factory::<Vec<Vec<u8>>, 5>,
            .factory::<fn(i32) -> i32, 6> { flex },
            ().factory::<fn(i32) -> i32, 7> { flex_col },
            ("lifetime").borrowed::<'static>,
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
fn postfix_calls_consume_move_only_values_once_in_source_order() {
    let calls = Cell::new(0);
    let factory = |expected, label: String| {
        assert_eq!(calls.get(), expected);
        calls.set(calls.get() + 1);
        List::new().child(label)
    };
    let owned = String::from("once");
    let once = || List::new().child(owned);

    let element = ui! {
        (0, $String::from("root")).factory { flex } [
            (1, $String::from("nested")).factory [.once],
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
        .List::new {
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
        .widgets::List::new { flex; } [
            "child".Button::with_label ["nested"].frame {
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
    assert_eq!(ui! { .List::new.frame }.0.calls, ["new", "frame(0)"]);
    assert_eq!(
        ui! { ().widgets::List::new.frame }.0.calls,
        ["new", "frame(0)"]
    );
}

#[test]
fn macro_suffixes_map_complete_trees_and_support_type_changing_chains() {
    use gds_uim::m as mapping;

    // No FluentBuilder or ParentElement implementation on the wrapper.
    struct Frame(List);
    impl Frame {
        fn label(self, label: &str) -> Self {
            Self(self.0.label(label))
        }
    }
    fn frame(mut element: List) -> Frame {
        element
            .calls
            .push(format!("frame({})", element.children.len()));
        Frame(element)
    }
    fn finish(wrapped: Frame) -> List {
        wrapped.0
    }

    let actual = ui! {
        .List::new ["first"]
            .mapping! { child: "second"; map: $frame }
            .gds_uim::m! { label: "mapped wrapper" }
            { label: "configured wrapper" }
            .finish
            .::gds_uim::m![flex]
            ["third"]
    };
    assert_eq!(
        actual.calls,
        [
            "new",
            "frame(2)",
            "label(mapped wrapper)",
            "label(configured wrapper)",
            "flex"
        ]
    );
    assert_eq!(
        actual.children,
        [
            Child::Text("first".into()),
            Child::Text("second".into()),
            Child::Text("third".into())
        ]
    );
}

#[test]
fn macro_suffixes_evaluate_the_input_and_callable_once_in_order() {
    use std::cell::RefCell;

    macro_rules! record {
        ($events:expr, $payload:expr) => {{
            $events.borrow_mut().push("make closure");
            let events = &$events;
            let payload = $payload;
            move |element| {
                events.borrow_mut().push("apply closure");
                drop(payload);
                element.flex()
            }
        }};
    }

    let events = RefCell::new(Vec::new());
    let payload = String::from("consumed once");
    let label = String::from("captured");
    let actual = ui! {
        $({ events.borrow_mut().push("element"); List::new() })
            .record!(events, payload)
            .gds_uim::m!(move { label: $label.as_str() })
    };
    assert_eq!(
        *events.borrow(),
        ["element", "make closure", "apply closure"]
    );
    assert_eq!(actual.calls, ["new", "flex", "label(captured)"]);
}

#[test]
fn macro_suffixes_work_in_children_values_and_constructor_arguments() {
    macro_rules! twice {
        () => {
            |value| value * 2.0
        };
    }
    fn label(element: List) -> String {
        element.calls.join(";")
    }
    fn identity<T>(value: T) -> T {
        value
    }

    let actual = ui! {
        .List::new {
            gap: 3.0.twice!().px;
            owned_pair: .List::new.gds_uim::m! { flex }.label, $"right".to_owned();
        } [
            (.List::new.gds_uim::m! { label: "argument" }).identity,
            .List::new.gds_uim::m! { label: "child" },
        ]
    };
    assert_eq!(
        actual.calls,
        ["new", "gap(6)", "owned_pair(new;flex, right)"]
    );
    assert_eq!(
        actual.children,
        [
            Child::Element(List::new().label("argument")),
            Child::Element(List::new().label("child"))
        ]
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
        .List::new {
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
    let constructor = ui! { (1.plus_one, $(2 + 1)).argument_list::List::new };
    assert_eq!(constructor.0.calls, ["new(2, 3)"]);
}

#[test]
fn debug_selectors_defer_expressions_and_support_borrowed_and_move_captures() {
    struct Debuggable;

    impl Debuggable {
        fn debug_selector(self, selector: impl FnOnce() -> String) -> impl FnOnce() -> String {
            selector
        }
    }

    let calls = Cell::new(0);
    let label = String::from("borrowed");
    let selector = ui! {
        Debuggable {
            #? $({ calls.set(calls.get() + 1); &label }) { to_string };
        }
    };
    assert_eq!(calls.get(), 0);
    assert_eq!(selector(), "borrowed");
    assert_eq!(calls.get(), 1);
    assert_eq!(label, "borrowed");

    let selector = {
        let id = String::from("owned");
        ui! { Debuggable { #? move format!("task-{id}") } }
    };
    assert_eq!(selector(), "task-owned");

    fn configure<F>(builder: impl FnOnce(Debuggable) -> F) -> F {
        builder(Debuggable)
    }

    let selector = configure(gds_uim::m! { #? "mapped" { into } });
    assert_eq!(selector(), "mapped");
}

#[test]
fn map_closures_share_attributes_conditions_and_value_syntax() {
    use gds_uim::m as mapping;

    // The macro's input binding must not shadow names from the caller.
    let value = 7.0;
    let enabled = true;
    let skipped = Cell::new(0);
    let actual = ui! {
        .List::new {
            map: $mapping! {
                flex;
                gap: value.px;
                set_pair: 1, 2;
                pair: $(3, 4);
                when: enabled, $mapping! { label: "selected" };
                when: false, $mapping! { gap: ${ skipped.set(skipped.get() + 1); 100.0 } };
            };
        }
    };
    assert_eq!(skipped.get(), 0);
    assert_eq!(
        actual.calls,
        [
            "new",
            "flex",
            "gap(7)",
            "set_pair(1, 2)",
            "pair(3, 4)",
            "label(selected)"
        ]
    );
}

#[test]
fn map_closures_defer_values_and_preserve_borrowed_captures() {
    fn deferred(builder: impl FnMut(List) -> List) -> impl FnMut(List) -> List {
        builder
    }

    let calls = Cell::new(0);
    let label = String::from("borrowed");
    let mut next = 0;
    {
        let mut builder = deferred(gds_uim::m! {
            gap: ${
                calls.set(calls.get() + 1);
                next += 1;
                next as f32
            };
            label: $label.as_str();
        });
        assert_eq!(calls.get(), 0);
        assert_eq!(
            builder(List::new()).calls,
            ["new", "gap(1)", "label(borrowed)"]
        );
        assert_eq!(
            builder(List::new()).calls,
            ["new", "gap(2)", "label(borrowed)"]
        );
    }
    assert_eq!(calls.get(), 2);
    assert_eq!(next, 2);
    assert_eq!(label, "borrowed");
}

#[test]
fn map_closures_support_move_captures_empty_bodies_and_type_changes() {
    fn owned_builder() -> impl FnOnce(List) -> List + 'static {
        let left = String::from("left");
        let right = String::from("right");
        gds_uim::m!(move { owned_pair: left, right })
    }

    assert_eq!(List::new().map(gds_uim::m! {}), List::new());
    assert_eq!(List::new().map(gds_uim::m!(move {})), List::new());
    assert_eq!(
        List::new().map(owned_builder()).calls,
        ["new", "owned_pair(left, right)"]
    );
    let wrapped: Option<List> = List::new().map(gds_uim::m! { map: $Some });
    assert_eq!(wrapped, Some(List::new()));
}

#[test]
fn bare_mapper_values_compose_nested_mappings_and_preserve_types() {
    // Bare mapper values generate closures without requiring an m! import.
    let value = 9.0;
    let calls = Cell::new(0);
    let actual = ui! {
        .List::new {
            label: "before";
            map: {
                flex;
                map: {
                    gap: value.px;
                    set_pair: ${ calls.set(calls.get() + 1); 1 }, 2;
                    when: true, $gds_uim::m! { label: "nested" };
                };
                label: "after";
            };
            map: {};
            map: move {}
        } ["child"]
    };
    assert_eq!(calls.get(), 1);
    assert_eq!(
        actual.calls,
        [
            "new",
            "label(before)",
            "flex",
            "gap(9)",
            "set_pair(1, 2)",
            "label(nested)",
            "label(after)"
        ]
    );
    assert_eq!(actual.children, [Child::Text("child".into())]);

    let wrapped: Option<List> = ui! { .List::new { map: { map: $Some } } };
    assert_eq!(wrapped, Some(List::new()));
}

#[test]
fn bare_move_mappers_own_captures_and_defer_evaluation() {
    struct Deferred;
    impl Deferred {
        fn defer(self, builder: impl FnOnce(List) -> List + 'static) -> impl FnOnce(List) -> List {
            builder
        }
    }

    let calls = std::rc::Rc::new(Cell::new(0));
    let builder = {
        let record = calls.clone();
        let label = String::from("owned");
        ui! {
            $(Deferred) {
                defer: move {
                    gap: ${ record.set(record.get() + 1); 12.0 };
                    label: $label.as_str()
                }
            }
        }
    };
    assert_eq!(calls.get(), 0);
    assert_eq!(
        builder(List::new()).calls,
        ["new", "gap(12)", "label(owned)"]
    );
    assert_eq!(calls.get(), 1);
}

#[test]
fn unified_expressions_support_values_tuples_and_callable_expressions() {
    fn pair(a: i32, b: i32) -> (i32, i32) {
        (a, b)
    }
    fn sum(pair: (i32, i32)) -> i32 {
        pair.0 + pair.1
    }
    fn unit(_: ()) -> usize {
        0
    }
    let existing = List::new();
    assert_eq!(
        ui! { existing { flex } ["child"] },
        List::new().flex().child("child")
    );
    assert_eq!(ui! { "ok".Button::with_label }, Button::with_label("ok"));
    assert_eq!(ui! { (2, 3,).pair.sum }, 5);
    assert_eq!(ui! { $(2, 3).sum }, 5);
    assert_eq!(ui! { $().unit }, 0);
    assert_eq!(ui! { .$(|| 7) }, 7);
    assert_eq!(ui! { 3 .$(|x| x + 1).$(|x| x * 2) }, 8);
    assert_eq!(
        ui! { ("left".String::from, "right".String::from).$(|a, b| a.len() + b.len()) },
        9
    );
    assert_eq!(ui! { 7 }, 7);
    assert_eq!(ui! { true }, true);
    assert_eq!(ui! { -2 }, -2);
}

#[test]
fn macros_are_values_with_all_delimiters_and_qualified_paths() {
    use gds_uim::m as mapping;
    macro_rules! node {
        () => {
            List::new()
        };
    }
    macro_rules! double {
        () => {
            |n| n * 2
        };
    }
    let actual = ui! {
        node! {} { flex } [
            concat!("a", "b"),
            format!["{}", 42],
            node!() { label: "macro child" },
            ..vec!["last"],
        ].mapping! { gap: 3.0.px }
    };
    assert_eq!(actual.calls, ["new", "flex", "gap(3)"]);
    assert_eq!(actual.children[0], Child::Text("ab".into()));
    assert_eq!(actual.children[1], Child::Text("42".into()));
    assert_eq!(
        actual.children[2],
        Child::Element(List::new().label("macro child"))
    );
    assert_eq!(actual.children[3], Child::Text("last".into()));
    assert_eq!(ui! { 3.double![] }, 6);
    assert_eq!(ui! { node!().gds_uim::m! {} }, List::new());
}

#[test]
fn clone_binds_to_a_value_and_is_explicit_for_callables() {
    struct Probe<'a> {
        clones: &'a Cell<usize>,
        text: String,
    }
    impl Clone for Probe<'_> {
        fn clone(&self) -> Self {
            self.clones.set(self.clones.get() + 1);
            Self {
                clones: self.clones,
                text: self.text.clone(),
            }
        }
    }
    fn consume(probe: Probe<'_>) -> String {
        probe.text
    }
    let clones = Cell::new(0);
    let source = Probe {
        clones: &clones,
        text: "owned".into(),
    };
    assert_eq!(ui! { ~source.consume }, "owned");
    assert_eq!(ui! { ~~$(source).consume }, "owned");
    assert_eq!(clones.get(), 3);
    assert_eq!(source.text, "owned");

    let callable = move |prefix: String| prefix + &consume(source);
    assert_eq!(ui! { "cloned ".String::from.~callable }, "cloned owned");
    assert_eq!(clones.get(), 4);
    assert_eq!(ui! { "original ".String::from.callable }, "original owned");
}

#[test]
fn computed_multi_argument_callable_runs_after_inputs_once() {
    let events = std::cell::RefCell::new(Vec::new());
    let log = &events;
    let payload = String::from("once");
    let actual = ui! {
        (
            $({ log.borrow_mut().push("left"); String::from("left") }),
            $({ log.borrow_mut().push("right"); String::from("right") }),
        ).$({
            log.borrow_mut().push("callable");
            move |left, right| {
                log.borrow_mut().push("call");
                drop(payload);
                left + &right
            }
        })
    };
    assert_eq!(actual, "leftright");
    assert_eq!(*events.borrow(), ["left", "right", "callable", "call"]);
}

#[test]
fn id_shorthand_preserves_type_changes_and_works_in_mappers() {
    struct Unnamed;
    impl Unnamed {
        fn id(self, id: &str) -> List {
            List::new().label(id)
        }
    }
    let actual = ui! { Unnamed { # "named"; flex } ["child"] };
    let explicit = ui! { Unnamed { id: "named"; flex } ["child"] };
    assert_eq!(actual, explicit);
    let mapped = ui! { Unnamed.gds_uim::m! { # concat!("map", "ped"); flex_col } };
    assert_eq!(mapped.calls, ["new", "label(mapped)", "flex_col"]);
    assert_eq!(
        ui! { Unnamed.gds_uim::m! { # "mapped"; flex_col } },
        ui! { Unnamed { # "mapped"; flex_col } }
    );
}

#[test]
fn iterator_method_blocks_and_rust_spreads_are_equivalent() {
    let calls = Cell::new(0);
    let render_item = |text: String| {
        calls.set(calls.get() + 1);
        List::new().child(text)
    };
    let items = vec![String::from("a"), String::from("b")];
    let from_rust = ui! { .List::new [..$(items.into_iter().map(render_item))] };
    assert_eq!(calls.get(), 2);
    let items = vec![String::from("a"), String::from("b")];
    let from_dsl = ui! { .List::new [..items { into_iter; map: render_item }] };
    assert_eq!(calls.get(), 4);
    assert_eq!(from_dsl, from_rust);
}

#[test]
fn conditional_handlers_are_constructed_even_when_not_called() {
    let constructed = Cell::new(0);
    let called = Cell::new(0);
    let result = ui! {
        .List::new {
            when: false, $({
                constructed.set(constructed.get() + 1);
                |node| { called.set(called.get() + 1); node }
            });
            when: false, { gap: ${ called.set(100); 1.0 } };
            flex:
        }
    };
    assert_eq!(result.calls, ["new", "flex"]);
    assert_eq!(constructed.get(), 1);
    assert_eq!(called.get(), 0);
}

#[test]
fn generated_names_do_not_capture_user_bindings() {
    let __uim_value = 4.0;
    let __uim_apply = |n: f32| n + 1.0;
    let actual = ui! {
        .List::new { gap: __uim_value }
            .$(|node| node.gap(__uim_apply(__uim_value)))
    };
    assert_eq!(actual.calls, ["new", "gap(4)", "gap(5)"]);
}

#[test]
fn callable_paths_keep_fn_mut_borrows_and_qualified_paths() {
    let mut total = 0;
    let mut accumulate = |n| {
        total += n;
        total
    };
    assert_eq!(ui! { 2.accumulate }, 2);
    assert_eq!(ui! { 3.accumulate }, 5);
    assert_eq!(ui! { 4 .$(&mut accumulate) }, 9);
    assert_eq!(total, 9);
    trait Factory {
        fn make() -> List;
    }
    impl Factory for List {
        fn make() -> List {
            List::new()
        }
    }
    assert_eq!(ui! { .<List as Factory>::make }, List::new());
    assert_eq!(ui! { "owned".::std::string::String::from }, "owned");
}

#[test]
fn alternating_blocks_preserve_order_across_type_changes_and_suffixes() {
    use std::cell::RefCell;
    struct First<'a>(&'a RefCell<Vec<&'static str>>);
    struct Second<'a>(&'a RefCell<Vec<&'static str>>);
    impl<'a> First<'a> {
        fn flex(self) -> Self {
            self.0.borrow_mut().push("flex");
            self
        }
        fn child(self, child: &'static str) -> Self {
            self.0.borrow_mut().push(child);
            self
        }
        fn f1(self) -> Second<'a> {
            self.0.borrow_mut().push("f1");
            Second(self.0)
        }
    }
    impl Second<'_> {
        fn f2(self) -> Self {
            self.0.borrow_mut().push("f2");
            self
        }
        fn child(self, child: &'static str) -> Self {
            self.0.borrow_mut().push(child);
            self
        }
    }
    fn some_post_process(value: Second<'_>) -> Second<'_> {
        value.0.borrow_mut().push("post");
        value
    }
    let events = RefCell::new(Vec::new());
    let value = First(&events);
    let _result = ui! {
        value { flex }
        [$({ events.borrow_mut().push("evaluate x"); "x" }), "y"]
        { f1; f2 }
        ["u", "v"]
        .some_post_process
        ["after"] { f2 }
    };
    assert_eq!(
        *events.borrow(),
        [
            "flex",
            "evaluate x",
            "x",
            "y",
            "f1",
            "f2",
            "u",
            "v",
            "post",
            "after",
            "f2"
        ]
    );

    let result = ui! { .List::new {} {} ["a"] ["b"] { flex } { flex_col } [] {} };
    assert_eq!(result, List::new().child("a").child("b").flex().flex_col());
    let nested = ui! {
        .List::new [
            .List::new ["a"] { flex } ["b"] { label: "nested" },
        ] { flex_col }
    };
    assert_eq!(
        nested.children,
        [Child::Element(
            List::new().child("a").flex().child("b").label("nested")
        )]
    );
    assert_eq!(nested.calls, ["new", "flex_col"]);
}

#[test]
fn arrow_closures_support_arity_patterns_types_and_move_captures() {
    let make = ui! { () => .List::new { flex } };
    assert_eq!(make().calls, ["new", "flex"]);
    assert_eq!(ui! { (2, 3).(a, b) => $(a + b) }, 5);
    assert_eq!(ui! { .() => "zero" }, "zero");

    let configure = ui! { (node: List, width: f32) => node { gap: width } };
    assert_eq!(configure(List::new(), 4.0).calls, ["new", "gap(4)"]);
    let configured = ui! {
        (.List::new, 7.0).(node, width) => node { gap: width }
    };
    assert_eq!(configured.calls, ["new", "gap(7)"]);

    let label = String::from("owned");
    let append = ui! { move (node: List, count: i32) => node [label, format!("{count}")] };
    assert_eq!(
        append(List::new(), 2).children,
        [Child::Text("owned".into()), Child::Text("2".into())]
    );

    let sums: Vec<i32> = ui! {
        $(vec![(1, 2), (3, 4)]) { into_iter; map: ((a, b),) => $(a + b); collect }
    };
    assert_eq!(sums, [3, 7]);
    let mut total = 0;
    let mut accumulate = ui! { (a: i32, b: i32) => ${ total += a + b; } };
    accumulate(2, 3);
    accumulate(4, 5);
    assert_eq!(total, 14);
}

#[test]
fn bare_and_named_closures_compose_and_defer_their_bodies() {
    let calls = Cell::new(0);
    let actual = ui! {
        .List::new {
            map: node => node { flex } ["named"];
            when: false, { gap: ${ calls.set(1); 10.0 } };
            map: { gap: 3.0 };
        }.{ flex_col }
    };
    assert_eq!(calls.get(), 0);
    assert_eq!(actual.calls, ["new", "flex", "gap(3)", "flex_col"]);
    assert_eq!(actual.children, [Child::Text("named".into())]);

    // The RHS of => remains an expression: a bare block returns a mapper.
    let factory = ui! { (_: i32) => {} };
    let identity = factory(1);
    assert_eq!(identity(List::new()), List::new());
    let nested = ui! { (a: i32) => move (b: i32) => $(a + b) };
    assert_eq!(nested(2)(3), 5);
}

#[test]
fn if_fragments_are_lazy_and_preserve_sibling_and_condition_order() {
    use std::cell::RefCell;
    let events = RefCell::new(Vec::new());
    let check = |name, result| {
        events.borrow_mut().push(name);
        result
    };
    let payload = String::from("owned child");
    let actual = ui! {
        .List::new [
            $({ events.borrow_mut().push("before"); "before" }),
            @if (check("first", false) || false)
                $({ events.borrow_mut().push("wrong"); "wrong" }),
            @else if (check("second", true) && !false)
                payload,
                .List::new { flex },
                @if (check("nested", true)) "nested" @end,
            @else if (check("skipped", true))
                "wrong",
            @else
                "wrong",
            @end,
            @if (check("absent", false)) "wrong" @end,
            $({ events.borrow_mut().push("after"); "after" }),
        ]
    };
    assert_eq!(
        *events.borrow(),
        ["before", "first", "second", "nested", "absent", "after"]
    );
    assert_eq!(
        actual.children,
        [
            Child::Text("before".into()),
            Child::Text("owned child".into()),
            Child::Element(List {
                calls: vec!["new".into(), "flex".into()],
                children: vec![]
            }),
            Child::Text("nested".into()),
            Child::Text("after".into()),
        ]
    );
}

#[test]
fn nested_attribute_controls_work_inside_mappers_and_keep_bindings_hygienic() {
    let selected = Some(8.0);
    let actual = ui! {
        .List::new {
            map: {
                @if (false)
                    label: "wrong";
                @else
                    flex;
                    @match (&selected)
                    @case (Some(__uim_value) if *__uim_value > 0.0)
                        gap: $*__uim_value;
                        when: true, { flex_col };
                    @case (_)
                    @end;
                @end;
                @if (false) gap: 99.0 @end;
                label: "after";
            };
        }
    };
    assert_eq!(
        actual.calls,
        ["new", "flex", "gap(8)", "flex_col", "label(after)"]
    );
    assert_eq!(selected, Some(8.0));
}

#[test]
fn match_fragments_support_guards_spreads_and_different_child_types() {
    let calls = Cell::new(0);
    let result: Result<Vec<&str>, &str> = Ok(vec!["a", "b"]);
    let actual = ui! {
        .List::new [
            @match ({ calls.set(calls.get() + 1); &result })
            @case (Ok(items) if items.is_empty())
                .List::new ["empty"],
            @case (Ok(items))
                ..items { iter; map: (item) => $*item },
                "ready",
            @case (Err(error))
                .List::new [format!("error: {error}")],
            @end,
            @match (3)
            @case (0 | 1) "wrong"
            @case (n @ 2..=4 if n == 3) "range"
            @case (_) // Empty fallback adds nothing.
            @end,
        ]
    };
    assert_eq!(calls.get(), 1);
    assert_eq!(result.as_ref().unwrap(), &["a", "b"]);
    assert_eq!(
        actual.children,
        ["a", "b", "ready", "range"].map(|x| Child::Text(x.into()))
    );
}

#[test]
fn value_controls_support_arguments_suffixes_and_closure_bodies() {
    fn finish(node: List) -> List {
        node.label("finished")
    }
    let actual = ui! {
        @if (false)
            .List::new ["wrong"]
        @else
            .List::new ["selected"]
        @end { flex } ["after"].finish {
            gap: @match (Some(3.0))
                 @case (Some(n)) n
                 @case (None) 0.0
                 @end;
        }
    };
    assert_eq!(actual.calls, ["new", "flex", "label(finished)", "gap(3)"]);
    assert_eq!(
        actual.children,
        [Child::Text("selected".into()), Child::Text("after".into())]
    );
    assert_eq!(
        ui! {
            (@if (true) 2 @else 9 @end, 3).(a, b) => $(a + b)
        },
        5
    );
    let render = ui! {
        (ready: bool) => @if (ready) .List::new ["yes"] @else .List::new ["no"] @end
    };
    assert_eq!(render(false).children, [Child::Text("no".into())]);
}

#[test]
fn match_values_preserve_borrows_from_place_expressions() {
    let source = Some(String::from("borrowed"));
    let fallback = String::from("fallback");
    let borrowed = ui! {
        @match (source)
        @case (Some(ref text)) text
        @case (None) $(&fallback)
        @end
    };
    assert_eq!(borrowed, "borrowed");
    assert_eq!(source.as_ref().unwrap(), borrowed);
}

#[test]
fn control_fragments_allow_type_changes_when_arms_agree() {
    struct Initial;
    struct Final {
        tag: &'static str,
    }
    impl Initial {
        fn finish(self, tag: &'static str) -> Final {
            Final { tag }
        }
    }
    impl Final {
        fn label(self, tag: &'static str) -> Self {
            Self { tag }
        }
    }
    let actual = ui! {
        Initial {
            @if (true) finish: "first" @else finish: "other" @end;
            @match (false)
            @case (true) label: "wrong"
            @case (false) label: "last"
            @end;
        }
    };
    assert_eq!(actual.tag, "last");
}
