//! icon-rs icons drawn by Proa UI's `Icon` component.

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

    /// The elements inside the icon's `<svg>` root, borrowed from its embedded source.
    fn data(&self) -> &'static str {
        let svg = self.0.svg;
        let start = svg.find('>').map_or(0, |at| at + 1);
        let end = svg.rfind("</svg>").unwrap_or(svg.len());
        &svg[start..end]
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
        assert!(
            !data.contains("#262626"),
            "the ink comes from the root's currentColor"
        );
    }
}
