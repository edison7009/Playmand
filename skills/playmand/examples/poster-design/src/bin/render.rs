//! Renders the poster layout for every format into PNGs, on the CPU with
//! tiny-skia (no GPU needed), plus a contact sheet of all of them.
//!
//! cargo run --release -- <out-dir>
//!
//! Print formats are written with their bleed and 300 dpi in the file, and
//! a trimmed preview beside them. Text is drawn from the font's own outlines,
//! with its kerning, so what the layout measured is what gets drawn.

use std::{fs, io::BufWriter, path::Path};

use ab_glyph::{Font, OutlineCurve};
use playmand_poster_design::*;
use tiny_skia::{
    Color, FillRule, FilterQuality, Paint, PathBuilder, Pixmap, PixmapPaint, Rect as SkRect,
    Stroke, Transform,
};

fn paint(c: Rgb) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color(Color::from_rgba8(c[0], c[1], c[2], 255));
    p.anti_alias = true;
    p
}

fn draw_text(pm: &mut Pixmap, fonts: &Fonts, t: &TextBlock) {
    let font = fonts.get(t.weight);
    let k = px_per_unit(font, t.size);
    let ascent = font.ascent_unscaled() * k;
    let mut pb = PathBuilder::new();
    for (i, line) in t.lines.iter().enumerate() {
        let baseline = t.y + ascent + i as f32 * t.size * t.leading;
        let mut x = t.x;
        let mut prev = None;
        for c in line.chars() {
            let g = font.glyph_id(c);
            if let Some(p) = prev {
                x += font.kern_unscaled(p, g) * k;
            }
            if let Some(outline) = font.outline(g) {
                let pt = |p: ab_glyph::Point| (x + p.x * k, baseline - p.y * k);
                let mut last: Option<(f32, f32)> = None;
                for curve in &outline.curves {
                    let (start, end) = match curve {
                        OutlineCurve::Line(a, b) => (pt(*a), pt(*b)),
                        OutlineCurve::Quad(a, _, b) => (pt(*a), pt(*b)),
                        OutlineCurve::Cubic(a, _, _, b) => (pt(*a), pt(*b)),
                    };
                    // A curve that does not continue the last one starts a new contour.
                    if last.is_none_or(|l| {
                        (l.0 - start.0).abs() > 0.01 || (l.1 - start.1).abs() > 0.01
                    }) {
                        if last.is_some() {
                            pb.close();
                        }
                        pb.move_to(start.0, start.1);
                    }
                    match curve {
                        OutlineCurve::Line(_, _) => pb.line_to(end.0, end.1),
                        OutlineCurve::Quad(_, c1, _) => {
                            let c = pt(*c1);
                            pb.quad_to(c.0, c.1, end.0, end.1)
                        }
                        OutlineCurve::Cubic(_, c1, c2, _) => {
                            let (a, b) = (pt(*c1), pt(*c2));
                            pb.cubic_to(a.0, a.1, b.0, b.1, end.0, end.1)
                        }
                    }
                    last = Some(end);
                }
                if last.is_some() {
                    pb.close();
                }
            }
            x += font.h_advance_unscaled(g) * k + t.tracking * t.size;
            prev = Some(g);
        }
    }
    if let Some(path) = pb.finish() {
        pm.fill_path(
            &path,
            &paint(t.colour),
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
}

fn draw(l: &Layout, fonts: &Fonts) -> Pixmap {
    let (w, h) = l.format.canvas();
    let mut pm = Pixmap::new(w, h).expect("canvas size");
    let bg = l.background;
    pm.fill(Color::from_rgba8(bg[0], bg[1], bg[2], 255));
    for s in &l.shapes {
        match *s {
            Shape::Rect { r, colour } => {
                if let Some(rect) = SkRect::from_xywh(r.x, r.y, r.w, r.h) {
                    pm.fill_rect(rect, &paint(colour), Transform::identity(), None);
                }
            }
            Shape::Circle {
                cx,
                cy,
                radius,
                colour,
            } => {
                if let Some(path) = PathBuilder::from_circle(cx, cy, radius) {
                    pm.fill_path(
                        &path,
                        &paint(colour),
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            Shape::Ring {
                cx,
                cy,
                radius,
                width,
                colour,
            } => {
                if let Some(path) = PathBuilder::from_circle(cx, cy, radius) {
                    let stroke = Stroke {
                        width,
                        ..Stroke::default()
                    };
                    pm.stroke_path(&path, &paint(colour), &stroke, Transform::identity(), None);
                }
            }
        }
    }
    for t in &l.texts {
        draw_text(&mut pm, fonts, t);
    }
    pm
}

/// PNG with the resolution recorded, so print software places it at the right size.
fn save(pm: &Pixmap, path: &Path, dpi: Option<u32>) {
    let file = fs::File::create(path).expect("create png");
    let mut enc = png::Encoder::new(BufWriter::new(file), pm.width(), pm.height());
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    if let Some(dpi) = dpi {
        let ppm = (dpi as f32 / 0.0254).round() as u32;
        enc.set_pixel_dims(Some(png::PixelDimensions {
            xppu: ppm,
            yppu: ppm,
            unit: png::Unit::Meter,
        }));
    }
    // tiny-skia keeps premultiplied pixels; every pixel here is opaque, so they are already straight.
    enc.write_header()
        .and_then(|mut w| w.write_image_data(pm.data()))
        .expect("write png");
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "out".into());
    let out = Path::new(&out);
    fs::create_dir_all(out).expect("create output folder");
    let fonts = Fonts::new(&BRAND);
    let mut tiles = Vec::new();
    for f in FORMATS {
        let l = layout(&f, &BRAND, &CONTENT, &fonts);
        let problems = check(&l);
        assert!(problems.is_empty(), "{problems:#?}");
        let pm = draw(&l, &fonts);
        match f.medium {
            Medium::Print { dpi } => {
                save(&pm, &out.join(format!("{}-print.png", f.name)), Some(dpi));
                // The trimmed page, as it will look once cut.
                let t = f.trim();
                let trimmed = pm
                    .clone_rect(
                        tiny_skia::IntRect::from_xywh(
                            t.x as i32, t.y as i32, t.w as u32, t.h as u32,
                        )
                        .unwrap(),
                    )
                    .unwrap();
                save(&trimmed, &out.join(format!("{}.png", f.name)), Some(dpi));
                tiles.push(trimmed);
            }
            Medium::Screen => {
                save(&pm, &out.join(format!("{}.png", f.name)), None);
                tiles.push(pm);
            }
        }
        println!("{:<13} {}x{} px", f.name, f.canvas().0, f.canvas().1);
    }
    // Contact sheet: every format at the same height, side by side.
    let th = 900.0;
    let widths: Vec<f32> = tiles
        .iter()
        .map(|p| p.width() as f32 * th / p.height() as f32)
        .collect();
    let gap = 40.0;
    let sw = widths.iter().sum::<f32>() + gap * (tiles.len() as f32 + 1.0);
    let mut sheet = Pixmap::new(sw as u32, (th + 2.0 * gap) as u32).unwrap();
    sheet.fill(Color::from_rgba8(0x2a, 0x29, 0x27, 255));
    let mut x = gap;
    for (pm, w) in tiles.iter().zip(&widths) {
        let s = th / pm.height() as f32;
        let p = PixmapPaint {
            quality: FilterQuality::Bicubic,
            ..PixmapPaint::default()
        };
        sheet.draw_pixmap(
            0,
            0,
            pm.as_ref(),
            &p,
            Transform::from_scale(s, s).post_translate(x, gap),
            None,
        );
        x += w + gap;
    }
    save(&sheet, &out.join("sheet.png"), None);
    println!("contact sheet: {}", out.join("sheet.png").display());
}
