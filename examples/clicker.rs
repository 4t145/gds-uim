use gpui::{
    App, Application, Bounds, Context, FontWeight, IntoElement, TitlebarOptions, Window,
    WindowBounds, WindowOptions, prelude::*, px, rgb, size,
};
use uim::{div, el};

#[derive(Default)]
struct Clicker {
    count: u64,
}

impl Render for Clicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title(&format!("Clicker — {}", self.count));

        div! {
            {
                flex;
                size_full;
                items_center;
                justify_center;
                p: 24.0.px;
                bg: 0x0f172a.rgb;
                text_color: 0xe2e8f0.rgb;
            } [
                $div! {
                    {
                        flex;
                        flex_col;
                        items_center;
                        w: 340.0.px;
                        p: 32.0.px;
                        gap: 16.0.px;
                        bg: 0x1e293b.rgb;
                        border_1;
                        border_color: 0x334155.rgb;
                        rounded_xl;
                        shadow_lg;
                    } [
                        $div! {
                            text_size: 14.0.px;
                            text_color: 0x94a3b8.rgb;
                            child: "CLICKS";
                        },
                        $div! {
                            text_size: 72.0.px;
                            font_weight: FontWeight::BOLD;
                            child: $format!("{}", self.count);
                        },
                        $div! {
                            text_size: 13.0.px;
                            text_color: 0x94a3b8.rgb;
                            child: "Click the button to increase the count.";
                        },
                        $el! {
                            #gpui::div { flex; gap: 12.0.px; } [
                                #gpui::div {
                                    id: "increment";
                                    debug_selector: $|| "increment".to_owned();
                                    flex;
                                    items_center;
                                    justify_center;
                                    w: 184.0.px;
                                    h: 48.0.px;
                                    rounded_lg;
                                    bg: 0x6366f1.rgb;
                                    text_color: 0xffffff.rgb;
                                    cursor_pointer;
                                    hover: $|style| style.bg(rgb(0x818cf8));
                                    on_click: $cx.listener(|this, _, _, cx| {
                                        this.count += 1;
                                        cx.notify();
                                    });
                                } ["Click +1"],
                                #gpui::div {
                                    id: "reset";
                                    debug_selector: $|| "reset".to_owned();
                                    flex;
                                    items_center;
                                    justify_center;
                                    w: 80.0.px;
                                    h: 48.0.px;
                                    rounded_lg;
                                    bg: 0x334155.rgb;
                                    cursor_pointer;
                                    hover: $|style| style.bg(rgb(0x475569));
                                    on_click: $cx.listener(|this, _, _, cx| {
                                        this.count = 0;
                                        cx.notify();
                                    });
                                } ["Reset"],
                            ]
                        },
                    ]
                },
            ]
        }
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(480.0), px(420.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(400.0), px(380.0))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Clicker".into()),
                    ..Default::default()
                }),
                app_id: Some("uim.clicker".into()),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Clicker::default()),
        )
        .expect("failed to open the clicker window");
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
    use gpui::{Modifiers, TestAppContext};

    #[gpui::test]
    fn clicking_increments_and_reset_repaints_the_example(cx: &mut TestAppContext) {
        let (view, visual) = cx.add_window_view(|_, _| Clicker::default());
        assert_eq!(view.read_with(visual, |view, _| view.count), 0);
        assert_eq!(visual.window_title().as_deref(), Some("Clicker — 0"));

        for expected in [1, 2] {
            let button = visual
                .debug_bounds("increment")
                .expect("increment button should be laid out");
            visual.simulate_click(button.center(), Modifiers::default());
            visual.run_until_parked();
            assert_eq!(view.read_with(visual, |view, _| view.count), expected);
            assert_eq!(visual.window_title(), Some(format!("Clicker — {expected}")));
        }

        let reset = visual
            .debug_bounds("reset")
            .expect("reset button should be laid out");
        visual.simulate_click(reset.center(), Modifiers::default());
        visual.run_until_parked();
        assert_eq!(view.read_with(visual, |view, _| view.count), 0);
        assert_eq!(visual.window_title().as_deref(), Some("Clicker — 0"));
    }
}
