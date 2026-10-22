//! icon-rs icons drawn by Proa UI's `Icon` component.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::components::IconDataRef;

/// An icon-rs line or glyph icon for Proa UI's `Icon`: its own elements, with the ink in
/// `currentColor` so it takes the text colour around it. Any accent detail keeps its colour.
#[derive(Clone, Copy)]
pub struct IconRs(pub icon_rs::Icon);

impl IconDataRef for IconRs {
    fn view_box(&self) -> Option<&'static str> {
        Some("0 0 24 24")
    }

    fn fill(&self) -> Option<&'static str> {
        Some("none")
    }

    fn stroke(&self) -> Option<&'static str> {
        Some("currentColor")
    }

    fn stroke_width(&self) -> Option<&'static str> {
        Some(match self.0.kind {
            icon_rs::Kind::Glyph => "2",
            _ => "1.75",
        })
    }

    fn stroke_linecap(&self) -> Option<&'static str> {
        Some("round")
    }

    fn stroke_linejoin(&self) -> Option<&'static str> {
        Some("round")
    }

    /// The elements inside the icon's `<svg>` root, with the ink in `currentColor`. Filled icons
    /// write the ink on their elements, not only the root, so each icon is recoloured once and
    /// kept for the life of the process.
    fn data(&self) -> &'static str {
        static RECOLOURED: OnceLock<Mutex<HashMap<&'static str, &'static str>>> = OnceLock::new();
        let mut cache = RECOLOURED
            .get_or_init(Default::default)
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        cache.entry(self.0.name).or_insert_with(|| {
            let svg: &'static str = Box::leak(self.0.current_color().into_boxed_str());
            let start = svg.find('>').map_or(0, |at| at + 1);
            let end = svg.rfind("</svg>").unwrap_or(svg.len());
            &svg[start..end]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_the_elements_inside_the_root() {
        let data = IconRs(icon_rs::PLATFORM_LINUX).data();
        assert!(data.trim_start().starts_with("<path"));
        assert!(!data.contains("<svg") && !data.contains("</svg>"));
        assert!(!data.contains("#262626"), "the ink is currentColor");
        assert!(
            data.contains(r#"fill="currentColor""#),
            "the solid shapes follow the text colour"
        );
    }
}
