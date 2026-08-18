pub mod backend;

pub use backend::gpui;

use crate::gpui::*;
use std::time::{Duration, Instant};

pub trait RenderMode: 'static + Sized {
    /// Should the loop be active?
    fn is_continuous(&self) -> bool { false }

    /// Optional frame rate limit (e.g., Some(Duration::from_millis(16)))
    fn min_frame_interval(&self) -> Option<Duration> { None }

    /// The math update step, provided with physical Delta Time
    fn tick(&mut self, _dt: f32, _cx: &mut Context<Self>) {}
}

pub struct PacedView<V: RenderMode + Render> {
    pub inner: Entity<V>,
    last_tick: Option<Instant>,
    frame_scheduled: bool,
}

impl<V: RenderMode + Render> PacedView<V> {
    fn on_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.frame_scheduled = false;
        
        let mut is_continuous = false;
        let mut min_interval = None;

        self.inner.update(cx, |inner, _cx| {
            is_continuous = inner.is_continuous();
            min_interval = inner.min_frame_interval();
        });

        if !is_continuous {
            self.last_tick = None;
            return;
        }

        let now = Instant::now();

        let dt = match self.last_tick {
            Some(last) => {
                let elapsed = now.duration_since(last);

                if let Some(min) = min_interval {
                    if elapsed < min {
                        self.frame_scheduled = true;
                        cx.on_next_frame(window, Self::on_frame);
                        return;
                    }
                }
                elapsed.as_secs_f32().min(0.1)
            }
            None => 0.0,
        };
        self.last_tick = Some(now);

        self.inner.update(cx, |inner, cx| {
            inner.tick(dt, cx);
            cx.notify(); // Triggers a re-render of the inner entity
        });

        // Keep the loop going!
        self.frame_scheduled = true;
        cx.on_next_frame(window, Self::on_frame);
    }
}

impl<V: RenderMode + Render> Render for PacedView<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut is_continuous = false;
        self.inner.update(cx, |inner, _cx| {
            is_continuous = inner.is_continuous();
        });

        // Boot up the VSync loop if it should be running but isn't
        if is_continuous && !self.frame_scheduled {
            self.frame_scheduled = true;
            
            cx.on_next_frame(window, Self::on_frame);
        } else if !is_continuous {
            self.last_tick = None;
        }

        // Entity<V> natively implements IntoElement if V: Render!
        self.inner.clone()
    }
}

pub trait PacedViewExt {
    fn new_paced_view<V, F>(&mut self, build_view: F) -> Entity<PacedView<V>>
    where
        V: RenderMode + Render + 'static,
        F: FnOnce(&mut Context<V>) -> V;
}
impl PacedViewExt for App {
    fn new_paced_view<V, F>(&mut self, build_view: F) -> Entity<PacedView<V>>
    where
        V: RenderMode + Render + 'static,
        F: FnOnce(&mut Context<V>) -> V,
    {
        self.new(|cx| {
            let inner = cx.new(build_view);
            PacedView {
                inner,
                last_tick: None,
                frame_scheduled: false,
            }
        })
    }
}
