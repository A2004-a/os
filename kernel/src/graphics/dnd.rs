//! NovaOS Apex — Desktop Drag-and-Drop Subsystem (M11)
//! Provides interactive drag gesture initiation, translucent drag ghost rendering,
//! target drop-zone detection, payload encapsulation, and cancel/commit lifecycles.

use super::color::Color;
use super::font::draw_text;
use super::geometry::{Point, Rect};
use super::surface::Surface;
use super::window::WindowId;

pub const DRAG_THRESHOLD_PIXELS: i32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DndPayloadType {
    File,
    Text,
    WindowTab,
}

#[derive(Clone, Copy, Debug)]
pub struct DndPayload {
    pub payload_type: DndPayloadType,
    pub data: [u8; 64],
    pub len: usize,
    pub is_directory: bool,
}

impl DndPayload {
    pub fn file(path: &str, is_dir: bool) -> Self {
        let mut data = [0u8; 64];
        let bytes = path.as_bytes();
        let len = bytes.len().min(64);
        data[..len].copy_from_slice(&bytes[..len]);
        Self {
            payload_type: DndPayloadType::File,
            data,
            len,
            is_directory: is_dir,
        }
    }

    pub fn text(s: &str) -> Self {
        let mut data = [0u8; 64];
        let bytes = s.as_bytes();
        let len = bytes.len().min(64);
        data[..len].copy_from_slice(&bytes[..len]);
        Self {
            payload_type: DndPayloadType::Text,
            data,
            len,
            is_directory: false,
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DndPhase {
    Idle,
    Potential,
    Dragging,
    Dropped,
}

pub struct DndEngine {
    pub phase: DndPhase,
    pub start_pos: Point,
    pub current_pos: Point,
    pub source_window: Option<WindowId>,
    pub target_window: Option<WindowId>,
    pub payload: Option<DndPayload>,
}

impl DndEngine {
    pub const fn new() -> Self {
        Self {
            phase: DndPhase::Idle,
            start_pos: Point::new(0, 0),
            current_pos: Point::new(0, 0),
            source_window: None,
            target_window: None,
            payload: None,
        }
    }

    pub fn start_potential(&mut self, source_win: WindowId, pos: Point, payload: DndPayload) {
        self.phase = DndPhase::Potential;
        self.start_pos = pos;
        self.current_pos = pos;
        self.source_window = Some(source_win);
        self.target_window = None;
        self.payload = Some(payload);
    }

    pub fn on_mouse_move(&mut self, pos: Point) -> bool {
        self.current_pos = pos;
        if self.phase == DndPhase::Potential {
            let dx = (pos.x - self.start_pos.x).abs();
            let dy = (pos.y - self.start_pos.y).abs();
            if dx >= DRAG_THRESHOLD_PIXELS || dy >= DRAG_THRESHOLD_PIXELS {
                self.phase = DndPhase::Dragging;
                return true;
            }
        }
        self.phase == DndPhase::Dragging
    }

    pub fn on_mouse_up(&mut self, target_win: Option<WindowId>) -> Option<DndPayload> {
        if self.phase == DndPhase::Dragging {
            self.phase = DndPhase::Dropped;
            self.target_window = target_win;
            let result = self.payload.take();
            self.reset();
            result
        } else {
            self.reset();
            None
        }
    }

    pub fn cancel(&mut self) {
        self.reset();
    }

    pub fn is_dragging(&self) -> bool {
        self.phase == DndPhase::Dragging
    }

    pub fn ghost_rect(&self) -> Rect {
        let w = 140;
        let h = 32;
        Rect::new(self.current_pos.x + 12, self.current_pos.y + 12, w, h)
    }

    pub fn render_ghost(&self, surf: &mut Surface) {
        if self.phase != DndPhase::Dragging {
            return;
        }
        let gr = self.ghost_rect();
        // Translucent dark slate badge with bright cyan border
        surf.fill_rect(gr, Color::from_argb(200, 0x1A, 0x23, 0x33));
        surf.draw_rect_outline(gr, Color::NOVA_ACCENT, 1);

        let icon = match self.payload.map(|p| p.payload_type) {
            Some(DndPayloadType::File) => "[F]",
            Some(DndPayloadType::Text) => "[T]",
            Some(DndPayloadType::WindowTab) => "[W]",
            None => "[*]",
        };
        draw_text(surf, icon, gr.x + 6, gr.y + 8, Color::NOVA_ACCENT);

        let label = self.payload.as_ref().map(|p| p.as_str()).unwrap_or("Item");
        let display_label = if label.len() > 12 { &label[..12] } else { label };
        draw_text(surf, display_label, gr.x + 36, gr.y + 8, Color::TEXT_PRIMARY);
    }

    fn reset(&mut self) {
        self.phase = DndPhase::Idle;
        self.source_window = None;
        self.target_window = None;
        self.payload = None;
    }
}
