use gpui_platform;
use gpui_rendermode::gpui::*;
use gpui_rendermode::gpui::Styled;
use gpui_rendermode::{RenderMode, PacedViewExt, PacedView};
use std::env;

struct HoverLoop {
    id: usize,
    progress: f32,
    is_hovered: bool,
}

impl HoverLoop {
    fn set_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        if self.is_hovered != hovered {
            self.is_hovered = hovered;
            
            if !hovered {
                self.progress = 0.0;
            }
            
            cx.notify();
        }
    }
}

impl RenderMode for HoverLoop {
    fn is_continuous(&self) -> bool {
        self.is_hovered
    }

    fn tick(&mut self, dt: f32, _cx: &mut Context<Self>) {
        if self.is_hovered {
            self.progress += dt * 1.5; 
            
            if self.progress >= 1.0 {
                self.progress = 0.0; 
            }
        }
    }
}

impl Render for HoverLoop {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x1e293b))
            .p_4()
            .rounded_lg()
            .justify_center()
            .items_center()
            .gap_6()
            .child(
                div()
                    .id(self.id) 
                    .flex()
                    .justify_center()
                    .items_center()
                    .w(px(150.0))
                    .h(px(40.0))
                    .bg(if self.is_hovered { rgb(0x4b5563) } else { rgb(0x333333) })
                    .border_1()
                    .border_color(rgb(0x6b7280))
                    .cursor_pointer()
                    .on_hover(cx.listener(|this, is_hovered: &bool, _window, cx| {
                        this.set_hovered(*is_hovered, cx);
                    }))
                    .child(
                        div().text_color(rgb(0xffffff)).child(format!("Button {}", self.id))
                    )
            )
            .child(
                div()
                    .w(px(200.0))
                    .h(px(16.0))
                    .bg(rgb(0x0f172a))
                    .border_1()
                    .border_color(rgb(0x333333))
                    .child(
                        div()
                            .h_full()
                            .w(relative(self.progress))
                            .bg(rgb(0x3b82f6))
                    )
            )
    }
}

struct GridApp {
    children: Vec<Entity<PacedView<HoverLoop>>>,
}

impl Render for GridApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("grid")
            .size_full()
            .bg(rgb(0x0f172a))
            .overflow_y_scroll()  
            .flex()
            .flex_wrap()
            .justify_center()
            .content_start() 
            .gap_4()
            .p_8()
            .children(self.children.iter().cloned())
    }
}

fn main() {
    // 1. Parse the first CLI argument as a usize, default to 20
    let num_buttons = env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(20);

    println!("Rendering {} buttons...", num_buttons);

    // 2. Add `move` so the closure takes ownership of `num_buttons`
    gpui_platform::application().run(move |cx: &mut App| {
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::new(px(200.0), px(200.0)),
                size: size(px(1000.0), px(800.0)), 
            })),
            ..Default::default()
        };

        // Add `move` here as well
        cx.open_window(options, move |_window, cx| {
            cx.new(|cx| {
                let mut children = Vec::new();
                
                // 3. Use our dynamic CLI variable
                for i in 0..num_buttons {
                    children.push(cx.new_paced_view(|_cx| HoverLoop {
                        id: i,
                        progress: 0.0,
                        is_hovered: false,
                    }));
                }
                
                GridApp { children }
            })
        }).unwrap();
    });
}
