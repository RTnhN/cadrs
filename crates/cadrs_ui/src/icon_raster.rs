//! How an icon-rs icon becomes pixels. Shared by `build.rs`, which embeds the common sizes, and
//! the runtime, which renders other sizes on demand (a UI scale or a high-DPI screen draws a
//! 16 px icon at 24 or 32 physical pixels, which an upscaled 16 px raster would blur).
//!
//! How an icon is rasterized depends on its kind:
//! - Glyph and line icons ([`icon_rs::Kind::is_tintable`]) are ink plus a small accent detail.
//!   The ink is rendered in white so the runtime can tint it through `ImageNode::color`; the
//!   accent detail is a second, separate white layer that the runtime tints with the accent.
//! - Solid and sketch icons keep their full colours; the runtime only fades them.

use icon_rs::{Icon, Palette};
use resvg::{tiny_skia, usvg};

/// Heavier copies of tintable icons, as (name, icon, stroke scale): the dialog's ✓ and ✕, the
/// landing page's create button, and the feature tree's chevrons (about 2 px at 18 px,
/// `screens/05`).
pub const WEIGHTS: &[(&str, &str, f32)] = &[
    ("check-bold", "check", 2.0),
    ("x-bold", "close", 2.0),
    ("plus-bold", "plus", 2.0),
    ("chevron-down-medium", "chevron-down", 1.3),
    ("chevron-right-medium", "chevron-right", 1.3),
];

/// Interface icons icon-rs doesn't have yet, drawn in its line style (`icons/`). A release of
/// icon-rs that adds one by the same name replaces it.
pub const STAND_INS: &[Icon] = &[
    stand_in("sun", "Light", include_str!("../icons/sun.svg")),
    stand_in("moon", "Dark", include_str!("../icons/moon.svg")),
    stand_in("contrast", "System", include_str!("../icons/contrast.svg")),
];

const fn stand_in(name: &'static str, title: &'static str, svg: &'static str) -> Icon {
    Icon {
        name,
        kind: icon_rs::Kind::Line,
        title,
        svg,
        svg_dark: svg,
    }
}

/// The icon by `name`: icon-rs's own, else a stand-in.
pub fn get(name: &str) -> Option<Icon> {
    icon_rs::get(name).or_else(|| STAND_INS.iter().find(|i| i.name == name).copied())
}

/// The icon `name` is drawn from, and its stroke weight (1 for icon-rs's own names).
pub fn source(name: &str) -> Option<(Icon, f32)> {
    match WEIGHTS.iter().find(|(n, ..)| *n == name) {
        Some(&(_, base, weight)) => get(base).map(|i| (i, weight)),
        None => get(name).map(|i| (i, 1.0)),
    }
}

/// The SVG of the icon's base layer and, for a tintable icon with an accent detail, of its
/// accent layer. `size` is the icon's size in logical px (the stroke weight depends on it).
pub fn layers(icon: &Icon, weight: f32, size: u32) -> (String, Option<String>) {
    if icon.kind.is_tintable() {
        let svg = with_stroke(icon.svg, stroke_scale(weight, size));
        let accent = icon.has_accent().then(|| accent_layer(&svg));
        (ink_layer(&svg), accent)
    } else {
        (icon.svg.to_owned(), None)
    }
}

/// How much heavier to draw a tintable icon: its weight variant, and a little heavier at 20 px
/// to match the stroke weight of the reference toolbar icons (about 2 px).
fn stroke_scale(weight: f32, size: u32) -> f32 {
    weight * if size == 20 { 1.1 } else { 1.0 }
}

/// Scales the root `stroke-width` (icon-rs draws tintable icons with the root width only).
fn with_stroke(svg: &str, scale: f32) -> String {
    if scale == 1.0 {
        return svg.to_owned();
    }
    let key = "stroke-width=\"";
    let start = svg.find(key).expect("root stroke-width") + key.len();
    let end = start + svg[start..].find('"').unwrap();
    let width: f32 = svg[start..end].parse().unwrap();
    format!("{}{}{}", &svg[..start], width * scale, &svg[end..])
}

/// The ink in white, with the accent detail removed.
fn ink_layer(svg: &str) -> String {
    let (ink, accent) = (Palette::LIGHT.ink.to_hex(), Palette::LIGHT.accent.to_hex());
    svg.replace(&format!("=\"{ink}\""), "=\"#ffffff\"")
        .replace(&format!("=\"{accent}\""), "=\"none\"")
}

/// Only the accent detail, in white.
fn accent_layer(svg: &str) -> String {
    let (ink, accent) = (Palette::LIGHT.ink.to_hex(), Palette::LIGHT.accent.to_hex());
    svg.replace(&format!("=\"{ink}\""), "=\"none\"")
        .replace(&format!("=\"{accent}\""), "=\"#ffffff\"")
}

/// The SVG rendered at `px` × `px`, as straight-alpha RGBA8.
pub fn rasterize(svg: &str, px: u32) -> Result<Vec<u8>, String> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let mut pixmap = tiny_skia::Pixmap::new(px, px).ok_or("empty icon size")?;
    let scale = px as f32 / tree.size().width().max(tree.size().height());
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect())
}
