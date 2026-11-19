//! Posters, social images and slides as data: formats with print bleed and
//! safe areas, a brand kit, a type scale, and a layout measured with the
//! real font. Everything here is plain Rust and testable without a GPU;
//! `src/bin/render.rs` draws the layout with tiny-skia into PNGs.
//!
//! One piece of content, laid out for each format: the checks (text inside
//! the safe area, nothing overlapping, legible sizes, enough contrast) run
//! on every one of them.

use ab_glyph::{Font, FontRef, GlyphId, VariableFont};

/// Inter (SIL Open Font License 1.1, see fonts/OFL.txt), one variable file
/// holding every weight.
pub const FONT: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

// ---------------------------------------------------------------- formats

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Medium {
    /// Printed: dots per inch, and bleed beyond the trim for the guillotine.
    Print {
        dpi: u32,
    },
    Screen,
}

/// An output format in pixels. `bleed` is extra canvas on every side that is
/// cut off in print; `margins` (left, top, right, bottom, as fractions of the
/// trimmed size) keep text clear of the edge, or of a platform's buttons.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Format {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
    pub bleed: u32,
    pub margins: [f32; 4],
    pub medium: Medium,
}

/// Millimetres to pixels at a resolution.
pub const fn mm(mm: f32, dpi: u32) -> u32 {
    (mm / 25.4 * dpi as f32 + 0.5) as u32
}

pub const A4: Format = Format {
    name: "a4",
    width: mm(210.0, 300),
    height: mm(297.0, 300),
    bleed: mm(3.0, 300),
    margins: [0.07, 0.06, 0.07, 0.06],
    medium: Medium::Print { dpi: 300 },
};
pub const A3: Format = Format {
    name: "a3",
    width: mm(297.0, 300),
    height: mm(420.0, 300),
    bleed: mm(3.0, 300),
    margins: [0.07, 0.06, 0.07, 0.06],
    medium: Medium::Print { dpi: 300 },
};
/// 4:5 feed post (Instagram, Xiaohongshu-style portrait).
pub const PORTRAIT_POST: Format = Format {
    name: "portrait-post",
    width: 1080,
    height: 1350,
    bleed: 0,
    margins: [0.07, 0.06, 0.07, 0.06],
    medium: Medium::Screen,
};
/// 9:16 story: the app draws its header over the top and its reply bar over the bottom.
pub const STORY: Format = Format {
    name: "story",
    width: 1080,
    height: 1920,
    bleed: 0,
    margins: [0.07, 0.13, 0.07, 0.18],
    medium: Medium::Screen,
};
/// The image shown when a link is shared (Open Graph).
pub const LINK_PREVIEW: Format = Format {
    name: "link-preview",
    width: 1200,
    height: 630,
    bleed: 0,
    margins: [0.06, 0.1, 0.06, 0.1],
    medium: Medium::Screen,
};
/// A 16:9 slide.
pub const SLIDE: Format = Format {
    name: "slide",
    width: 1920,
    height: 1080,
    bleed: 0,
    margins: [0.06, 0.08, 0.06, 0.08],
    medium: Medium::Screen,
};
pub const FORMATS: [Format; 6] = [A4, A3, PORTRAIT_POST, STORY, LINK_PREVIEW, SLIDE];

/// A rectangle in canvas pixels, y down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn right(&self) -> f32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
    pub fn contains(&self, o: &Rect) -> bool {
        o.x >= self.x - 0.5
            && o.y >= self.y - 0.5
            && o.right() <= self.right() + 0.5
            && o.bottom() <= self.bottom() + 0.5
    }
    pub fn overlaps(&self, o: &Rect) -> bool {
        self.x < o.right() && o.x < self.right() && self.y < o.bottom() && o.y < self.bottom()
    }
}

impl Format {
    /// Canvas size including bleed.
    pub fn canvas(&self) -> (u32, u32) {
        (self.width + 2 * self.bleed, self.height + 2 * self.bleed)
    }
    /// The trimmed page, in canvas pixels.
    pub fn trim(&self) -> Rect {
        let b = self.bleed as f32;
        Rect {
            x: b,
            y: b,
            w: self.width as f32,
            h: self.height as f32,
        }
    }
    /// Where text and key content must stay.
    pub fn safe(&self) -> Rect {
        let t = self.trim();
        let [l, top, r, bottom] = self.margins;
        Rect {
            x: t.x + l * t.w,
            y: t.y + top * t.h,
            w: t.w * (1.0 - l - r),
            h: t.h * (1.0 - top - bottom),
        }
    }
    pub fn landscape(&self) -> bool {
        self.width > self.height
    }
    /// The smallest body text that stays readable, in pixels. Print: 9 pt
    /// at the format's resolution. Screen: about 11 pt on a phone that
    /// shows the image 375 points wide.
    pub fn min_body_px(&self) -> f32 {
        match self.medium {
            Medium::Print { dpi } => 9.0 / 72.0 * dpi as f32,
            Medium::Screen => 11.0 * self.width.min(self.height).min(1080) as f32 / 375.0,
        }
    }
}

// ---------------------------------------------------------------- brand

/// sRGB colours, 0-255.
pub type Rgb = [u8; 3];

#[derive(Clone, Copy, Debug)]
pub struct Brand {
    pub paper: Rgb,
    pub ink: Rgb,
    /// Secondary text: still has to pass 4.5:1 on paper.
    pub soft: Rgb,
    pub line: Rgb,
    /// The accent for shapes and large type.
    pub accent: Rgb,
    /// A darker accent for small text: the bright one is under 4.5:1 on paper.
    pub accent_text: Rgb,
    /// Weights from the variable font.
    pub regular: f32,
    pub medium: f32,
    pub bold: f32,
}

pub const BRAND: Brand = Brand {
    paper: [0xf3, 0xf0, 0xea],
    ink: [0x1f, 0x1d, 0x1a],
    soft: [0x5f, 0x5a, 0x53],
    line: [0xd8, 0xd2, 0xc7],
    accent: [0xd4, 0x4f, 0x22],
    accent_text: [0xa8, 0x37, 0x0e],
    regular: 400.0,
    medium: 560.0,
    bold: 760.0,
};

/// WCAG relative luminance and contrast ratio.
pub fn contrast(a: Rgb, b: Rgb) -> f32 {
    let lum = |c: Rgb| {
        let ch = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(c[0]) + 0.7152 * ch(c[1]) + 0.0722 * ch(c[2])
    };
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

// ---------------------------------------------------------------- content

#[derive(Clone, Copy, Debug)]
pub struct Content {
    pub kicker: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub body: &'static str,
    pub stats: [(&'static str, &'static str); 3],
    pub cta: &'static str,
}

pub const CONTENT: Content = Content {
    kicker: "PLAYMAND 0.5",
    title: "Look first. Build it in Rust.",
    subtitle: "Games, videos and web effects made to a picture of the target.",
    body: "Agree on the look, bring in the assets code cannot make, check every one, then let Rust do the light, the motion and the effects.",
    stats: [
        ("66 KB", "card pack on the web"),
        ("32 KB", "living poster"),
        ("0.20", "Bevy, version-matched"),
    ],
    cta: "playmand.com",
};

// ---------------------------------------------------------------- type

/// The font at three weights.
pub struct Fonts {
    pub regular: FontRef<'static>,
    pub medium: FontRef<'static>,
    pub bold: FontRef<'static>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Weight {
    Regular,
    Medium,
    Bold,
}

impl Fonts {
    pub fn new(brand: &Brand) -> Self {
        let at = |w: f32| {
            let mut f = FontRef::try_from_slice(FONT).expect("the bundled font parses");
            assert!(f.set_variation(b"wght", w), "the font has a weight axis");
            f
        };
        Fonts {
            regular: at(brand.regular),
            medium: at(brand.medium),
            bold: at(brand.bold),
        }
    }
    pub fn get(&self, w: Weight) -> &FontRef<'static> {
        match w {
            Weight::Regular => &self.regular,
            Weight::Medium => &self.medium,
            Weight::Bold => &self.bold,
        }
    }
}

/// Pixels per font unit at a text size (the size is the em, in pixels).
pub fn px_per_unit(font: &FontRef, size: f32) -> f32 {
    size / font.units_per_em().unwrap_or(1000.0)
}

/// Width of a line of text, with kerning and letter spacing (`tracking`, in ems).
pub fn text_width(font: &FontRef, size: f32, tracking: f32, text: &str) -> f32 {
    let k = px_per_unit(font, size);
    let mut w = 0.0;
    let mut prev: Option<GlyphId> = None;
    for c in text.chars() {
        let g = font.glyph_id(c);
        if let Some(p) = prev {
            w += font.kern_unscaled(p, g) * k;
        }
        w += font.h_advance_unscaled(g) * k + tracking * size;
        prev = Some(g);
    }
    w - if text.is_empty() {
        0.0
    } else {
        tracking * size
    }
}

/// Greedy word wrap to a width.
pub fn wrap(font: &FontRef, size: f32, tracking: f32, text: &str, max: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && text_width(font, size, tracking, &candidate) > max {
            lines.push(std::mem::take(&mut line));
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

// ---------------------------------------------------------------- layout

#[derive(Clone, Debug)]
pub struct TextBlock {
    pub role: &'static str,
    pub lines: Vec<String>,
    /// Top-left of the first line's em box.
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub leading: f32,
    pub tracking: f32,
    pub weight: Weight,
    pub colour: Rgb,
    /// What is behind it (for the contrast check).
    pub on: Rgb,
    /// Large text may use 3:1 contrast; body text needs 4.5:1.
    pub large: bool,
    /// The measured box the text occupies.
    pub bounds: Rect,
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Rect {
        r: Rect,
        colour: Rgb,
    },
    Circle {
        cx: f32,
        cy: f32,
        radius: f32,
        colour: Rgb,
    },
    Ring {
        cx: f32,
        cy: f32,
        radius: f32,
        width: f32,
        colour: Rgb,
    },
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub format: Format,
    pub background: Rgb,
    pub shapes: Vec<Shape>,
    pub texts: Vec<TextBlock>,
}

#[allow(clippy::too_many_arguments)]
fn block(
    fonts: &Fonts,
    role: &'static str,
    text: &str,
    x: f32,
    y: f32,
    max_w: f32,
    size: f32,
    leading: f32,
    tracking: f32,
    weight: Weight,
    colour: Rgb,
    on: Rgb,
    large: bool,
) -> TextBlock {
    let font = fonts.get(weight);
    let lines = wrap(font, size, tracking, text, max_w);
    let w = lines
        .iter()
        .map(|l| text_width(font, size, tracking, l))
        .fold(0.0, f32::max);
    let h = size * leading * (lines.len() as f32 - 1.0) + size * 1.2;
    TextBlock {
        role,
        lines,
        x,
        y,
        size,
        leading,
        tracking,
        weight,
        colour,
        on,
        large,
        bounds: Rect { x, y, w, h },
    }
}

/// Lay out the content for a format. Sizes follow the trimmed page's short
/// side, the title shrinks until it fits in three lines, and wide formats
/// put the numbers in a column of their own. Tall formats put the accent
/// disc above the text; it is tried from large to small, and the first size
/// at which every check passes wins, so tall pages get a bold disc and
/// short ones give the text room.
pub fn layout(f: &Format, brand: &Brand, c: &Content, fonts: &Fonts) -> Layout {
    // The body is the one optional piece: keep it if any disc size leaves
    // room, otherwise drop it and keep the disc large.
    let mut last = None;
    for with_body in [true, false] {
        for scale in [1.0, 0.86, 0.72, 0.6, 0.48, 0.36] {
            let l = compose(f, brand, c, fonts, scale, with_body);
            if check(&l).is_empty() {
                return l;
            }
            last = Some(l);
        }
    }
    last.expect("at least one attempt")
}

fn compose(
    f: &Format,
    brand: &Brand,
    c: &Content,
    fonts: &Fonts,
    disc: f32,
    with_body: bool,
) -> Layout {
    let safe = f.safe();
    let trim = f.trim();
    let u = trim.w.min(trim.h);
    let mut shapes = Vec::new();
    // Decoration: a large accent disc bleeding off a corner, rings around it.
    // Wide formats: bottom right, beside the text. Tall formats: top right,
    // with the text starting below it.
    let (cx, cy, radius) = if f.landscape() {
        (
            trim.right() - 0.08 * trim.w,
            trim.bottom() - 0.04 * trim.h,
            0.42 * u * disc,
        )
    } else {
        let radius = 0.42 * trim.w * disc;
        (trim.right() - 0.12 * trim.w, trim.y + 0.04 * trim.h, radius)
    };
    for k in 1..=4 {
        shapes.push(Shape::Ring {
            cx,
            cy,
            radius: radius * (1.0 + 0.16 * k as f32),
            width: 0.0025 * u,
            colour: brand.line,
        });
    }
    shapes.push(Shape::Circle {
        cx,
        cy,
        radius,
        colour: brand.accent,
    });

    let text_w = if f.landscape() {
        safe.w * 0.6
    } else {
        safe.w * 0.86
    };
    let mut texts = Vec::new();
    let mut y = if f.landscape() {
        safe.y
    } else {
        safe.y.max(cy + radius + 0.05 * u)
    };
    let kicker = block(
        fonts,
        "kicker",
        c.kicker,
        safe.x,
        y,
        text_w,
        (0.026 * u).max(f.min_body_px()),
        1.2,
        0.14,
        Weight::Medium,
        brand.accent_text,
        brand.paper,
        false,
    );
    y = kicker.bounds.bottom() + 0.035 * u;
    texts.push(kicker);
    // The title: as big as fits in three lines.
    let mut size = if f.landscape() { 0.13 * u } else { 0.115 * u };
    let mut title = block(
        fonts,
        "title",
        c.title,
        safe.x,
        y,
        text_w,
        size,
        1.02,
        -0.02,
        Weight::Bold,
        brand.ink,
        brand.paper,
        true,
    );
    while title.lines.len() > 3 {
        size *= 0.94;
        title = block(
            fonts,
            "title",
            c.title,
            safe.x,
            y,
            text_w,
            size,
            1.02,
            -0.02,
            Weight::Bold,
            brand.ink,
            brand.paper,
            true,
        );
    }
    y = title.bounds.bottom() + 0.035 * u;
    texts.push(title);
    let subtitle = block(
        fonts,
        "subtitle",
        c.subtitle,
        safe.x,
        y,
        text_w,
        0.042 * u,
        1.25,
        0.0,
        Weight::Medium,
        brand.soft,
        brand.paper,
        false,
    );
    y = subtitle.bounds.bottom() + 0.04 * u;
    texts.push(subtitle);
    shapes.push(Shape::Rect {
        r: Rect {
            x: safe.x,
            y,
            w: 0.12 * u,
            h: 0.008 * u,
        },
        colour: brand.accent,
    });
    y += 0.04 * u;
    let body_size = (0.03 * u).max(f.min_body_px());
    if with_body && (!f.landscape() || f.height >= 1000) {
        let body = block(
            fonts,
            "body",
            c.body,
            safe.x,
            y,
            text_w.min(safe.w * 0.8),
            body_size,
            1.45,
            0.0,
            Weight::Regular,
            brand.ink,
            brand.paper,
            false,
        );
        texts.push(body);
    }

    // Numbers: a row along the bottom (portrait), or a column on the right (wide).
    let value_size = 0.07 * u;
    let label_size = (0.024 * u).max(f.min_body_px());
    let foot = (0.03 * u).max(f.min_body_px());
    let foot_y = safe.bottom() - foot * 1.2;
    if f.landscape() {
        let col_x = safe.x + safe.w * 0.68;
        let col_w = safe.right() - col_x;
        let mut sy = safe.y + 0.02 * u;
        for (i, (v, l)) in c.stats.iter().enumerate() {
            let value = block(
                fonts,
                "stat",
                v,
                col_x,
                sy,
                col_w,
                value_size,
                1.0,
                -0.02,
                Weight::Bold,
                brand.ink,
                brand.paper,
                true,
            );
            let label = block(
                fonts,
                "label",
                l,
                col_x,
                value.bounds.bottom() + 0.004 * u,
                col_w,
                label_size,
                1.25,
                0.0,
                Weight::Regular,
                brand.soft,
                brand.paper,
                false,
            );
            sy = label.bounds.bottom() + 0.04 * u;
            if i < 2 {
                shapes.push(Shape::Rect {
                    r: Rect {
                        x: col_x,
                        y: sy - 0.02 * u,
                        w: col_w,
                        h: 0.002 * u,
                    },
                    colour: brand.line,
                });
            }
            texts.push(value);
            texts.push(label);
        }
    } else {
        let col_w = safe.w / 3.0;
        let label_lines = 2.0;
        let row_y = foot_y - 0.06 * u - value_size * 1.2 - label_size * 1.25 * label_lines;
        shapes.push(Shape::Rect {
            r: Rect {
                x: safe.x,
                y: row_y - 0.035 * u,
                w: safe.w,
                h: 0.002 * u,
            },
            colour: brand.line,
        });
        for (i, (v, l)) in c.stats.iter().enumerate() {
            let x = safe.x + i as f32 * col_w;
            let value = block(
                fonts,
                "stat",
                v,
                x,
                row_y,
                col_w * 0.92,
                value_size,
                1.0,
                -0.02,
                Weight::Bold,
                brand.ink,
                brand.paper,
                true,
            );
            let label = block(
                fonts,
                "label",
                l,
                x,
                value.bounds.bottom() + 0.006 * u,
                col_w * 0.88,
                label_size,
                1.25,
                0.0,
                Weight::Regular,
                brand.soft,
                brand.paper,
                false,
            );
            texts.push(value);
            texts.push(label);
        }
    }
    texts.push(block(
        fonts,
        "cta",
        c.cta,
        safe.x,
        foot_y,
        safe.w,
        foot,
        1.2,
        0.0,
        Weight::Bold,
        brand.ink,
        brand.paper,
        false,
    ));
    Layout {
        format: *f,
        background: brand.paper,
        shapes,
        texts,
    }
}

/// Everything wrong with a layout, as messages; empty means it passes.
pub fn check(l: &Layout) -> Vec<String> {
    let mut problems = Vec::new();
    let safe = l.format.safe();
    for t in &l.texts {
        // Text over a solid shape is no longer on the paper its contrast was checked against.
        for s in &l.shapes {
            if let Shape::Rect { r, .. } = *s {
                if r.overlaps(&t.bounds) {
                    problems.push(format!("{}: a rule runs through {}", l.format.name, t.role));
                }
            }
            if let Shape::Circle { cx, cy, radius, .. } = *s {
                let nx = cx.clamp(t.bounds.x, t.bounds.right());
                let ny = cy.clamp(t.bounds.y, t.bounds.bottom());
                if (nx - cx).hypot(ny - cy) < radius {
                    problems.push(format!(
                        "{}: {} runs over the accent disc",
                        l.format.name, t.role
                    ));
                }
            }
        }
        if !safe.contains(&t.bounds) {
            problems.push(format!(
                "{}: {} outside the safe area",
                l.format.name, t.role
            ));
        }
        let ratio = contrast(t.colour, t.on);
        let need = if t.large { 3.0 } else { 4.5 };
        if ratio < need {
            problems.push(format!(
                "{}: {} contrast {ratio:.2}:1, needs {need}:1",
                l.format.name, t.role
            ));
        }
        if !t.large && t.size + 0.01 < l.format.min_body_px() {
            problems.push(format!(
                "{}: {} at {:.0} px, below the {:.0} px minimum",
                l.format.name,
                t.role,
                t.size,
                l.format.min_body_px()
            ));
        }
    }
    for (i, a) in l.texts.iter().enumerate() {
        for b in &l.texts[i + 1..] {
            if a.bounds.overlaps(&b.bounds) {
                problems.push(format!("{}: {} overlaps {}", l.format.name, a.role, b.role));
            }
        }
    }
    if let Some(t) = l.texts.iter().find(|t| t.role == "title") {
        if t.lines.len() > 3 {
            problems.push(format!(
                "{}: title on {} lines",
                l.format.name,
                t.lines.len()
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Vec<Layout> {
        let fonts = Fonts::new(&BRAND);
        FORMATS
            .iter()
            .map(|f| layout(f, &BRAND, &CONTENT, &fonts))
            .collect()
    }

    #[test]
    fn print_sizes_are_exact_at_300_dpi_with_3_mm_bleed() {
        assert_eq!((A4.width, A4.height, A4.bleed), (2480, 3508, 35));
        assert_eq!((A3.width, A3.height), (3508, 4961));
        assert_eq!(A4.canvas(), (2550, 3578));
        assert!((A4.min_body_px() - 37.5).abs() < 0.01, "9 pt at 300 dpi");
    }

    #[test]
    fn every_format_passes_every_check() {
        for l in all() {
            let problems = check(&l);
            assert!(problems.is_empty(), "{problems:#?}");
        }
    }

    #[test]
    fn no_content_is_dropped() {
        for l in all() {
            let text: String = l
                .texts
                .iter()
                .flat_map(|t| t.lines.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            for s in [CONTENT.title, CONTENT.subtitle, CONTENT.cta] {
                let words: Vec<&str> = s.split_whitespace().collect();
                assert!(
                    words.iter().all(|w| text.contains(w)),
                    "{}: missing words of {s:?}",
                    l.format.name
                );
            }
            for (v, _) in CONTENT.stats {
                assert!(text.contains(v), "{}: missing {v}", l.format.name);
            }
        }
    }

    #[test]
    fn the_brand_colours_are_legible() {
        assert!(contrast(BRAND.ink, BRAND.paper) >= 7.0);
        assert!(contrast(BRAND.soft, BRAND.paper) >= 4.5);
        assert!(contrast(BRAND.accent_text, BRAND.paper) >= 4.5);
        assert!(
            contrast(BRAND.accent, BRAND.paper) < 4.5,
            "why accent_text exists"
        );
        assert!((contrast([0, 0, 0], [255, 255, 255]) - 21.0).abs() < 0.01);
    }

    #[test]
    fn the_checker_catches_problems() {
        let fonts = Fonts::new(&BRAND);
        let mut l = layout(&PORTRAIT_POST, &BRAND, &CONTENT, &fonts);
        l.texts[0].bounds.x = -10.0;
        l.texts[1].colour = [0xc0, 0xbc, 0xb4];
        l.texts[2].bounds = l.texts[1].bounds;
        if let Some(Shape::Circle { cx, cy, .. }) = l
            .shapes
            .iter()
            .find(|s| matches!(s, Shape::Circle { .. }))
            .copied()
        {
            l.texts[3].bounds = Rect {
                x: cx - 10.0,
                y: cy - 10.0,
                w: 20.0,
                h: 20.0,
            };
        }
        let problems = check(&l).join("\n");
        assert!(problems.contains("runs over the accent disc"), "{problems}");
        assert!(problems.contains("kicker outside"), "{problems}");
        assert!(problems.contains("title contrast"), "{problems}");
        assert!(problems.contains("overlaps"), "{problems}");
    }

    #[test]
    fn wrapping_respects_the_width() {
        let fonts = Fonts::new(&BRAND);
        let lines = wrap(&fonts.regular, 40.0, 0.0, CONTENT.body, 600.0);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(text_width(&fonts.regular, 40.0, 0.0, line) <= 600.0 || !line.contains(' '));
        }
        assert!(
            text_width(&fonts.bold, 40.0, 0.0, "Rust")
                > text_width(&fonts.regular, 40.0, 0.0, "Rust"),
            "bold is wider"
        );
    }
}
