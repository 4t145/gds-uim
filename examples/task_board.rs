use gds_uim::{el, m};
use gpui::{
    App, Application, Bounds, ClickEvent, Context, Div, FontWeight, IntoElement, SharedString,
    Stateful, TitlebarOptions, Window, WindowBounds, WindowOptions, div, prelude::*, px, relative,
    rgb, size,
};

const BACKGROUND: u32 = 0x10141f;
const PANEL: u32 = 0x191f2e;
const BORDER: u32 = 0x30384b;
const MUTED: u32 = 0x95a1b8;
const ACCENT: u32 = 0x818cf8;
const GREEN: u32 = 0x6ee7b7;
const AMBER: u32 = 0xfcd34d;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Filter {
    #[default]
    All,
    Open,
    Done,
}

impl Filter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "All tasks",
            Self::Open => "Open",
            Self::Done => "Completed",
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::All => "filter-all",
            Self::Open => "filter-open",
            Self::Done => "filter-done",
        }
    }
}

struct Task {
    id: usize,
    title: &'static str,
    category: &'static str,
    due: &'static str,
    description: &'static str,
    done: bool,
    priority: bool,
}

struct TaskBoard {
    tasks: Vec<Task>,
    filter: Filter,
    priority_only: bool,
    selected: Option<usize>,
}

impl Default for TaskBoard {
    fn default() -> Self {
        Self {
            tasks: vec![
                Task {
                    id: 0,
                    title: "Polish the workspace header",
                    category: "Interface",
                    due: "Today",
                    description: "Give the workspace a clear title and a short description. Keep the main action easy to find.",
                    done: false,
                    priority: true,
                },
                Task {
                    id: 1,
                    title: "Build the task detail panel",
                    category: "Interface",
                    due: "Today",
                    description: "Show the selected task's context, status and priority in one place, with room for longer descriptions.",
                    done: false,
                    priority: false,
                },
                Task {
                    id: 2,
                    title: "Review the filter interactions",
                    category: "Interaction",
                    due: "Tomorrow",
                    description: "Check that switching filters keeps a visible task selected. Handle the empty state when no tasks match.",
                    done: false,
                    priority: true,
                },
                Task {
                    id: 3,
                    title: "Define the spacing scale",
                    category: "Design system",
                    due: "Yesterday",
                    description: "Use consistent spacing between cards, controls and text so the layout feels calm and readable.",
                    done: true,
                    priority: false,
                },
                Task {
                    id: 4,
                    title: "Add hover feedback",
                    category: "Interaction",
                    due: "Yesterday",
                    description: "Give clickable controls a visible hover state while keeping the selected task easy to recognize.",
                    done: true,
                    priority: true,
                },
            ],
            filter: Filter::All,
            priority_only: false,
            selected: Some(0),
        }
    }
}

// m! also works in ordinary Rust. The element tree can apply this helper with .panel.
fn panel(element: Div) -> Div {
    element.map(m! { rounded_xl; bg: PANEL.rgb; border_1; border_color: BORDER.rgb })
}

// Components expose named builder methods; event behavior belongs to TaskBoard.
#[derive(IntoElement)]
struct Button {
    element: Stateful<Div>,
    label: SharedString,
    highlighted: bool,
}

impl Button {
    fn new(id: &'static str) -> Self {
        Self {
            element: el! { .div { # id; #? move id { into } } },
            label: "".into(),
            highlighted: false,
        }
    }

    fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    fn on_click(mut self, listener: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.element = self.element.on_click(listener);
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let Self {
            element,
            label,
            highlighted,
        } = self;
        el! {
            element {
                flex; flex_shrink_0; items_center; justify_center; h: 36.0.px; px: 14.0.px;
                rounded_lg; text_size: 13.0.px; font_weight: FontWeight::MEDIUM; cursor_pointer;
                @if (highlighted)
                    bg: 0x424c99.rgb;
                    hover: { bg: 0x525fb5.rgb };
                @else
                    bg: 0x242c3e.rgb;
                    hover: { bg: 0x343e54.rgb };
                @end;
            } [label]
        }
    }
}

#[derive(IntoElement)]
struct Badge {
    label: &'static str,
    color: u32,
}

impl Badge {
    fn with_label(label: &'static str) -> Self {
        Self {
            label,
            color: MUTED,
        }
    }

    fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let Self { label, color } = self;
        el! {
            .div {
                flex_shrink_0; px: 8.0.px; py: 4.0.px; rounded_md;
                text_size: 11.0.px; text_color: color.rgb; bg: BACKGROUND.rgb;
            } [label]
        }
    }
}

#[derive(IntoElement)]
struct StatCard {
    label: &'static str,
    value: usize,
    color: u32,
}

impl StatCard {
    fn with_label(label: &'static str) -> Self {
        Self {
            label,
            value: 0,
            color: MUTED,
        }
    }

    fn value(mut self, value: usize) -> Self {
        self.value = value;
        self
    }

    fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }
}

impl RenderOnce for StatCard {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let Self {
            label,
            value,
            color,
        } = self;
        el! {
            .div {
                flex; flex_col; flex_1; gap: 6.0.px; p: 18.0.px;
            } [
                .div { text_size: 12.0.px; text_color: MUTED.rgb } [label],
                .div { text_size: 28.0.px; font_weight: FontWeight::BOLD; text_color: color.rgb } [
                    value { to_string },
                ],
            ].panel
        }
    }
}

impl TaskBoard {
    fn matches(&self, task: &Task) -> bool {
        let status_matches = match self.filter {
            Filter::All => true,
            Filter::Open => !task.done,
            Filter::Done => task.done,
        };
        status_matches && (!self.priority_only || task.priority)
    }

    fn reconcile_selection(&mut self) {
        if !self
            .tasks
            .iter()
            .any(|task| Some(task.id) == self.selected && self.matches(task))
        {
            self.selected = self
                .tasks
                .iter()
                .find(|task| self.matches(task))
                .map(|task| task.id);
        }
    }

    fn update_selected(&mut self, update: impl FnOnce(&mut Task)) {
        if let Some(task) = self
            .tasks
            .iter_mut()
            .find(|task| Some(task.id) == self.selected)
        {
            update(task);
        }
        self.reconcile_selection();
    }

    fn on_filter(
        filter: Filter,
    ) -> impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) {
        move |this, _, _, cx| {
            this.filter = filter;
            this.reconcile_selection();
            cx.notify();
        }
    }

    fn on_select_task(
        id: usize,
    ) -> impl Fn(&mut Self, &ClickEvent, &mut Window, &mut Context<Self>) {
        move |this, _, _, cx| {
            this.selected = Some(id);
            cx.notify();
        }
    }

    fn on_reset(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        *self = Self::default();
        cx.notify();
    }

    fn on_toggle_done(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.update_selected(|task| task.done = !task.done);
        cx.notify();
    }

    fn on_toggle_priority(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.update_selected(|task| task.priority = !task.priority);
        cx.notify();
    }

    fn on_toggle_priority_only(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.priority_only = !self.priority_only;
        self.reconcile_selection();
        cx.notify();
    }

    fn on_show_all(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.filter = Filter::All;
        self.priority_only = false;
        self.reconcile_selection();
        cx.notify();
    }

    fn filter_button(&self, filter: Filter, cx: &mut Context<Self>) -> Button {
        el! {
            $(filter.id()).Button::new {
                label: $filter.label();
                highlighted: $self.filter == filter;
                on_click: (cx, filter.Self::on_filter).Context::listener;
            }
        }
    }

    fn task_row(&self, task: &Task, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = task.id;
        let selected = self.selected == Some(id);
        let background = if selected { 0x252d48 } else { PANEL };
        let hover = if selected { 0x2e3858 } else { 0x222a3c };
        let text = if task.done { MUTED } else { 0xe8edf7 };
        el! {
            .div {
                # $("task", id); #? move format!("task-{id}");
                flex; flex_shrink_0; items_center; gap: 12.0.px; p: 16.0.px; rounded_lg;
                border_1; border_color: BORDER.rgb; bg: background.rgb;
                cursor_pointer; hover: move { bg: hover.rgb };
                when: selected, { border_color: ACCENT.rgb };
                on_click: (cx, id.Self::on_select_task).Context::listener;
            } [
                .div { flex; flex_col; flex_1; min_w_0; gap: 6.0.px } [
                    .div { text_size: 14.0.px; text_color: text.rgb; truncate } [$task.title],
                    .div { text_size: 11.0.px; text_color: MUTED.rgb } [
                        format!("{}  /  {}", task.category, task.due),
                    ],
                ],
                @if (task.priority)
                    "Priority".Badge::with_label { color: AMBER },
                @end,
                @if (task.done)
                    "Done".Badge::with_label { color: GREEN },
                @else
                    "Open".Badge::with_label,
                @end,
            ]
        }
    }

    fn details(&self, cx: &mut Context<Self>) -> Div {
        // A value match binds the selected task and returns a Div from either arm.
        el! {
            @match (self.tasks.iter().find(|task| Some(task.id) == self.selected))
            @case (Some(task))
                .div { flex; flex_col; h_full; gap: 16.0.px } [
                    .div { # "detail-content"; flex_1; min_h_0; overflow_y_scroll } [
                        .div { flex; flex_col; gap: 16.0.px } [
                            .div { text_size: 11.0.px; text_color: MUTED.rgb } ["TASK DETAILS"],
                            .div { text_size: 21.0.px; font_weight: FontWeight::BOLD } [$task.title],
                            .div { flex; gap: 8.0.px } [
                                $(task.category).Badge::with_label { color: ACCENT },
                                @if (task.done)
                                    "Completed".Badge::with_label { color: GREEN },
                                @else
                                    "In progress".Badge::with_label { color: AMBER },
                                @end,
                            ],
                            .div { text_size: 14.0.px; text_color: MUTED.rgb; line_height: 22.0.px } [$task.description],
                            .div { text_size: 12.0.px; text_color: MUTED.rgb } [format!("Planned for {}", task.due.to_lowercase())],
                        ],
                    ],
                    .div { flex; flex_col; flex_shrink_0; gap: 8.0.px } [
                        "toggle-done".Button::new {
                            label: @if (task.done) "Reopen task" @else "Mark complete" @end;
                            highlighted: true;
                            on_click: (cx, Self::on_toggle_done).Context::listener;
                        },
                        "toggle-priority".Button::new {
                            label: @if (task.priority) "Remove priority" @else "Make priority" @end;
                            on_click: (cx, Self::on_toggle_priority).Context::listener;
                        },
                    ],
                ]
            @case (None)
                .div { flex; flex_col; gap: 12.0.px } [
                    .div { text_size: 18.0.px; font_weight: FontWeight::BOLD } ["Nothing selected"],
                    .div { text_size: 13.0.px; text_color: MUTED.rgb } [
                        "Choose a task from the list to see its details.",
                    ],
                ]
            @end
        }
    }
}

impl Render for TaskBoard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let completed = self.tasks.iter().filter(|task| task.done).count();
        let priority = self
            .tasks
            .iter()
            .filter(|task| task.priority && !task.done)
            .count();
        let total = self.tasks.len();
        let progress = if total == 0 {
            0.0
        } else {
            completed as f32 / total as f32
        };
        window.set_window_title(&format!("Task board — {completed}/{total} complete"));

        let rows: Vec<_> = self
            .tasks
            .iter()
            .filter(|task| self.matches(task))
            .map(|task| self.task_row(task, cx))
            .collect();
        let visible = rows.len();
        let filters = [Filter::All, Filter::Open, Filter::Done];

        el! {
            .div {
                flex; flex_col; size_full; p: 28.0.px; gap: 20.0.px;
                bg: BACKGROUND.rgb; text_color: 0xe8edf7.rgb;
            } [
                .div { flex; items_center; justify_between; flex_shrink_0 } [
                    .div { flex; flex_col; gap: 8.0.px } [
                        .div { text_size: 11.0.px; text_color: ACCENT.rgb } ["WORKSPACE / INTERFACE LAB"],
                        .div { text_size: 30.0.px; font_weight: FontWeight::BOLD } ["A little progress, every day."],
                        .div { text_size: 13.0.px; text_color: MUTED.rgb } ["Select a task, set a priority, and keep things moving."],
                    ],
                    "reset-board".Button::new {
                        label: "Reset demo";
                        on_click: (cx, Self::on_reset).Context::listener;
                    },
                ],
                .div { flex; gap: 14.0.px; flex_shrink_0 } [
                    "OPEN TASKS".StatCard::with_label { value: $total - completed; color: ACCENT },
                    "COMPLETED".StatCard::with_label { value: completed; color: GREEN },
                    "PRIORITY / OPEN".StatCard::with_label { value: priority; color: AMBER },
                ],
                .div { flex; items_center; gap: 14.0.px; flex_shrink_0 } [
                    .div { flex_1; h: 6.0.px; rounded_full; bg: BORDER.rgb; overflow_hidden } [
                        .div { h_full; w: progress.relative; rounded_full; bg: GREEN.rgb },
                    ],
                    .div { text_size: 12.0.px; text_color: MUTED.rgb } [format!("{completed} of {total} complete")],
                ],
                .div { flex; flex_1; min_h_0; gap: 20.0.px } [
                    .div { flex; flex_col; flex_1; min_w_0; gap: 14.0.px } [
                        .div { flex; items_center; justify_between; flex_shrink_0 } [
                            .div { text_size: 17.0.px; font_weight: FontWeight::BOLD } [format!("Tasks ({visible})")],
                            "priority-only".Button::new {
                                label: "Priority only";
                                highlighted: $self.priority_only;
                                on_click: (cx, Self::on_toggle_priority_only).Context::listener;
                            },
                        ],
                        .div { flex; gap: 8.0.px; flex_shrink_0 } [
                            ..filters { into_iter; map: filter => (self, filter, cx).Self::filter_button },
                        ],
                        .div { # "task-list"; flex; flex_col; flex_1; min_h_0; gap: 8.0.px; overflow_y_scroll } [
                            @if (visible == 0)
                                .div {
                                    #? "empty-state".String::from;
                                    flex; flex_col; items_center; gap: 14.0.px; py: 36.0.px;
                                } [
                                    .div { text_size: 16.0.px } ["No tasks in this view"],
                                    .div { text_size: 12.0.px; text_color: MUTED.rgb } ["Try another filter to see more tasks."],
                                    "show-all".Button::new {
                                        label: "Show all tasks";
                                        highlighted: true;
                                        on_click: (cx, Self::on_show_all).Context::listener;
                                    },
                                ],
                            @else
                                ..rows,
                            @end,
                        ],
                    ],
                    .div.panel {
                        # "details"; #? "details-panel" { into };
                        w: 290.0.px; flex_shrink_0; p: 22.0.px; overflow_hidden;
                    } [(self, cx).Self::details],
                ],
                .div { text_size: 11.0.px; text_color: MUTED.rgb; flex_shrink_0 } [
                    "Demo workspace · Changes stay in memory. Reset demo restores the starting tasks.",
                ],
            ]
        }
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1080.0), px(820.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(960.0), px(700.0))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Task board".into()),
                    ..Default::default()
                }),
                app_id: Some("gds_uim.task-board".into()),
                ..Default::default()
            },
            |_, cx| cx.new(|_| TaskBoard::default()),
        )
        .expect("failed to open the task board window");
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, TestAppContext, VisualTestContext};

    fn click(visual: &mut VisualTestContext, selector: &'static str) {
        let bounds = visual
            .debug_bounds(selector)
            .expect("control should be visible");
        visual.simulate_click(bounds.center(), Modifiers::default());
        visual.run_until_parked();
    }

    fn visible_tasks(board: &TaskBoard) -> Vec<usize> {
        board
            .tasks
            .iter()
            .filter(|task| board.matches(task))
            .map(|task| task.id)
            .collect()
    }

    #[gpui::test]
    fn filters_completes_reopens_and_resets_tasks(cx: &mut TestAppContext) {
        let (view, visual) = cx.add_window_view(|_, _| TaskBoard::default());
        visual.simulate_resize(size(px(1080.0), px(820.0)));
        visual.run_until_parked();
        assert_eq!(
            visual.window_title().as_deref(),
            Some("Task board — 2/5 complete")
        );

        click(visual, "filter-open");
        assert_eq!(
            view.read_with(visual, |board, _| visible_tasks(board)),
            [0, 1, 2]
        );
        click(visual, "task-2");
        assert_eq!(view.read_with(visual, |board, _| board.selected), Some(2));
        click(visual, "toggle-done");
        assert!(view.read_with(visual, |board, _| board.tasks[2].done));
        assert_eq!(
            view.read_with(visual, |board, _| visible_tasks(board)),
            [0, 1]
        );
        assert_eq!(view.read_with(visual, |board, _| board.selected), Some(0));
        assert_eq!(
            visual.window_title().as_deref(),
            Some("Task board — 3/5 complete")
        );

        click(visual, "filter-done");
        click(visual, "task-2");
        click(visual, "toggle-done");
        assert!(!view.read_with(visual, |board, _| board.tasks[2].done));
        assert_eq!(
            view.read_with(visual, |board, _| visible_tasks(board)),
            [3, 4]
        );

        click(visual, "reset-board");
        assert!(
            view.read_with(visual, |board, _| board.filter == Filter::All
                && !board.priority_only)
        );
        assert_eq!(view.read_with(visual, |board, _| board.selected), Some(0));
        assert_eq!(
            visual.window_title().as_deref(),
            Some("Task board — 2/5 complete")
        );
        assert!(visual.debug_bounds("task-2").is_some());
    }

    #[gpui::test]
    fn priority_filter_handles_empty_results_and_restores_the_list(cx: &mut TestAppContext) {
        let (view, visual) = cx.add_window_view(|_, _| TaskBoard::default());
        visual.simulate_resize(size(px(960.0), px(700.0)));
        visual.run_until_parked();
        click(visual, "priority-only");
        click(visual, "filter-done");
        assert_eq!(view.read_with(visual, |board, _| board.selected), Some(4));
        click(visual, "toggle-priority");
        assert_eq!(view.read_with(visual, |board, _| board.selected), None);
        assert!(visual.debug_bounds("empty-state").is_some());
        assert!(view.read_with(visual, |board, _| visible_tasks(board).is_empty()));

        click(visual, "show-all");
        assert_eq!(view.read_with(visual, |board, _| board.selected), Some(0));
        assert_eq!(
            view.read_with(visual, |board, _| visible_tasks(board)),
            [0, 1, 2, 3, 4]
        );
        click(visual, "filter-done");
        click(visual, "task-4");
        click(visual, "toggle-priority");
        assert!(view.read_with(visual, |board, _| board.tasks[4].priority));

        click(visual, "filter-all");
        let row = visual.debug_bounds("task-0").unwrap();
        let panel = visual.debug_bounds("details-panel").unwrap();
        assert!(row.size.width > px(300.0));
        assert!(row.right() <= panel.left());
        assert!(panel.right() <= px(960.0));
        let action = visual.debug_bounds("toggle-priority").unwrap();
        assert!(action.bottom() <= panel.bottom());
    }
}
