//! NovaOS Apex — 2D Geometry Primitives (Point, Size, Rect)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    #[inline(always)]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const ZERO: Self = Self { width: 0, height: 0 };

    #[inline(always)]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const ZERO: Self = Self { x: 0, y: 0, width: 0, height: 0 };

    #[inline(always)]
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    #[inline(always)]
    pub fn from_points(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let left = x0.min(x1);
        let top = y0.min(y1);
        let right = x0.max(x1);
        let bottom = y0.max(y1);
        Self {
            x: left,
            y: top,
            width: (right - left) as u32,
            height: (bottom - top) as u32,
        }
    }

    #[inline(always)]
    pub const fn left(&self) -> i32 {
        self.x
    }

    #[inline(always)]
    pub const fn top(&self) -> i32 {
        self.y
    }

    #[inline(always)]
    pub fn right(&self) -> i32 {
        self.x + self.width as i32
    }

    #[inline(always)]
    pub fn bottom(&self) -> i32 {
        self.y + self.height as i32
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[inline(always)]
    pub fn contains_point(&self, pt: Point) -> bool {
        self.contains_coords(pt.x, pt.y)
    }

    #[inline(always)]
    pub fn contains_coords(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.right() && py >= self.y && py < self.bottom()
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        if self.is_empty() || other.is_empty() {
            return false;
        }
        self.left() < other.right()
            && self.right() > other.left()
            && self.top() < other.bottom()
            && self.bottom() > other.top()
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if left < right && top < bottom {
            Some(Rect {
                x: left,
                y: top,
                width: (right - left) as u32,
                height: (bottom - top) as u32,
            })
        } else {
            None
        }
    }

    pub fn union(&self, other: &Rect) -> Rect {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Rect {
            x: left,
            y: top,
            width: (right - left) as u32,
            height: (bottom - top) as u32,
        }
    }

    pub fn clamp(&self, bounds: Rect) -> Option<Rect> {
        self.intersection(&bounds)
    }

    pub fn inset(&self, dx: i32, dy: i32) -> Rect {
        let double_dx = (dx * 2) as u32;
        let double_dy = (dy * 2) as u32;
        let new_w = if self.width > double_dx { self.width - double_dx } else { 0 };
        let new_h = if self.height > double_dy { self.height - double_dy } else { 0 };
        Rect {
            x: self.x + dx,
            y: self.y + dy,
            width: new_w,
            height: new_h,
        }
    }
}
