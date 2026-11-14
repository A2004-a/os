//! NovaOS Apex — 32-Bit ARGB Color Model & Alpha Blending Engine
//! Windows 11-class modern dark palette with Porter-Duff alpha compositing.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub raw: u32,
}

impl Color {
    pub const TRANSPARENT: Self = Self::from_argb(0, 0, 0, 0);
    pub const BLACK: Self = Self::from_rgb(0, 0, 0);
    pub const WHITE: Self = Self::from_rgb(255, 255, 255);
    pub const RED: Self = Self::from_rgb(255, 0, 0);
    pub const GREEN: Self = Self::from_rgb(0, 255, 0);
    pub const BLUE: Self = Self::from_rgb(0, 0, 255);

    // Modern NovaOS Apex Palette (Windows 11-class Dark/Mica identity)
    pub const NOVA_MIDNIGHT: Self = Self::from_hex(0x0D1117);
    pub const NOVA_SURFACE: Self = Self::from_hex(0x161B22);
    pub const NOVA_CARD: Self = Self::from_hex(0x21262D);
    pub const NOVA_BORDER: Self = Self::from_hex(0x30363D);
    pub const NOVA_ACCENT: Self = Self::from_hex(0x00D4FF);
    pub const NOVA_ACCENT_HOVER: Self = Self::from_hex(0x33DDFF);
    pub const NOVA_ACCENT_PRESSED: Self = Self::from_hex(0x00A3C4);
    pub const TEXT_PRIMARY: Self = Self::from_hex(0xF0F6FC);
    pub const TEXT_SECONDARY: Self = Self::from_hex(0x8B949E);
    pub const SUCCESS_GREEN: Self = Self::from_hex(0x238636);
    pub const DANGER_RED: Self = Self::from_hex(0xDA3633);
    pub const WARNING_AMBER: Self = Self::from_hex(0xD29922);
    pub const TITLE_BAR_ACTIVE: Self = Self::from_hex(0x1F242C);
    pub const TITLE_BAR_INACTIVE: Self = Self::from_hex(0x14181F);
    pub const CLOSE_BUTTON_HOVER: Self = Self::from_hex(0xC42B1C);

    #[inline(always)]
    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        Self {
            raw: ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32),
        }
    }

    #[inline(always)]
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_argb(255, r, g, b)
    }

    #[inline(always)]
    pub const fn from_hex(hex: u32) -> Self {
        Self::from_rgb(
            ((hex >> 16) & 0xFF) as u8,
            ((hex >> 8) & 0xFF) as u8,
            (hex & 0xFF) as u8,
        )
    }

    #[inline(always)]
    pub const fn a(self) -> u8 {
        ((self.raw >> 24) & 0xFF) as u8
    }

    #[inline(always)]
    pub const fn r(self) -> u8 {
        ((self.raw >> 16) & 0xFF) as u8
    }

    #[inline(always)]
    pub const fn g(self) -> u8 {
        ((self.raw >> 8) & 0xFF) as u8
    }

    #[inline(always)]
    pub const fn b(self) -> u8 {
        (self.raw & 0xFF) as u8
    }

    #[inline(always)]
    pub fn with_alpha(self, alpha: u8) -> Self {
        Self::from_argb(alpha, self.r(), self.g(), self.b())
    }

    /// Porter-Duff Source-Over alpha blending
    #[inline(always)]
    pub fn blend(src: Color, dst: Color) -> Color {
        let sa = src.a();
        if sa == 255 {
            return src;
        }
        if sa == 0 {
            return dst;
        }

        let sa32 = sa as u32;
        let inv_sa = 255 - sa32;

        let r = ((src.r() as u32 * sa32) + (dst.r() as u32 * inv_sa)) / 255;
        let g = ((src.g() as u32 * sa32) + (dst.g() as u32 * inv_sa)) / 255;
        let b = ((src.b() as u32 * sa32) + (dst.b() as u32 * inv_sa)) / 255;

        Color::from_argb(255, r as u8, g as u8, b as u8)
    }

    /// Alpha blending with additional global opacity modulation (0..255)
    #[inline(always)]
    pub fn blend_with_opacity(src: Color, dst: Color, global_opacity: u8) -> Color {
        if global_opacity == 0 {
            return dst;
        }
        let effective_sa = ((src.a() as u32) * (global_opacity as u32)) / 255;
        if effective_sa == 255 {
            return src;
        }
        if effective_sa == 0 {
            return dst;
        }

        let inv_sa = 255 - effective_sa;

        let r = ((src.r() as u32 * effective_sa) + (dst.r() as u32 * inv_sa)) / 255;
        let g = ((src.g() as u32 * effective_sa) + (dst.g() as u32 * inv_sa)) / 255;
        let b = ((src.b() as u32 * effective_sa) + (dst.b() as u32 * inv_sa)) / 255;

        Color::from_argb(255, r as u8, g as u8, b as u8)
    }
}
