// SPDX-FileCopyrightText: 2026 Proa Labs, Inc.
// SPDX-License-Identifier: (MIT OR Apache-2.0) AND (ISC AND MIT)

//! Icon component implementation
#![allow(unused_must_use)]

use proa_core::{ClassList, UnsafeRaw, WebContext, WebRenderSync, WriteBuf};
use proa_macros::html_sync;
/// Trait for types that can provide SVG icon data.
///
/// This trait is implemented for `IconData` and can be implemented for
/// `icondata_core::IconData` to allow direct use of icons from the icondata ecosystem.
///
/// # Example implementing for icondata
///
/// ```ignore
/// use icondata_core::IconData as ExtIconData;
/// use crate::components::IconDataRef;
///
/// impl IconDataRef for ExtIconData {
///     fn view_box(&self) -> Option<&'static str> { self.view_box }
///     fn fill(&self) -> Option<&'static str> { self.fill }
///     fn stroke(&self) -> Option<&'static str> { self.stroke }
///     fn stroke_width(&self) -> Option<&'static str> { self.stroke_width }
///     fn stroke_linecap(&self) -> Option<&'static str> { self.stroke_linecap }
///     fn stroke_linejoin(&self) -> Option<&'static str> { self.stroke_linejoin }
///     fn data(&self) -> &'static str { self.data }
/// }
/// ```
pub trait IconDataRef {
    /// SVG viewBox attribute
    fn view_box(&self) -> Option<&'static str>;
    /// SVG fill attribute
    fn fill(&self) -> Option<&'static str>;
    /// SVG stroke attribute
    fn stroke(&self) -> Option<&'static str>;
    /// SVG stroke-width attribute
    fn stroke_width(&self) -> Option<&'static str>;
    /// SVG stroke-linecap attribute
    fn stroke_linecap(&self) -> Option<&'static str>;
    /// SVG stroke-linejoin attribute
    fn stroke_linejoin(&self) -> Option<&'static str>;
    /// Inner SVG content (paths, circles, etc.)
    fn data(&self) -> &'static str;
}
/// SVG icon data structure.
///
/// This struct is layout-compatible with `icondata_core::IconData`. You can either:
/// 1. Use this struct directly with helper constructors like `IconData::lucide()`
/// 2. Use icons from the `icondata` crate by implementing `IconDataRef` for `icondata_core::IconData`
///
/// # Example with Lucide-style icon
///
/// ```ignore
/// use crate::components::{Icon, IconData, IconSize};
///
/// // Simple: just provide the SVG path data
/// static CHEVRON: IconData = IconData::lucide(r#"<path d="m6 9 6 6 6-6"/>"#);
/// ```
///
/// # Example with custom icon (full control)
///
/// ```ignore
/// use crate::components::{Icon, IconData, IconSize};
///
/// static MY_ICON: IconData = IconData {
///     view_box: Some("0 0 24 24"),
///     fill: Some("currentColor"),
///     data: r#"<path d="..."/>"#,
///     ..IconData::EMPTY
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconData {
    /// CSS style attribute
    pub style: Option<&'static str>,
    /// SVG x attribute
    pub x: Option<&'static str>,
    /// SVG y attribute
    pub y: Option<&'static str>,
    /// SVG width attribute
    pub width: Option<&'static str>,
    /// SVG height attribute
    pub height: Option<&'static str>,
    /// SVG viewBox attribute
    pub view_box: Option<&'static str>,
    /// SVG stroke-linecap attribute
    pub stroke_linecap: Option<&'static str>,
    /// SVG stroke-linejoin attribute
    pub stroke_linejoin: Option<&'static str>,
    /// SVG stroke-width attribute
    pub stroke_width: Option<&'static str>,
    /// SVG stroke attribute
    pub stroke: Option<&'static str>,
    /// SVG fill attribute
    pub fill: Option<&'static str>,
    /// Inner SVG content (paths, circles, etc.)
    pub data: &'static str,
}
impl IconDataRef for IconData {
    fn view_box(&self) -> Option<&'static str> {
        self.view_box
    }
    fn fill(&self) -> Option<&'static str> {
        self.fill
    }
    fn stroke(&self) -> Option<&'static str> {
        self.stroke
    }
    fn stroke_width(&self) -> Option<&'static str> {
        self.stroke_width
    }
    fn stroke_linecap(&self) -> Option<&'static str> {
        self.stroke_linecap
    }
    fn stroke_linejoin(&self) -> Option<&'static str> {
        self.stroke_linejoin
    }
    fn data(&self) -> &'static str {
        self.data
    }
}
impl IconDataRef for &'static IconData {
    fn view_box(&self) -> Option<&'static str> {
        (*self).view_box()
    }
    fn fill(&self) -> Option<&'static str> {
        (*self).fill()
    }
    fn stroke(&self) -> Option<&'static str> {
        (*self).stroke()
    }
    fn stroke_width(&self) -> Option<&'static str> {
        (*self).stroke_width()
    }
    fn stroke_linecap(&self) -> Option<&'static str> {
        (*self).stroke_linecap()
    }
    fn stroke_linejoin(&self) -> Option<&'static str> {
        (*self).stroke_linejoin()
    }
    fn data(&self) -> &'static str {
        (*self).data()
    }
}

impl IconData {
    /// Empty icon data - use with struct update syntax: `..IconData::EMPTY`
    pub const EMPTY: Self = Self {
        style: None,
        x: None,
        y: None,
        width: None,
        height: None,
        view_box: None,
        stroke_linecap: None,
        stroke_linejoin: None,
        stroke_width: None,
        stroke: None,
        fill: None,
        data: "",
    };
    /// Create a Lucide-style icon with standard defaults.
    ///
    /// This sets up the icon with:
    /// - viewBox: "0 0 24 24"
    /// - stroke: "currentColor"
    /// - fill: "none"
    /// - stroke-width: "2"
    /// - stroke-linecap: "round"
    /// - stroke-linejoin: "round"
    ///
    /// # Example
    ///
    /// ```ignore
    /// use crate::components::IconData;
    ///
    /// static LU_CHEVRON_DOWN: IconData = IconData::lucide(r#"<path d="m6 9 6 6 6-6"/>"#);
    /// static LU_CHECK: IconData = IconData::lucide(r#"<path d="M20 6 9 17l-5-5"/>"#);
    /// static LU_X: IconData = IconData::lucide(r#"<path d="M18 6 6 18"/><path d="m6 6 12 12"/>"#);
    /// ```
    pub const fn lucide(data: &'static str) -> Self {
        Self {
            style: None,
            x: None,
            y: None,
            width: None,
            height: None,
            view_box: Some("0 0 24 24"),
            stroke_linecap: Some("round"),
            stroke_linejoin: Some("round"),
            stroke_width: Some("2"),
            stroke: Some("currentColor"),
            fill: Some("none"),
            data,
        }
    }
    /// Create a filled icon (no stroke, uses fill).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use crate::components::IconData;
    ///
    /// static CIRCLE_FILLED: IconData = IconData::filled(r#"<circle cx="12" cy="12" r="10"/>"#);
    /// ```
    pub const fn filled(data: &'static str) -> Self {
        Self {
            style: None,
            x: None,
            y: None,
            width: None,
            height: None,
            view_box: Some("0 0 24 24"),
            stroke_linecap: None,
            stroke_linejoin: None,
            stroke_width: None,
            stroke: None,
            fill: Some("currentColor"),
            data,
        }
    }
}
/// Icon size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconSize {
    /// Extra small - 12px
    Xs,
    /// Small - 16px
    Sm,
    /// Medium - 20px (default)
    #[default]
    Md,
    /// Large - 24px
    Lg,
    /// Extra large - 32px
    Xl,
}
impl IconSize {
    /// Get the size in pixels as a string
    pub const fn as_px(self) -> &'static str {
        match self {
            IconSize::Xs => "12",
            IconSize::Sm => "16",
            IconSize::Md => "20",
            IconSize::Lg => "24",
            IconSize::Xl => "32",
        }
    }
    /// Get the Tailwind size class
    pub const fn as_class(self) -> &'static str {
        match self {
            IconSize::Xs => "size-3",
            IconSize::Sm => "size-4",
            IconSize::Md => "size-5",
            IconSize::Lg => "size-6",
            IconSize::Xl => "size-8",
        }
    }
}
/// Icon component - renders SVG icons
///
/// Works with any type implementing `IconDataRef`, including:
/// - `proa_ui::IconData` - for custom icons or using `IconData::lucide()`
/// - `icondata_core::IconData` - after implementing `IconDataRef` for it
///
/// # Example with IconData
///
/// ```ignore
/// use crate::components::{Icon, IconData, IconSize};
///
/// static CHEVRON: IconData = IconData::lucide(r#"<path d="m6 9 6 6 6-6"/>"#);
///
/// html_sync! {
///     {Icon { icon: &CHEVRON, size: Some(IconSize::Md), class: None }}
/// }
/// ```
///
/// # Example with icondata crate
///
/// ```ignore
/// use icondata_lu::LuUser;
/// use crate::components::{Icon, IconSize};
///
/// // After implementing IconDataRef for icondata_core::IconData
/// html_sync! {
///     {Icon { icon: LuUser, size: Some(IconSize::Md), class: None }}
/// }
/// ```
pub struct Icon<I: IconDataRef> {
    /// The icon data (implements IconDataRef)
    pub icon: I,
    /// Size of the icon
    pub size: Option<IconSize>,
    /// Additional CSS classes
    pub class: Option<&'static str>,
}
impl Default for Icon<&'static IconData> {
    fn default() -> Self {
        static EMPTY_ICON: IconData = IconData {
            style: None,
            x: None,
            y: None,
            width: None,
            height: None,
            view_box: Some("0 0 24 24"),
            stroke_linecap: None,
            stroke_linejoin: None,
            stroke_width: None,
            stroke: None,
            fill: None,
            data: "",
        };
        Self {
            icon: &EMPTY_ICON,
            size: None,
            class: None,
        }
    }
}
impl<I: IconDataRef, B: WriteBuf, L: ::proa_core::DataLoader> WebRenderSync<L, B> for Icon<I> {
    fn render(
        self,
        cx: &mut WebContext<L, B>,
    ) -> Result<(), proa_core::shared::write_buf::WriteError> {
        let size = self.size.unwrap_or_default();
        let size_px = size.as_px();
        let classes = ClassList::<2>::new().add("shrink-0").add_opt(self.class);
        let view_box = self.icon.view_box().unwrap_or("0 0 24 24");
        let fill = self.icon.fill().unwrap_or("none");
        let stroke = self.icon.stroke().unwrap_or("currentColor");
        if let Some(stroke_width) = self.icon.stroke_width() {
            if let Some(stroke_linecap) = self.icon.stroke_linecap() {
                if let Some(stroke_linejoin) = self.icon.stroke_linejoin() {
                    html_sync! {
                        <svg
                            data-icon
                            xmlns="http://www.w3.org/2000/svg"
                            width={size_px}
                            height={size_px}
                            viewBox={view_box}
                            fill={fill}
                            stroke={stroke}
                            stroke-width={stroke_width}
                            stroke-linecap={stroke_linecap}
                            stroke-linejoin={stroke_linejoin}
                            class={classes}
                        >
                            {proa_core::foreign_content_ok(UnsafeRaw(self.icon.data()))} // proa-perf-ok: trusted static markup emitted through the low-level writer
                        </svg>
                    }
                    .render(cx)?;
                    return Ok(());
                }
            }
        }
        html_sync! {
            <svg
                data-icon
                xmlns="http://www.w3.org/2000/svg"
                width={size_px}
                height={size_px}
                viewBox={view_box}
                fill={fill}
                stroke={stroke}
                class={classes}
            >
                {proa_core::foreign_content_ok(UnsafeRaw(self.icon.data()))} // proa-perf-ok: trusted static markup emitted through the low-level writer
            </svg>
        }
        .render(cx)
    }
}
