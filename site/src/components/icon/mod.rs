// SPDX-FileCopyrightText: 2026 Proa Labs, Inc.
// SPDX-License-Identifier: (MIT OR Apache-2.0) AND (ISC AND MIT)

//! Icon component - renders SVG icons
//!
//! This component provides an `IconData` struct that is layout-compatible
//! with `icondata_core::IconData`, so you can use icons from the `icondata`
//! crate ecosystem (Lucide, Heroicons, Font Awesome, etc.) by adding those
//! crates as dependencies.
//!
//! # Usage with icondata crate
//!
//! ```ignore
//! // In Cargo.toml: icondata_lu = "0.1"
//! use icondata_lu::LuChevronDown;
//! use crate::components::{Icon, IconSize};
//!
//! html! {
//!     {Icon { icon: LuChevronDown, size: Some(IconSize::Md), class: None }}
//! }
//! ```
//!
//! # Usage with custom icons
//!
//! ```ignore
//! use crate::components::{Icon, IconData, IconSize};
//!
//! static MY_ICON: IconData = IconData {
//!     style: None, x: None, y: None, width: None, height: None,
//!     view_box: Some("0 0 24 24"),
//!     stroke_linecap: Some("round"),
//!     stroke_linejoin: Some("round"),
//!     stroke_width: Some("2"),
//!     stroke: Some("currentColor"),
//!     fill: Some("none"),
//!     data: r#"<path d="m6 9 6 6 6-6"/>"#,
//! };
//! ```

#[allow(clippy::module_inception)]
mod icon;
mod md;

pub use icon::{Icon, IconData, IconDataRef, IconSize};
