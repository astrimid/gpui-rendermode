# gpui-rendermode

**Declarative frame scheduling and VSync-aligned animation loops for Zed's GPUI framework.**

Building continuous animations (progress bars, spinners, or video) in GPUI requires ...

`gpui-rendermode` abstracts the boilerplate away. It provides a simple `RenderMode` trait that gives your components a clean `tick(dt)` method, automatically handling delta-time math, frame scheduling, and CPU parking when animations are paused.

## Features

* **⏱️ Delta Time (`dt`) provided automatically:** Write frame-rate independent animations.
* **⏸️ Auto-parking:** Return `false` from `is_continuous()` and your loop goes to sleep at 0% CPU. To wake up, just change that boolean and notify the Entity/View from the input event handler to invalidate the view and cause animation to start running on next frame.
* **🐢 Throttling:** Optional `min_frame_interval` for locking frame rates (e.g., 30 FPS limits).
* **🧩 Native GPUI Integration:** Completely transparent to GPUI. `PacedView<T>` implements `Render` and generates a standard `View<T>`.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gpui = { package = "gpui-unofficial", version = "^1.14.2" }
gpui-rendermode = { git = "https://github.com/astrimid/gpui-rendermode" }
```

## Quick Start

Here is how to create a progress bar that smoothly fills up only while the user is hovering over it.

### 1. Implement `RenderMode`

Define your state and implement `RenderMode`. You just need to tell the engine *when* to run, and *what* the math is.

```rust
use gpui::*;
use gpui_rendermode::RenderMode;

struct HoverButton {
    progress: f32,
    is_hovered: bool,
}

impl RenderMode for HoverButton {
    // 1. Tell the engine when the animation loop should be active
    fn is_continuous(&self) -> bool {
        self.is_hovered
    }

    // 2. Do your math: `dt` is the physical time in seconds
    // since the last animation tick.
    fn tick(&mut self, dt: Duration, _cx: &mut Context<Self>) {
        if self.is_hovered {
            self.progress += dt * 1.5; // Fill up over ~0.66 seconds
            
            if self.progress >= 1.0 {
                self.progress = 0.0; // Loop back to zero
            }
        }
    }
}

```

### 2. Implement standard `Render`

Implement GPUI's standard `Render` trait exactly as you normally would. Use `cx.notify()` when external interactions start and stop animation.

```rust
impl Render for HoverButton {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(200.0))
            .h(px(40.0))
            .bg(rgb(0x333333))
            // The unified GPUI 2 hover event
            .on_hover(cx.listener(|this, is_hovered: &bool, _window, cx| {
                this.is_hovered = *is_hovered;
                if !this.is_hovered {
                    this.progress = 0.0;
                }
                cx.notify(); // Wake up the engine!
            }))
            .child(
                // The animated fill
                div()
                    .h_full()
                    .w(relative(self.progress))
                    .bg(rgb(0x3b82f6))
            )
    }
}

```

### 3. Instantiate with `new_paced_view`

To render your animated component, bring `PacedViewExt` into scope and construct it using `cx.new_paced_view()` instead of the standard `cx.new_view()`.

```rust
use gpui_rendermode::PacedViewExt;

// Inside any parent component's render method, or window initialization:
cx.new_paced_view(|_cx| HoverButton {
    progress: 0.0,
    is_hovered: false,
})

```

## How it works under the hood

When `is_continuous` returns `true`, the `PacedView` wrapper automatically calculates physical Delta Time, executes your `tick(dt)` method, triggers a `cx.notify()` to dirty the element, and schedules itself for the next frame using `cx.on_next_frame(window, ...)`.

When `is_continuous` returns `false`, it drops out of the queue, instantly reducing overhead to zero until state is changed again.

