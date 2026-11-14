//! NovaOS Apex — Graphics & Window Server Subsystem (Milestone 7)
//! 2D Software Compositor, Window Hierarchy, Alpha Blending, and NovaUI Engine.

pub mod app_tests;
pub mod bar;
pub mod color;
pub mod compositor;
pub mod context_menu;
pub mod cursor;
pub mod damage;
pub mod desktop;
pub mod desktop_features;
pub mod events;
pub mod font;
pub mod geometry;
pub mod launcher;
pub mod notifications;
pub mod quick_settings;
pub mod shell_tests;
pub mod snap;
pub mod surface;
pub mod task_switcher;
pub mod tests;
pub mod window;
pub mod windows_features_test;
pub mod workspace;

pub use bar::NovaBar;
pub use color::Color;
pub use compositor::Compositor;
pub use context_menu::{ContextMenu, ContextAction};
pub use cursor::{Cursor, CursorType};
pub use damage::DamageRegion;
pub use desktop::{DesktopShell, DESKTOP};
pub use desktop_features::{DesktopFeatures, WallpaperTheme};
pub use events::{UserEvent, WindowEvent};
pub use geometry::{Point, Rect, Size};
pub use launcher::NovaLauncher;
pub use notifications::NotificationCenter;
pub use quick_settings::QuickSettings;
pub use snap::{SnapEngine, SnapPosition};
pub use surface::Surface;
pub use task_switcher::TaskSwitcher;
pub use window::{Window, WindowId, WindowState};
pub use workspace::WorkspaceManager;

use crate::drivers::framebuffer::get_driver;
use crate::scheduler::sync::SpinLock;
use crate::serial;

pub static COMPOSITOR: SpinLock<Option<Compositor>> = SpinLock::new("compositor", None);

pub fn init() {
    let fb = get_driver();
    let width = if fb.width > 0 { fb.width } else { 1280 };
    let height = if fb.height > 0 { fb.height } else { 800 };

    // Disable raw console overwriting GOP framebuffer once graphical desktop begins
    crate::drivers::console::set_graphics_active(true);

    let mut comp = Compositor::new(width, height);
    comp.composite();
    *COMPOSITOR.lock() = Some(comp);

    desktop::init();

    serial::write_str("[GRAPHICS] NovaOS Apex 2D Software Compositor & Desktop Shell initialized (");
    print_u64(width as u64);
    serial::write_str("x");
    print_u64(height as u64);
    serial::write_str(")\n");
}

fn print_u64(mut n: u64) {
    if n == 0 {
        serial::write_str("0");
        return;
    }
    let mut buf = [0u8; 20];
    let mut i = 0;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        serial::write_byte(buf[j]);
    }
}
