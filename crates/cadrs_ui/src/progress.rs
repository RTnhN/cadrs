//! A progress bar, modeled on gpui-component's `Progress`: a rounded grey track with a blue fill
//! from the left. Set [`ProgressBar::value`] on the spawned entity to move it.

use bevy::prelude::*;

use crate::theme::Theme;

pub struct ProgressPlugin;

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_fill);
    }
}

/// How far along, 0 to 1. On the track; its fill follows.
#[derive(Component, Debug, Clone, Copy)]
pub struct ProgressBar {
    pub value: f32,
}

/// The track's fill.
#[derive(Component, Debug, Clone, Copy)]
struct ProgressFill;

/// Builder for a progress bar.
pub struct Progress {
    name: String,
    value: f32,
    height: f32,
}

impl Progress {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: 0.0,
            height: 6.0,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    pub fn build(self, theme: &Theme) -> impl Bundle {
        let value = self.value.clamp(0.0, 1.0);
        (
            Name::new(self.name),
            ProgressBar { value },
            Node {
                flex_grow: 1.0,
                height: Val::Px(self.height),
                border_radius: BorderRadius::MAX,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(theme.secondary),
            children![(
                ProgressFill,
                Node {
                    width: Val::Percent(value * 100.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(theme.primary),
            )],
        )
    }
}

fn sync_fill(
    bars: Query<(&ProgressBar, &Children), Changed<ProgressBar>>,
    mut fills: Query<&mut Node, With<ProgressFill>>,
) {
    for (bar, children) in &bars {
        for &c in children {
            if let Ok(mut n) = fills.get_mut(c) {
                n.width = Val::Percent(bar.value.clamp(0.0, 1.0) * 100.0);
            }
        }
    }
}
