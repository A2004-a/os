//! NovaOS Apex — Damage Tracking & Dirty Rectangle Accumulation
//! Minimizes compositor work by tracking modified screen regions.

use super::geometry::Rect;
use alloc::vec::Vec;

pub const MAX_DAMAGE_RECTS: usize = 16;

#[derive(Clone, Debug)]
pub struct DamageRegion {
    rects: Vec<Rect>,
}

impl DamageRegion {
    pub const fn new() -> Self {
        Self { rects: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }

    pub fn clear(&mut self) {
        self.rects.clear();
    }

    pub fn rects(&self) -> &[Rect] {
        &self.rects
    }

    pub fn add_rect(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }

        // Check if fully contained in an existing rect
        for r in &self.rects {
            if r.x <= rect.x && r.y <= rect.y && r.right() >= rect.right() && r.bottom() >= rect.bottom() {
                return;
            }
        }

        // Try to merge with an intersecting or touching rect
        for r in &mut self.rects {
            if r.intersects(&rect) {
                *r = r.union(&rect);
                return;
            }
        }

        // If list exceeds limit, collapse all into single bounding box
        if self.rects.len() >= MAX_DAMAGE_RECTS {
            let mut bbox = rect;
            for r in &self.rects {
                bbox = bbox.union(r);
            }
            self.rects.clear();
            self.rects.push(bbox);
        } else {
            self.rects.push(rect);
        }
    }

    pub fn bounding_box(&self) -> Option<Rect> {
        if self.rects.is_empty() {
            return None;
        }
        let mut bbox = self.rects[0];
        for r in &self.rects[1..] {
            bbox = bbox.union(r);
        }
        Some(bbox)
    }
}
