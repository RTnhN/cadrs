//! Text that a click copies: a readout's value (Mass properties' volume, mass, …). Hovered, it
//! shows the pointer hand and "Click to copy"; clicked, its text goes to the clipboard with a
//! "Copied …" toast.

use bevy::prelude::*;

use crate::cursor::CursorKind;
use crate::splitter::HoverCursor;
use crate::theme::Theme;
use crate::tooltip::Tooltip;

pub struct CopyTextPlugin;

impl Plugin for CopyTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_click);
    }
}

/// A click on this `Text` copies it.
#[derive(Component, Debug, Clone, Copy)]
pub struct CopyOnClick;

/// What makes a `Text` copy on click.
pub fn copy_on_click() -> impl Bundle {
    (
        CopyOnClick,
        HoverCursor(CursorKind::Pointer),
        Tooltip::new("Click to copy"),
    )
}

fn on_click(
    click: On<Pointer<Click>>,
    q: Query<&Text, With<CopyOnClick>>,
    clipboard: Option<ResMut<bevy::clipboard::Clipboard>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if click.button != PointerButton::Primary {
        return;
    }
    let Ok(text) = q.get(click.entity) else {
        return;
    };
    if text.0.is_empty() {
        return;
    }
    if let Some(mut clip) = clipboard
        && let Err(e) = clip.set_text(text.0.clone())
    {
        warn!("clipboard: {e:?}");
    }
    crate::show_toast(&mut commands, &theme, format!("Copied {}", text.0));
}
