# gpui-rendermode

**Painless animations for GPUI** — automatic frame scheduling with 0% CPU when idle.

## The Animation Pipeline

GPUI animations follow 4 layers:

1. **Clocking** — when and how often progress advances
2. **Control** — start, stop, reset, pause, reverse, repeat
3. **Interpolation** — progress-to-value mapping
4. **Rendering** — converting the current value into GPUI elements

`gpui-rendermode` handles **Clocking** for you, so you can focus on the rest.

## The Problem

Without this crate, Clocking means manually handling:
- Delta time calculations
- Frame scheduling (`request_animation_frame`)
- CPU management (preventing wasted cycles)

## The Solution

Implement `RenderMode` for your component. You get:
- ✅ Automatic `dt` (delta time) in seconds — **Clocking** done
- ✅ Auto-parking — loop sleeps when not needed
- ✅ Optional frame rate limiting
- ✅ GPUI integration — just use `PacedView`

## Quick Example

A progress bar that fills on hover and smoothly drains when you leave:

```rust
use gpui::*;
use gpui_rendermode::RenderMode;

struct HoverProgress {
    progress: f32,
    is_hovered: bool,
}

impl RenderMode for HoverProgress {
    // Control: Run while hovered OR while progress is draining back to 0
    fn is_continuous(&self) -> bool {
        self.is_hovered || self.progress > 0.0
    }

    // Clocking: `dt` = seconds since last frame (frame-rate independent)
    fn tick(&mut self, dt: f32, _cx: &mut Context<Self>) {
        // Control + Interpolation combined
        const SPEED: f32 = 1.5;
        let direction = if self.is_hovered { 1.0 } else { -1.0 };
        self.progress = (self.progress + dt * SPEED * direction).clamp(0.0, 1.0);
    }
}

impl Render for HoverProgress {
    // Rendering: Convert progress to GPUI elements
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(200.0))
            .h(px(40.0))
            .bg(rgb(0x1a1a1a))
            .on_hover(cx.listener(|this, hovered: &bool, _window, cx| {
                this.is_hovered = *hovered;
                cx.notify(); // Wake up the animation
            }))
            .child(
                div()
                    .w(relative(self.progress)) // Interpolation: progress → width
                    .h_full()
                    .bg(rgb(0x3b82f6))
            )
    }
}
```

## Usage

```rust
use gpui_rendermode::PacedViewExt;

// Instead of cx.new_view(), use:
cx.new_paced_view(|_cx| HoverProgress {
    progress: 0.0,
    is_hovered: false,
})
```

## How It Works

The `PacedView` wrapper manages the frame loop for you:

| Your Component | `PacedView` wrapper |
|----------------|---------------------|
| `is_continuous()` → true | Schedules next frame via `cx.on_next_frame()` |
| `tick(dt)` | Gets real delta time (in seconds) |
| `is_continuous()` → false | Stops scheduling — **0% CPU usage** |
| `cx.notify()` | Wakes up the loop from any event handler |

### Key Pattern: Smart Continuous Detection

Notice the `is_continuous()` logic:
```rust
fn is_continuous(&self) -> bool {
    self.is_hovered || self.progress > 0.0
}
```

This ensures:
- Animation runs **while hovered** (filling up)
- Animation runs **after hover ends** (draining back to 0)
- Animation stops **when completely drained** (0% CPU)

## Advanced Features

### Frame Rate Limiting (Clocking)

```rust
impl RenderMode for MyAnimation {
    fn min_frame_interval(&self) -> Option<Duration> {
        Some(Duration::from_millis(33)) // 30 FPS max
    }
}
```

### Full Control Layer

```rust
struct FullControl {
    progress: f32,
    state: PlayState,
    direction: f32, // 1.0 or -1.0
}

enum PlayState { Idle, Playing, Paused, Reversing }

impl RenderMode for FullControl {
    fn is_continuous(&self) -> bool {
        matches!(self.state, PlayState::Playing | PlayState::Reversing)
    }
    
    fn tick(&mut self, dt: f32, _cx: &mut Context<Self>) {
        const SPEED: f32 = 1.5;
        match self.state {
            PlayState::Playing => {
                self.progress = (self.progress + dt * SPEED * self.direction).clamp(0.0, 1.0);
                if self.progress >= 1.0 { self.state = PlayState::Idle; } // Stop
            }
            PlayState::Reversing => {
                self.progress = (self.progress - dt * SPEED).max(0.0);
                if self.progress <= 0.0 { self.state = PlayState::Idle; }
            }
            _ => {}
        }
    }
}
```

### Multi-Stage Animation

```rust
enum Stage { Idle, FadingIn, Active, FadingOut }

impl RenderMode for MyComponent {
    fn tick(&mut self, dt: f32, cx: &mut Context<Self>) {
        const SPEED: f32 = 2.0;
        
        match self.stage {
            Stage::FadingIn => {
                self.progress = (self.progress + dt * SPEED).min(1.0);
                if self.progress >= 1.0 {
                    self.stage = Stage::Active;
                }
            }
            Stage::FadingOut => {
                self.progress = (self.progress - dt * SPEED).max(0.0);
                if self.progress <= 0.0 {
                    self.stage = Stage::Idle;
                }
            }
            _ => {}
        }
    }
    
    fn is_continuous(&self) -> bool {
        matches!(self.stage, Stage::FadingIn | Stage::FadingOut)
    }
}
```

## Installation

```toml
[dependencies]
gpui = { package = "gpui-unofficial", version = "^1.14.2" }
gpui-rendermode = { git = "https://github.com/astrimid/gpui-rendermode" }
```

## License

MIT
