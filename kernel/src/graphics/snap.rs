//! NovaOS Apex — Snap Assist Window Docking & Tiling Engine
//! Computes Windows 11-class multi-zone snapping (halves, quarters, maximize, and restore).

use super::color::Color;
use super::geometry::Rect;
use super::surface::Surface;

pub const EDGE_THRESHOLD: i32 = 16;
pub const CORNER_THRESHOLD: i32 = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapPosition {
    None,
    LeftHalf,
    RightHalf,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Maximize,
}

pub struct SnapEngine;

impl SnapEngine {
    pub const fn usable_screen_rect(screen_w: u32, screen_h: u32, taskbar_h: u32) -> Rect {
        let avail_h = if screen_h > taskbar_h { screen_h - taskbar_h } else { screen_h };
        Rect::new(0, 0, screen_w, avail_h)
    }

    pub fn calculate_snap_rect(target: SnapPosition, usable: Rect) -> Rect {
        let half_w = usable.width / 2;
        let rem_w = usable.width - half_w;
        let half_h = usable.height / 2;
        let rem_h = usable.height - half_h;

        match target {
            SnapPosition::LeftHalf => Rect::new(usable.x, usable.y, half_w, usable.height),
            SnapPosition::RightHalf => Rect::new(usable.x + half_w as i32, usable.y, rem_w, usable.height),
            SnapPosition::TopLeft => Rect::new(usable.x, usable.y, half_w, half_h),
            SnapPosition::TopRight => Rect::new(usable.x + half_w as i32, usable.y, rem_w, half_h),
            SnapPosition::BottomLeft => Rect::new(usable.x, usable.y + half_h as i32, half_w, rem_h),
            SnapPosition::BottomRight => Rect::new(usable.x + half_w as i32, usable.y + half_h as i32, rem_w, rem_h),
            SnapPosition::Maximize => usable,
            SnapPosition::None => usable,
        }
    }

    pub fn detect_snap_zone(drag_x: i32, drag_y: i32, screen_w: u32, screen_h: u32, taskbar_h: u32) -> SnapPosition {
        let max_y = (screen_h.saturating_sub(taskbar_h)) as i32;

        // Top edge detection
        if drag_y <= EDGE_THRESHOLD {
            if drag_x <= CORNER_THRESHOLD {
                return SnapPosition::TopLeft;
            } else if drag_x >= (screen_w as i32 - CORNER_THRESHOLD) {
                return SnapPosition::TopRight;
            } else {
                return SnapPosition::Maximize;
            }
        }

        // Bottom edge detection
        if drag_y >= (max_y - EDGE_THRESHOLD) && drag_y <= max_y + 10 {
            if drag_x <= CORNER_THRESHOLD {
                return SnapPosition::BottomLeft;
            } else if drag_x >= (screen_w as i32 - CORNER_THRESHOLD) {
                return SnapPosition::BottomRight;
            }
        }

        // Left edge detection
        if drag_x <= EDGE_THRESHOLD {
            return SnapPosition::LeftHalf;
        }

        // Right edge detection
        if drag_x >= (screen_w as i32 - EDGE_THRESHOLD) {
            return SnapPosition::RightHalf;
        }

        SnapPosition::None
    }

    /// Renders a ghost preview highlight representing the target snap zone
    pub fn render_preview(target: SnapPosition, usable: Rect, surface: &mut Surface) {
        if target == SnapPosition::None {
            return;
        }
        let snap_rect = Self::calculate_snap_rect(target, usable);

        // Translucent cyan glass tint (alpha ~40)
        let fill_color = Color::from_argb(40, 0, 212, 255);
        surface.fill_rect(snap_rect, fill_color);

        // 2-pixel accent border
        surface.draw_rect_outline(snap_rect, Color::NOVA_ACCENT, 2);
    }
}
