//! Coordinate spaces.
//!
//! Three are in play, and mixing them silently is the classic screen-bot failure.
//! Each gets its own type, and every conversion lives here.
//!
//! ```text
//!   NormPoint  --to_window(size)-->  WindowPoint  --to_screen(origin)-->  ScreenPoint
//!   (config)                         (index a Frame)                     (enigo)
//! ```

use serde::{Deserialize, Serialize};

/// Normalized to the game window: `0.0..=1.0` on both axes.
///
/// The only form that belongs in config files — normalized coordinates survive a
/// resolution change or a moved window.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormPoint {
    pub x: f32,
    pub y: f32,
}

/// Pixels relative to the game window's top-left corner — what you index into a
/// captured [`crate::capture::Frame`] with.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WindowPoint {
    pub x: u32,
    pub y: u32,
}

/// Absolute desktop pixels — the only form `enigo` accepts.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

impl NormPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Clamped, so a config typo of `1.2` lands on the window edge rather than
    /// panicking on an out-of-bounds crop.
    pub fn to_window(self, size: (u32, u32)) -> WindowPoint {
        let (w, h) = size;
        WindowPoint {
            x: scale_clamped(self.x, w),
            y: scale_clamped(self.y, h),
        }
    }
}

impl WindowPoint {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Offset by the window's desktop origin, as of the frame this came from.
    pub fn to_screen(self, origin: (i32, i32)) -> ScreenPoint {
        ScreenPoint {
            x: origin.0 + self.x as i32,
            y: origin.1 + self.y as i32,
        }
    }
}

/// How HUD regions are stored in `config/screen.toml`.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NormRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Window pixels, ready to hand to `image::imageops::crop_imm`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WindowRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl NormRect {
    pub fn to_window(self, size: (u32, u32)) -> WindowRect {
        let (sw, sh) = size;
        let x = scale_clamped(self.x, sw);
        let y = scale_clamped(self.y, sh);
        // Clamp the extent to what's left of the frame, so an oversized region
        // degrades to a smaller crop instead of panicking.
        let w = scale_clamped(self.w, sw).min(sw.saturating_sub(x)).max(1);
        let h = scale_clamped(self.h, sh).min(sh.saturating_sub(y)).max(1);
        WindowRect { x, y, w, h }
    }

    pub fn center(self) -> NormPoint {
        NormPoint::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

/// Where the game window is, and how big.
///
/// Passed explicitly rather than stashed on the actuator, so a click can never
/// resolve against stale geometry: the viewport always comes from the frame the
/// decision was made on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Viewport {
    pub origin: (i32, i32),
    pub size: (u32, u32),
}

impl Viewport {
    pub const fn new(origin: (i32, i32), size: (u32, u32)) -> Self {
        Self { origin, size }
    }

    /// The one conversion path from config coordinates to desktop pixels.
    pub fn resolve(&self, at: NormPoint) -> ScreenPoint {
        at.to_window(self.size).to_screen(self.origin)
    }
}

fn scale_clamped(v: f32, extent: u32) -> u32 {
    if !v.is_finite() || v <= 0.0 {
        return 0;
    }
    let scaled = (v * extent as f32).round();
    if scaled >= extent as f32 {
        extent
    } else {
        scaled as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: (u32, u32) = (1920, 1200);

    #[test]
    fn norm_to_window_scales_both_axes() {
        let p = NormPoint::new(0.5, 0.25).to_window(SIZE);
        assert_eq!(p, WindowPoint::new(960, 300));
    }

    #[test]
    fn window_to_screen_offsets_by_origin() {
        let p = WindowPoint::new(100, 50).to_screen((1920, -200));
        assert_eq!(p, ScreenPoint { x: 2020, y: -150 });
    }

    #[test]
    fn placement_survives_a_moved_window() {
        let at = NormPoint::new(0.42, 0.55);
        let a = at.to_window(SIZE).to_screen((0, 0));
        let b = at.to_window(SIZE).to_screen((640, 480));
        assert_eq!(b.x - a.x, 640);
        assert_eq!(b.y - a.y, 480);
    }

    #[test]
    fn placement_survives_a_resized_window() {
        let at = NormPoint::new(0.5, 0.5);
        assert_eq!(at.to_window((1920, 1200)), WindowPoint::new(960, 600));
        assert_eq!(at.to_window((1280, 720)), WindowPoint::new(640, 360));
    }

    #[test]
    fn out_of_range_norm_values_clamp_instead_of_panicking() {
        assert_eq!(NormPoint::new(-0.5, 2.0).to_window(SIZE), WindowPoint::new(0, 1200));
        assert_eq!(NormPoint::new(f32::NAN, 0.5).to_window(SIZE), WindowPoint::new(0, 600));
    }

    #[test]
    fn rect_converts_to_a_croppable_window_rect() {
        let r = NormRect { x: 0.46, y: 0.02, w: 0.08, h: 0.04 };
        let wr = r.to_window(SIZE);
        assert_eq!(wr, WindowRect { x: 883, y: 24, w: 154, h: 48 });
    }

    /// `crop_imm` panics on a rect that escapes the frame.
    #[test]
    fn oversized_rect_is_clipped_to_the_frame() {
        let r = NormRect { x: 0.9, y: 0.9, w: 0.5, h: 0.5 };
        let wr = r.to_window(SIZE);
        assert!(wr.x + wr.w <= SIZE.0, "{wr:?} escapes width");
        assert!(wr.y + wr.h <= SIZE.1, "{wr:?} escapes height");
    }

    #[test]
    fn rect_center_is_the_midpoint() {
        let r = NormRect { x: 0.2, y: 0.4, w: 0.2, h: 0.2 };
        assert_eq!(r.center(), NormPoint::new(0.3, 0.5));
    }
}
