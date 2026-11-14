//! NovaOS Zero — Desktop Window Server
//! Manages Window Hierarchy, Stacking Z-Order, Window Decorations, and Mouse Pointer Overlay.

use super::compositor::{Color, Rect, Surface};

pub const TITLEBAR_HEIGHT: u32 = 24;
pub const BORDER_WIDTH: u32 = 1;
pub const MAX_WINDOWS: usize = 8;

pub struct Window {
    pub id: u32,
    pub title: &'static str,
    pub rect: Rect,
    pub is_focused: bool,
    pub surface: Surface,
    pub z_order: usize,
}

impl Window {
    pub fn new(id: u32, title: &'static str, x: i32, y: i32, width: u32, height: u32) -> Self {
        let mut surface = Surface::new(width, height);
        surface.clear(Color::WINDOW_BG);

        Window {
            id,
            title,
            rect: Rect::new(x, y, width, height + TITLEBAR_HEIGHT),
            is_focused: false,
            surface,
            z_order: 0,
        }
    }
}

pub struct WindowServer {
    pub windows: [Option<Window>; MAX_WINDOWS],
    pub mouse_x: i32,
    pub mouse_y: i32,
    next_id: u32,
}

impl WindowServer {
    pub const fn new() -> Self {
        const INIT_WINDOW: Option<Window> = None;
        WindowServer {
            windows: [INIT_WINDOW; MAX_WINDOWS],
            mouse_x: 100,
            mouse_y: 100,
            next_id: 1,
        }
    }

    /// Create and register a new managed window
    pub fn create_window(
        &mut self,
        title: &'static str,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Option<u32> {
        for slot in self.windows.iter_mut() {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;
                let mut win = Window::new(id, title, x, y, width, height);
                win.z_order = id as usize;
                *slot = Some(win);
                self.focus_window(id);
                return Some(id);
            }
        }
        None
    }

    /// Bring a window to the top of the Z-order stack and focus it
    pub fn focus_window(&mut self, id: u32) {
        let mut highest_z = 0;
        for win in self.windows.iter().flatten() {
            if win.z_order > highest_z {
                highest_z = win.z_order;
            }
        }

        for win in self.windows.iter_mut().flatten() {
            if win.id == id {
                win.is_focused = true;
                win.z_order = highest_z + 1;
            } else {
                win.is_focused = false;
            }
        }
    }

    /// Move window by relative delta
    pub fn move_window(&mut self, id: u32, dx: i32, dy: i32) {
        for win in self.windows.iter_mut().flatten() {
            if win.id == id {
                win.rect.x += dx;
                win.rect.y += dy;
                break;
            }
        }
    }

    /// Composite all windows in Z-order and draw mouse cursor onto destination surface
    pub fn composite_desktop(&self, dest: &mut Surface) {
        // 1. Draw Wallpaper
        dest.clear(Color::DARK_SLATE);

        // 2. Sort window indices by Z-order
        let mut order: [usize; MAX_WINDOWS] = [0, 1, 2, 3, 4, 5, 6, 7];
        for i in 0..MAX_WINDOWS {
            for j in (i + 1)..MAX_WINDOWS {
                let z_i = self.windows[order[i]].as_ref().map_or(0, |w| w.z_order);
                let z_j = self.windows[order[j]].as_ref().map_or(0, |w| w.z_order);
                if z_i > z_j {
                    order.swap(i, j);
                }
            }
        }

        // 3. Render Windows from back to front
        for &idx in &order {
            if let Some(ref win) = self.windows[idx] {
                self.render_window_decorations(win, dest);
            }
        }

        // 4. Render Mouse Cursor Overlay
        self.render_cursor(dest, self.mouse_x, self.mouse_y);
    }

    fn render_window_decorations(&self, win: &Window, dest: &mut Surface) {
        let x = win.rect.x;
        let y = win.rect.y;
        let w = win.rect.width;
        let h = win.rect.height;

        // Window drop shadow
        dest.fill_rect(Rect::new(x + 4, y + 4, w, h), Color::rgba(0, 0, 0, 80));

        // Window border outline
        let border_color = if win.is_focused { Color::BLUE } else { Color::BORDER };
        dest.draw_rect_outline(Rect::new(x, y, w, h), BORDER_WIDTH, border_color);

        // Titlebar
        let titlebar_color = if win.is_focused { Color::TITLEBAR_BG } else { Color::WINDOW_BG };
        dest.fill_rect(Rect::new(x + 1, y + 1, w - 2, TITLEBAR_HEIGHT), titlebar_color);

        // Title text
        dest.draw_text(x + 8, y + 5, win.title, Color::WHITE);

        // Close Button [x]
        let btn_x = x + w as i32 - 18;
        let btn_y = y + 4;
        dest.fill_rect(Rect::new(btn_x, btn_y, 14, 14), Color::RED);
        dest.draw_text(btn_x + 3, btn_y, "x", Color::WHITE);

        // Blit Client Area Surface
        dest.blit(&win.surface, x + 1, y + TITLEBAR_HEIGHT as i32);
    }

    fn render_cursor(&self, dest: &mut Surface, cx: i32, cy: i32) {
        // 12x12 arrow cursor sprite bitmap
        const CURSOR_MASK: [u16; 12] = [
            0b1000_0000_0000,
            0b1100_0000_0000,
            0b1110_0000_0000,
            0b1111_0000_0000,
            0b1111_1000_0000,
            0b1111_1100_0000,
            0b1111_1110_0000,
            0b1111_0000_0000,
            0b1101_1000_0000,
            0b1000_1100_0000,
            0b0000_1100_0000,
            0b0000_0110_0000,
        ];

        for (row, &bits) in CURSOR_MASK.iter().enumerate() {
            for col in 0..12 {
                if (bits & (0x8000 >> col)) != 0 {
                    dest.set_pixel(cx + col, cy + row as i32, Color::WHITE);
                }
            }
        }
    }
}
