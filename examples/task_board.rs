use gds_uim::el;
use gpui::{
    App, Application, Bounds, Context, Div, FontWeight, IntoElement, SharedString, Stateful,
    TitlebarOptions, Window, WindowBounds, WindowOptions, div, prelude::*, px, relative, rgb, size,
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

// Small view helpers keep the element tree readable without a separate style DSL.
fn button(id: &'static str, label: impl Into<SharedString>, highlighted: bool) -> Stateful<Div> {
    let label = label.into();
    let background = if highlighted { 0x424c99 } else { 0x242c3e };
    let hover = if highlighted { 0x525fb5 } else { 0x343e54 };
    el! {
        #div {
            id: id; debug_selector: $move || id.to_owned();
            flex; flex_shrink_0; items_center; justify_center; h: 36.0.px; px: 14.0.px;
            rounded_lg; text_size: 13.0.px; font_weight: FontWeight::MEDIUM;
            bg: background.rgb; cursor_pointer; hover => { bg: hover.rgb };
        } [$label]
    }
}

fn badge(label: &'static str, color: u32) -> Div {
    el! {
        #div {
            flex_shrink_0; px: 8.0.px; py: 4.0.px; rounded_md;
            text_size: 11.0.px; text_color: color.rgb; bg: BACKGROUND.rgb;
        } [$label]
    }
}

fn stat_card(label: &'static str, value: usize, color: u32) -> Div {
    el! {
        #div {
            flex; flex_col; flex_1; gap: 6.0.px; p: 18.0.px;
            rounded_xl; bg: PANEL.rgb; border_1; border_color: BORDER.rgb;
        } [
            #div { text_size: 12.0.px; text_color: MUTED.rgb } [$label],
            #div { text_size: 28.0.px; font_weight: FontWeight::BOLD; text_color: color.rgb } [
                $value.to_string(),
            ],
        ]
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

    fn show_all(&mut self) {
        self.filter = Filter::All;
        self.priority_only = false;
        self.reconcile_selection();
    }

    fn filter_button(&self, filter: Filter, cx: &mut Context<Self>) -> Stateful<Div> {
        button(filter.id(), filter.label(), self.filter == filter).on_click(cx.listener(
            move |this, _, _, cx| {
                this.filter = filter;
                this.reconcile_selection();
                cx.notify();
            },
        ))
    }

    fn task_row(&self, task: &Task, cx: &mut Context<Self>) -> Stateful<Div> {
        let id = task.id;
        let selected = self.selected == Some(id);
        let background = if selected { 0x252d48 } else { PANEL };
        let hover = if selected { 0x2e3858 } else { 0x222a3c };
        let text = if task.done { MUTED } else { 0xe8edf7 };
        let priority = task.priority.then(|| badge("Priority", AMBER));
        el! {
            #div {
                id: $("task", id); debug_selector: $move || format!("task-{id}");
                flex; flex_shrink_0; items_center; gap: 12.0.px; p: 16.0.px; rounded_lg;
                border_1; border_color: BORDER.rgb; bg: background.rgb;
                cursor_pointer; hover => { bg: hover.rgb };
                @selected => { border_color: ACCENT.rgb };
                on_click: $cx.listener(move |this, _, _, cx| {
                    this.selected = Some(id);
                    cx.notify();
                });
            } [
                #div { flex; flex_col; flex_1; min_w_0; gap: 6.0.px } [
                    #div { text_size: 14.0.px; text_color: text.rgb; truncate } [$task.title],
                    #div { text_size: 11.0.px; text_color: MUTED.rgb } [
                        $format!("{}  /  {}", task.category, task.due),
                    ],
                ],
                ..priority,
                $badge(if task.done { "Done" } else { "Open" }, if task.done { GREEN } else { MUTED }),
            ]
        }
    }

    fn details(&self, cx: &mut Context<Self>) -> Div {
        let Some(task) = self
            .tasks
            .iter()
            .find(|task| Some(task.id) == self.selected)
        else {
            return el! {
                #div { flex; flex_col; gap: 12.0.px } [
                    #div { text_size: 18.0.px; font_weight: FontWeight::BOLD } ["Nothing selected"],
                    #div { text_size: 13.0.px; text_color: MUTED.rgb } [
                        "Choose a task from the list to see its details.",
                    ],
                ]
            };
        };
        let toggle_done = button(
            "toggle-done",
            if task.done {
                "Reopen task"
            } else {
                "Mark complete"
            },
            true,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.update_selected(|task| task.done = !task.done);
            cx.notify();
        }));
        let toggle_priority = button(
            "toggle-priority",
            if task.priority {
                "Remove priority"
            } else {
                "Make priority"
            },
            false,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            this.update_selected(|task| task.priority = !task.priority);
            cx.notify();
        }));
        el! {
            #div { flex; flex_col; h_full; gap: 16.0.px } [
                #div { id: "detail-content"; flex_1; min_h_0; overflow_y_scroll } [
                    #div { flex; flex_col; gap: 16.0.px } [
                        #div { text_size: 11.0.px; text_color: MUTED.rgb } ["TASK DETAILS"],
                        #div { text_size: 21.0.px; font_weight: FontWeight::BOLD } [$task.title],
                        #div { flex; gap: 8.0.px } [
                            $badge(task.category, ACCENT),
                            $badge(if task.done { "Completed" } else { "In progress" }, if task.done { GREEN } else { AMBER }),
                        ],
                        #div { text_size: 14.0.px; text_color: MUTED.rgb; line_height: 22.0.px } [$task.description],
                        #div { text_size: 12.0.px; text_color: MUTED.rgb } [$format!("Planned for {}", task.due.to_lowercase())],
                    ],
                ],
                #div { flex; flex_col; flex_shrink_0; gap: 8.0.px } [
                    $toggle_done,
                    $toggle_priority,
                ],
            ]
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
        let filters: Vec<_> = [Filter::All, Filter::Open, Filter::Done]
            .into_iter()
            .map(|filter| self.filter_button(filter, cx))
            .collect();
        let details = self.details(cx);
        let reset =
            button("reset-board", "Reset demo", false).on_click(cx.listener(|this, _, _, cx| {
                *this = Self::default();
                cx.notify();
            }));
        let priority_filter = button("priority-only", "Priority only", self.priority_only)
            .on_click(cx.listener(|this, _, _, cx| {
                this.priority_only = !this.priority_only;
                this.reconcile_selection();
                cx.notify();
            }));
        let empty = (visible == 0).then(|| {
            let show_all = button("show-all", "Show all tasks", true).on_click(cx.listener(|this, _, _, cx| {
                this.show_all();
                cx.notify();
            }));
            el! {
                #div {
                    debug_selector: $|| "empty-state".to_owned();
                    flex; flex_col; items_center; gap: 14.0.px; py: 36.0.px;
                } [
                    #div { text_size: 16.0.px } ["No tasks in this view"],
                    #div { text_size: 12.0.px; text_color: MUTED.rgb } ["Try another filter to see more tasks."],
                    $show_all,
                ]
            }
        });

        el! {
            #div {
                flex; flex_col; size_full; p: 28.0.px; gap: 20.0.px;
                bg: BACKGROUND.rgb; text_color: 0xe8edf7.rgb;
            } [
                #div { flex; items_center; justify_between; flex_shrink_0 } [
                    #div { flex; flex_col; gap: 8.0.px } [
                        #div { text_size: 11.0.px; text_color: ACCENT.rgb } ["WORKSPACE / INTERFACE LAB"],
                        #div { text_size: 30.0.px; font_weight: FontWeight::BOLD } ["A little progress, every day."],
                        #div { text_size: 13.0.px; text_color: MUTED.rgb } ["Select a task, set a priority, and keep things moving."],
                    ],
                    $reset,
                ],
                #div { flex; gap: 14.0.px; flex_shrink_0 } [
                    $stat_card("OPEN TASKS", total - completed, ACCENT),
                    $stat_card("COMPLETED", completed, GREEN),
                    $stat_card("PRIORITY / OPEN", priority, AMBER),
                ],
                #div { flex; items_center; gap: 14.0.px; flex_shrink_0 } [
                    #div { flex_1; h: 6.0.px; rounded_full; bg: BORDER.rgb; overflow_hidden } [
                        #div { h_full; w: progress.relative; rounded_full; bg: GREEN.rgb },
                    ],
                    #div { text_size: 12.0.px; text_color: MUTED.rgb } [$format!("{completed} of {total} complete")],
                ],
                #div { flex; flex_1; min_h_0; gap: 20.0.px } [
                    #div { flex; flex_col; flex_1; min_w_0; gap: 14.0.px } [
                        #div { flex; items_center; justify_between; flex_shrink_0 } [
                            #div { text_size: 17.0.px; font_weight: FontWeight::BOLD } [$format!("Tasks ({visible})")],
                            $priority_filter,
                        ],
                        #div { flex; gap: 8.0.px; flex_shrink_0 } [..filters],
                        #div { id: "task-list"; flex; flex_col; flex_1; min_h_0; gap: 8.0.px; overflow_y_scroll } [
                            ..rows,
                            ..empty,
                        ],
                    ],
                    #div {
                        id: "details"; debug_selector: $|| "details-panel".to_owned();
                        w: 290.0.px; flex_shrink_0; p: 22.0.px; rounded_xl;
                        bg: PANEL.rgb; border_1; border_color: BORDER.rgb; overflow_hidden;
                    } [$details],
                ],
                #div { text_size: 11.0.px; text_color: MUTED.rgb; flex_shrink_0 } [
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
