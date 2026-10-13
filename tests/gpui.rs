use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gds_uim::{el, m};
use gpui::{
    AnyElement, App, Bounds, Display, Div, FlexDirection, InteractiveText, IntoElement, Modifiers,
    MouseButton, Pixels, RenderOnce, SharedString, Stateful, StyleRefinement, StyledText,
    TestAppContext, Window, deferred, div, point, prelude::*, px, size,
};

// Build fresh elements inside Render so GPUI owns their frame allocations and
// installs event handlers on the window's rendered view.
struct TestView(Box<dyn FnMut() -> AnyElement>);

#[test]
fn map_closures_infer_real_gpui_style_and_element_types() {
    fn refine(builder: impl FnOnce(StyleRefinement) -> StyleRefinement) -> StyleRefinement {
        builder(StyleRefinement::default())
    }

    let refinement = refine(m! {
        bg: 0x818cf8.gpui::rgb;
        p: 8.0.px;
        border_1;
    });
    assert_eq!(refinement.background, Some(gpui::rgb(0x818cf8).into()));
    assert_eq!(refinement.padding.top, Some(px(8.0).into()));
    assert_eq!(refinement.border_widths.top, Some(px(1.0).into()));

    let mut element: Stateful<Div> = div().map(m! {
        id: "styled";
        @true: $m! { gap: 4.0.px };
        hover => { bg: 0x818cf8.gpui::rgb };
        active => { bg: 0x6366f1.gpui::rgb };
        focus => { border_2; border_color: 0xffffff.gpui::rgb };
        group_hover: "group", $m! { text_color: 0xffffff.gpui::rgb };
    });
    assert_eq!(element.style().gap.height, Some(px(4.0).into()));

    let mut vector = gpui::svg().when(true, m! { size: 16.0.px });
    assert_eq!(vector.style().size.width, Some(px(16.0).into()));
    let mut custom = List::new().map(m! { flex; gap: 6.0.px });
    assert_eq!(custom.style().display, Some(Display::Flex));
    assert_eq!(custom.style().gap.height, Some(px(6.0).into()));

    let mut shortcut: Stateful<Div> = gds_uim::div! {
        map => {
            id: "mapped-shortcut";
            hover => { bg: 0x818cf8.gpui::rgb };
            p: 8.0.px
        }
    };
    assert_eq!(shortcut.style().padding.top, Some(px(8.0).into()));

    let mut suffixed: Stateful<Div> = gds_uim::div! {
        { flex; }
        .gds_uim::m! { id: "suffixed-shortcut"; hover => { bg: 0x475569.gpui::rgb } }
        { gap: 6.0.px }
    };
    assert_eq!(suffixed.style().display, Some(Display::Flex));
    assert_eq!(suffixed.style().gap.height, Some(px(6.0).into()));
}

impl Render for TestView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        (self.0)()
    }
}

// Small application components backed by actual GPUI elements. Their styling,
// children, conversion, layout, and painting all use GPUI's implementations.
#[derive(IntoElement)]
struct List(Div);

impl List {
    fn new() -> Self {
        Self(div())
    }

    fn on_children_prepainted(
        self,
        listener: impl Fn(Vec<Bounds<Pixels>>, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self(self.0.on_children_prepainted(listener))
    }
}

impl Styled for List {
    fn style(&mut self) -> &mut StyleRefinement {
        self.0.style()
    }
}

impl ParentElement for List {
    fn extend(&mut self, children: impl IntoIterator<Item = AnyElement>) {
        self.0.extend(children);
    }
}

impl RenderOnce for List {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.0
    }
}

#[derive(IntoElement)]
struct Button {
    element: Div,
    label: SharedString,
}

impl Button {
    fn with_label(label: impl Into<SharedString>) -> Self {
        Self {
            element: div(),
            label: label.into(),
        }
    }

    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.element.style()
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.element.child(self.label)
    }
}

#[test]
fn accepts_optional_sections_with_real_gpui_components() {
    let elements = [
        el! { List },
        el! { List() },
        el! { List {} },
        el! { List [] },
        el! { List {} [] },
        el! { List() {} },
        el! { List() [] },
        el! { List() {} [] },
    ];

    for mut element in elements {
        assert_eq!(element.style().display, None);
        let _: AnyElement = element.into_any_element();
    }

    let mut element = el! { List { flex; flex_col; gap: 8.0.px } };
    let style = element.style();
    assert_eq!(style.display, Some(Display::Flex));
    assert_eq!(style.flex_direction, Some(FlexDirection::Column));
    assert_eq!(style.gap.height, Some(px(8.0).into()));

    let _: Div = el! { $(div()) {} [] };
    let _: Div = el! { $div() {} [] };
    let _: Div = el! { #div };
    let _: Div = el! { #div() };
    let _: Div = el! { #gpui::div {} [] };
    let _: gpui::Deferred = el! { #deferred($div()) };
}

#[gpui::test]
fn lays_out_the_documented_tree_with_real_gpui(cx: &mut TestAppContext) {
    let root_bounds = Rc::new(RefCell::new(Vec::new()));
    let nested_bounds = Rc::new(RefCell::new(Vec::new()));
    let record_root = root_bounds.clone();
    let record_nested = nested_bounds.clone();
    let condition = Rc::new(Cell::new(true));
    let show_dynamic = condition.clone();

    let (view, visual) = cx.add_window_view(move |_, _| {
        TestView(Box::new(move || {
            let record_root = record_root.clone();
            let record_nested = record_nested.clone();
            let rest = [div().w(px(20.0)).h(px(30.0)), div().w(px(20.0)).h(px(40.0))];
            let existing = div().size(px(16.0));

            el! {
                List {
                    flex;
                    flex_col;
                    w: 200.0.px;
                    gap: 8.0.px;
                    on_children_prepainted: $move |bounds, _window, _cx| {
                        *record_root.borrow_mut() = bounds;
                    };
                    @$show_dynamic.get(): $|list| list.child(div().w(px(20.0)).h(px(10.0)));
                } [
                    #div {
                        flex;
                        flex_col;
                        w: 40.0.px;
                        gap: 3.0.px;
                        on_children_prepainted: $move |bounds, _window, _cx| {
                            *record_nested.borrow_mut() = bounds;
                        };
                    } [
                        #div { w: 30.0.px; h: 5.0.px },
                        #div() { w: 30.0.px; h: 7.0.px },
                    ],
                    Button with_label("confirm") { label: "确定"; w: 50.0.px; h: 24.0.px },
                    ..rest,
                    $existing,
                ]
            }
            .into_any_element()
        }))
    });

    let assert_column = |heights: &[f32]| {
        let bounds = root_bounds.borrow();
        assert_eq!(bounds.len(), heights.len());
        let mut expected_top = px(0.0);
        for (child, expected_height) in bounds.iter().zip(heights) {
            assert_eq!(child.origin.y, expected_top);
            assert_eq!(child.size.height, px(*expected_height));
            expected_top += px(expected_height + 8.0);
        }
    };
    assert_column(&[10.0, 15.0, 24.0, 30.0, 40.0, 16.0]);

    {
        let nested = nested_bounds.borrow();
        assert_eq!(nested.len(), 2);
        assert_eq!(nested[0].size, size(px(30.0), px(5.0)));
        assert_eq!(nested[1].size, size(px(30.0), px(7.0)));
        assert_eq!(nested[1].origin.y - nested[0].bottom(), px(3.0));
    }

    condition.set(false);
    view.update(visual, |_, cx| cx.notify());
    visual.run_until_parked();
    assert_column(&[15.0, 24.0, 30.0, 40.0, 16.0]);
}

#[gpui::test]
fn constructs_and_paints_gpui_text_with_constructor_arguments(cx: &mut TestAppContext) {
    let layouts = Rc::new(RefCell::new(Vec::new()));
    let record_layouts = layouts.clone();

    let (_view, _visual) = cx.add_window_view(move |_, _| {
        TestView(Box::new(move || {
            let texts: [StyledText; 2] = [
                el! { gpui::StyledText("构造器参数") },
                el! { StyledText new("指定构造器") },
            ];
            let text = StyledText::new("两个参数");
            *record_layouts.borrow_mut() = texts
                .iter()
                .chain(std::iter::once(&text))
                .map(|text| text.layout().clone())
                .collect();
            let interactive: InteractiveText = el! { gpui::InteractiveText("text-id", text) };

            el! {
                #div {
                    flex;
                    flex_col;
                    w: 200.0.px;
                } [
                    ..texts,
                    $interactive,
                ]
            }
            .into_any_element()
        }))
    });

    let layouts = layouts.borrow();
    assert_eq!(layouts.len(), 3);
    for (layout, expected) in layouts.iter().zip(["构造器参数", "指定构造器", "两个参数"])
    {
        assert_eq!(layout.text(), expected);
        assert!(layout.bounds().size.width > px(0.0));
    }
}

#[test]
fn uses_gpui_conditions_and_evaluates_expressions_once() {
    let calls = Cell::new(0);
    let skipped = Cell::new(0);
    let disabled = false;
    let enabled = true;
    let evaluate = || {
        calls.set(calls.get() + 1);
        px(8.0)
    };

    let mut element: Stateful<Div> = el! {
        $({ calls.set(calls.get() + 1); div() }) {
            id: "condition-test";
            flex;
            @ disabled: $|element| {
                skipped.set(skipped.get() + 1);
                element.gap(px(100.0))
            };
            @ $enabled && !disabled: $|element| element.gap(evaluate());
        } [
            "标题",
            ..$std::iter::once({ calls.set(calls.get() + 1); div() }),
            #div { flex } ["内容"],
        ]
    };

    assert_eq!(skipped.get(), 0);
    assert_eq!(calls.get(), 3);
    assert_eq!(element.style().gap.height, Some(px(8.0).into()));
}

#[test]
fn conditional_arrow_changes_the_border_only_when_selected() {
    const ACCENT: u32 = 0x818cf8;
    for selected in [false, true] {
        let mut row: Stateful<Div> = gds_uim::div! {
            id: "row";
            border_1;
            border_color: 0x30384b.gpui::rgb;
            @selected => { border_color: ACCENT.gpui::rgb };
        };
        let expected = if selected { ACCENT } else { 0x30384b };
        assert_eq!(row.style().border_color, Some(gpui::rgb(expected).into()));
    }
}

#[gpui::test]
fn dispatches_gpui_events_after_a_type_changing_id_method(cx: &mut TestAppContext) {
    let clicks = Rc::new(Cell::new(0));
    let mouse_downs = Rc::new(Cell::new(0));
    let record_click = clicks.clone();
    let record_mouse_down = mouse_downs.clone();

    let (_view, visual) = cx.add_window_view(move |_, _| {
        TestView(Box::new(move || {
            let record_click = record_click.clone();
            let record_mouse_down = record_mouse_down.clone();
            let element: Stateful<Div> = el! {
                #div {
                    id: "button";
                    size: 40.0.px;
                    on_mouse_down: MouseButton::Left, $move |_event, _window, _cx| {
                        record_mouse_down.set(record_mouse_down.get() + 1);
                    };
                    on_click: $move |_event, _window, _cx| {
                        record_click.set(record_click.get() + 1);
                    };
                }
            };
            element.into_any_element()
        }))
    });
    visual.simulate_click(point(px(10.0), px(10.0)), Modifiers::default());

    assert_eq!(mouse_downs.get(), 1);
    assert_eq!(clicks.get(), 1);
}

#[test]
fn accepts_hash_factories_for_svg_and_image_elements() {
    let mut vector: gpui::Svg = el! {
        #gpui::svg { path: "icons/confirm.svg"; size: 16.0.px }
    };
    let mut image: gpui::Img = el! {
        #gpui::img("images/avatar.png") { size: 32.0.px }
    };

    assert_eq!(vector.style().size.width, Some(px(16.0).into()));
    assert_eq!(image.style().size.height, Some(px(32.0).into()));
    let _: [AnyElement; 2] = [vector.into_any_element(), image.into_any_element()];
}

#[gpui::test]
fn paints_a_canvas_from_a_generic_hash_factory(cx: &mut TestAppContext) {
    let painted = Rc::new(RefCell::new(None));
    let record_paint = painted.clone();

    let (_view, _visual) = cx.add_window_view(move |_, _| {
        TestView(Box::new(move || {
            let record_paint = record_paint.clone();
            el! {
                #div { flex } [
                    #gpui::canvas::<Pixels>(
                        $|bounds, _window, _cx| bounds.size.width,
                        $move |bounds, prepaint_width, _window, _cx| {
                            *record_paint.borrow_mut() = Some((bounds, prepaint_width));
                        },
                    ) { size: 24.0.px },
                ]
            }
            .into_any_element()
        }))
    });

    let painted = painted.borrow();
    let (bounds, prepaint_width) = painted.as_ref().expect("canvas should be painted");
    assert_eq!(bounds.size, size(px(24.0), px(24.0)));
    assert_eq!(*prepaint_width, px(24.0));
}

#[gpui::test]
fn suffix_deferred_preserves_layout_and_orders_paint_by_priority(cx: &mut TestAppContext) {
    fn recorded_canvas(
        label: &'static str,
        extent: f32,
        paints: Rc<RefCell<Vec<&'static str>>>,
    ) -> impl IntoElement {
        el! {
            #gpui::canvas::<()>(
                $|_, _, _| (),
                $move |_, _, _, _| paints.borrow_mut().push(label),
            ) { size: extent.px; }
        }
    }

    let bounds: Rc<RefCell<Vec<Bounds<Pixels>>>> = Rc::new(RefCell::new(Vec::new()));
    let paints = Rc::new(RefCell::new(Vec::new()));
    let record_bounds = bounds.clone();
    let record_paints = paints.clone();
    let (_view, visual) = cx.add_window_view(move |_, _| {
        TestView(Box::new(move || {
            let record_bounds = record_bounds.clone();
            el! {
                #div {
                    flex;
                    flex_col;
                    w: 100.0.px;
                    gap: 4.0.px;
                    on_children_prepainted: $move |children, _, _| {
                        *record_bounds.borrow_mut() = children;
                    };
                } [
                    #recorded_canvas("normal-first", 8.0, $record_paints.clone()),
                    #div [
                        #recorded_canvas("deferred-high", 16.0, $record_paints.clone()),
                    ].m! { size: 16.0.px }.deferred.m! { priority: 10; },
                    #recorded_canvas("normal-second", 8.0, $record_paints.clone()),
                    #div { size: 12.0.px; } [
                        #recorded_canvas("deferred-low", 12.0, $record_paints.clone()),
                    ].gpui::deferred { priority: 1; },
                ]
            }
            .into_any_element()
        }))
    });
    visual.run_until_parked();

    let bounds = bounds.borrow();
    assert_eq!(bounds.len(), 4);
    let mut top = px(0.0);
    for (child, extent) in bounds.iter().zip([8.0, 16.0, 8.0, 12.0]) {
        assert_eq!(child.size, size(px(extent), px(extent)));
        assert_eq!(child.origin.y, top);
        top += px(extent + 4.0);
    }
    assert_eq!(
        *paints.borrow(),
        [
            "normal-first",
            "normal-second",
            "deferred-low",
            "deferred-high"
        ]
    );
}

#[test]
fn builtin_shortcuts_use_gpui_factories_and_accept_element_bodies() {
    // A local function with the same name must not change the built-in macro.
    fn div() -> u32 {
        7
    }
    use gds_uim::div as container;

    let mut element: Div = container! { flex; gap: 6.0.px; };
    assert_eq!(div(), 7);
    assert_eq!(element.style().display, Some(Display::Flex));
    assert_eq!(element.style().gap.height, Some(px(6.0).into()));
    let _: Div = gds_uim::div! {};
    let _: Div = gds_uim::div! {
        { flex; }
        ["Title", $gds_uim::div! { child: "Content"; }]
    };
    let _: gpui::Deferred = gds_uim::div! { .gpui::deferred { priority: 1; } };

    let mut vector: gpui::Svg = gds_uim::svg! {
        path: "icons/confirm.svg";
        size: 16.0.px;
    };
    let mut image: gpui::Img = gds_uim::img! {
        ("images/avatar.png") { size: 32.0.px; }
    };
    let _: gpui::Img = gds_uim::img!("images/avatar.png");
    assert_eq!(vector.style().size.width, Some(px(16.0).into()));
    assert_eq!(image.style().size.height, Some(px(32.0).into()));
}

#[test]
fn shortcuts_cover_wrappers_caches_and_virtual_lists() {
    let _: gpui::Anchored = gds_uim::anchored! {
        { anchor: gpui::Corner::TopLeft; }
        [#gpui::div]
    };
    let _: gpui::Deferred = gds_uim::deferred!(#gpui::div ["Content"]);
    let _: gpui::Deferred = gds_uim::deferred! {
        (#gpui::div ["Content"]) { priority: 2; }
    };

    let provider = gpui::retain_all("image-cache");
    let mut cache: gpui::ImageCacheElement = gds_uim::image_cache! {
        (provider) { size: 32.0.px; } [#gpui::div]
    };
    assert_eq!(cache.style().size.width, Some(px(32.0).into()));

    let state = gpui::ListState::new(0, gpui::ListAlignment::Top, px(20.0));
    let mut list: gpui::List = gds_uim::list! {
        (state, $|_, _, _| gds_uim::div! {}.into_any_element()) { size: 20.0.px; }
    };
    assert_eq!(list.style().size.height, Some(px(20.0).into()));

    let mut uniform: gpui::UniformList = gds_uim::uniform_list! {
        ("uniform-list", 0, $|range, _, _| {
            range.map(|_| gds_uim::div! { size: 24.0.px; }).collect::<Vec<_>>()
        }) { size: 24.0.px; }
    };
    assert_eq!(uniform.style().size.height, Some(px(24.0).into()));

    // SurfaceSource is platform-specific and uninhabited on Linux. Check the
    // shortcut's signature without requiring a macOS video buffer.
    fn make_surface(source: gpui::SurfaceSource) -> gpui::Surface {
        gds_uim::surface! { (source) { size: 16.0.px; } }
    }
    let _make_surface: fn(gpui::SurfaceSource) -> gpui::Surface = make_surface;
}

#[gpui::test]
fn paints_a_canvas_from_shortcut_macros(cx: &mut TestAppContext) {
    let painted = Rc::new(RefCell::new(None));
    let record_paint = painted.clone();
    let (_view, _visual) = cx.add_window_view(move |_, _| {
        TestView(Box::new(move || {
            let record_paint = record_paint.clone();
            gds_uim::div! {
                { flex; }
                [
                    $gds_uim::canvas! {
                        ::<Pixels>(
                            $|bounds, _, _| bounds.size.width,
                            $move |bounds, width, _, _| {
                                *record_paint.borrow_mut() = Some((bounds, width));
                            },
                        ) { size: 24.0.px; }
                    },
                ]
            }
            .into_any_element()
        }))
    });
    let painted = painted.borrow();
    let (bounds, width) = painted.as_ref().expect("shortcut canvas should be painted");
    assert_eq!(bounds.size, size(px(24.0), px(24.0)));
    assert_eq!(*width, px(24.0));
}
