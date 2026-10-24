//! The CAM Studio dialogs as data: each dialog lists its [`Field`]s (number, text, select,
//! checkbox, a pick list fed by the view, buttons, notes, section headings and a tab strip),
//! [`spawn_fields`] builds them in a feature dialog card, and every change comes back as a
//! [`FieldChange`] (key and [`Value`]) for the dialog's draft. Widgets are named
//! `cam-f-<key>`, so scenarios and tests find them by key.

use bevy::prelude::*;
use bevy::text::FontWeight;
use cadrs_ui::prelude::*;
use cadrs_ui::{Button, Checkbox, NumberField, Select, SelectionList, TabStrip};

/// What a number means (its suffix and how text reads back).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Mm,
    /// mm/min.
    Feed,
    Deg,
    Rpm,
    Percent,
    Seconds,
    /// A plain number.
    None,
}

impl Unit {
    pub fn suffix(self) -> &'static str {
        match self {
            Unit::Mm => " mm",
            Unit::Feed => " mm/min",
            Unit::Deg => "°",
            Unit::Rpm => " rpm",
            Unit::Percent => " %",
            Unit::Seconds => " s",
            Unit::None => "",
        }
    }
}

/// A dialog field.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
    /// A bold heading over the fields after it.
    Section(String),
    /// A small grey note (or a red one when `error`).
    Note { text: String, error: bool },
    Number {
        key: String,
        label: String,
        value: f64,
        unit: Unit,
    },
    /// A whole number (a tool's T number, a count).
    Count {
        key: String,
        label: String,
        value: i64,
    },
    Text {
        key: String,
        label: String,
        value: String,
    },
    Select {
        key: String,
        label: String,
        options: Vec<String>,
        selected: usize,
    },
    Check {
        key: String,
        label: String,
        value: bool,
    },
    /// Items picked in the view; `active` when it takes the next pick.
    Picks {
        key: String,
        placeholder: String,
        items: Vec<String>,
        active: bool,
    },
    Button {
        key: String,
        label: String,
        icon: &'static str,
    },
    /// A tab strip over the fields (the toolpath's Tool path | Link | Misc).
    Tabs {
        key: String,
        labels: Vec<String>,
        selected: usize,
    },
}

impl Field {
    pub fn number(key: &str, label: &str, value: f64, unit: Unit) -> Field {
        Field::Number {
            key: key.into(),
            label: label.into(),
            value,
            unit,
        }
    }

    pub fn count(key: &str, label: &str, value: i64) -> Field {
        Field::Count {
            key: key.into(),
            label: label.into(),
            value,
        }
    }

    pub fn text(key: &str, label: &str, value: &str) -> Field {
        Field::Text {
            key: key.into(),
            label: label.into(),
            value: value.into(),
        }
    }

    pub fn select<S: Into<String>>(
        key: &str,
        label: &str,
        options: impl IntoIterator<Item = S>,
        selected: usize,
    ) -> Field {
        Field::Select {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().map(Into::into).collect(),
            selected,
        }
    }

    pub fn check(key: &str, label: &str, value: bool) -> Field {
        Field::Check {
            key: key.into(),
            label: label.into(),
            value,
        }
    }

    pub fn section(text: &str) -> Field {
        Field::Section(text.into())
    }

    pub fn note(text: impl Into<String>) -> Field {
        Field::Note {
            text: text.into(),
            error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Field {
        Field::Note {
            text: text.into(),
            error: true,
        }
    }

    pub fn button(key: &str, label: &str, icon: &'static str) -> Field {
        Field::Button {
            key: key.into(),
            label: label.into(),
            icon,
        }
    }

    pub fn picks(key: &str, placeholder: &str, items: Vec<String>, active: bool) -> Field {
        Field::Picks {
            key: key.into(),
            placeholder: placeholder.into(),
            items,
            active,
        }
    }
}

/// A changed value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Text(String),
    Index(usize),
    Bool(bool),
    /// A button was clicked, or a pick list was clicked (to take the next picks).
    Activate,
    /// A pick list's item was removed.
    Remove(usize),
}

/// A field changed (the dialog's draft takes it).
#[derive(Message, Debug, Clone, PartialEq)]
pub struct FieldChange {
    pub key: String,
    pub value: Value,
}

/// The widget name of a key.
pub fn widget(key: &str) -> String {
    format!("cam-f-{key}")
}

/// The key of a widget name.
pub fn key_of(name: &str) -> Option<&str> {
    name.strip_prefix("cam-f-")
}

/// "6", "0.25", "12000": a value without trailing zeros.
pub fn fmt_num(v: f64, unit: Unit) -> String {
    let decimals = match unit {
        Unit::Rpm | Unit::Feed => 0,
        Unit::Percent | Unit::Deg => 2,
        _ => 4,
    };
    let s = format!("{v:.decimals$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    };
    format!("{}{}", if s == "-0" { "0" } else { &s }, unit.suffix())
}

/// Reads a number typed into a field: a plain number, optionally with its unit; lengths can
/// also be given in inches ("0.25 in", "1/4in").
pub fn parse_num(text: &str, unit: Unit) -> Option<f64> {
    let t = text.trim().to_lowercase();
    let (body, scale) = if unit == Unit::Mm && (t.ends_with("in") || t.ends_with('"')) {
        (t.trim_end_matches("in").trim_end_matches('"').trim(), 25.4)
    } else {
        let mut b = t.as_str();
        for s in ["mm/min", "mm", "rpm", "°", "deg", "%", "s", "sec", "min"] {
            if let Some(x) = b.strip_suffix(s) {
                b = x.trim();
                break;
            }
        }
        (b, 1.0)
    };
    let v = match body.split_once('/') {
        Some((a, b)) => a.trim().parse::<f64>().ok()? / b.trim().parse::<f64>().ok()?,
        None => body.parse::<f64>().ok()?,
    };
    v.is_finite().then_some(v * scale)
}

const LABEL_W: f32 = 112.0;

fn row(b: &mut ChildSpawner, t: &Theme, label: &str, content: impl Bundle) {
    b.spawn(Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(6.0),
        min_height: Val::Px(26.0),
        width: Val::Percent(100.0),
        ..default()
    })
    .with_children(|r| {
        r.spawn((
            t.text(label, 12.0, FontWeight::NORMAL, t.muted_foreground),
            Node {
                width: Val::Px(LABEL_W),
                flex_shrink: 0.0,
                ..default()
            },
        ));
        r.spawn(Node {
            flex_grow: 1.0,
            min_width: Val::Px(0.0),
            ..default()
        })
        .with_children(|c| {
            c.spawn(content);
        });
    });
}

/// Builds the fields into a dialog body.
pub fn spawn_fields(b: &mut ChildSpawner, t: &Theme, fields: &[Field]) {
    for f in fields {
        match f {
            Field::Section(text) => {
                b.spawn((
                    t.text(text, 12.0, FontWeight::BOLD, t.foreground),
                    Node {
                        margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(8.0), Val::Px(2.0)),
                        ..default()
                    },
                ));
            }
            Field::Note { text, error } => {
                let c = if *error { t.danger } else { t.muted_foreground };
                b.spawn((
                    t.text(text, 11.0, FontWeight::NORMAL, c),
                    Node {
                        margin: UiRect::vertical(Val::Px(2.0)),
                        max_width: Val::Percent(100.0),
                        ..default()
                    },
                ));
            }
            Field::Number {
                key,
                label,
                value,
                unit,
            } => {
                b.spawn(
                    NumberField::new(widget(key), label.clone())
                        .text(fmt_num(*value, *unit))
                        .label_width(LABEL_W)
                        .label_size(12.0)
                        .build(t),
                );
            }
            Field::Count { key, label, value } => {
                b.spawn(
                    NumberField::new(widget(key), label.clone())
                        .text(value.to_string())
                        .label_width(LABEL_W)
                        .label_size(12.0)
                        .build(t),
                );
            }
            Field::Text { key, label, value } => {
                row(
                    b,
                    t,
                    label,
                    TextInput::new(widget(key))
                        .value(value.clone())
                        .select_all_on_focus()
                        .width(Val::Percent(100.0))
                        .height(24.0)
                        .build(t),
                );
            }
            Field::Select {
                key,
                label,
                options,
                selected,
            } => {
                let mut s = Select::new(widget(key))
                    .bordered()
                    .width(Val::Percent(100.0));
                for o in options {
                    s = s.option(o.clone(), true);
                }
                row(b, t, label, s.selected(*selected).build(t));
            }
            Field::Check { key, label, value } => {
                b.spawn(
                    Checkbox::new(widget(key))
                        .label(label.clone())
                        .checked(*value)
                        .build(t),
                );
            }
            Field::Picks {
                key,
                placeholder,
                items,
                active,
            } => {
                b.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        margin: UiRect::vertical(Val::Px(3.0)),
                        ..default()
                    },
                    children![
                        SelectionList::new(widget(key))
                            .placeholder(placeholder.clone())
                            .items(items.clone())
                            .active(*active)
                            .build(t)
                    ],
                ));
            }
            Field::Button { key, label, icon } => {
                b.spawn(
                    Button::new(widget(key))
                        .label(label.clone())
                        .icon(*icon)
                        .outline()
                        .build(t),
                );
            }
            Field::Tabs {
                key,
                labels,
                selected,
            } => {
                let mut s = TabStrip::new(widget(key)).compact();
                for l in labels {
                    s = s.tab(l.clone());
                }
                b.spawn(s.selected(*selected).build(t));
            }
        }
    }
}

/// The number field's unit, by key, while the dialog is open (to read typed text back).
#[derive(Resource, Debug, Default, Clone)]
pub struct FieldUnits(pub std::collections::HashMap<String, Unit>);

impl FieldUnits {
    pub fn of(fields: &[Field]) -> FieldUnits {
        FieldUnits(
            fields
                .iter()
                .filter_map(|f| match f {
                    Field::Number { key, unit, .. } => Some((key.clone(), *unit)),
                    Field::Count { key, .. } => Some((key.clone(), Unit::None)),
                    _ => None,
                })
                .collect(),
        )
    }
}

pub struct FormPlugin;

impl Plugin for FormPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FieldChange>()
            .init_resource::<FieldUnits>()
            .add_observer(on_number)
            .add_observer(on_select)
            .add_observer(on_check)
            .add_observer(on_text)
            .add_observer(on_activate)
            .add_observer(on_list_activate)
            .add_observer(on_list_remove)
            .add_observer(on_tab);
    }
}

fn key_for(q: &Query<&Name>, e: Entity) -> Option<String> {
    q.get(e)
        .ok()
        .and_then(|n| key_of(n.as_str()).map(String::from))
}

fn on_number(
    ev: On<cadrs_ui::NumberFieldCommit>,
    q: Query<&Name>,
    units: Res<FieldUnits>,
    mut out: MessageWriter<FieldChange>,
) {
    let Some(key) = key_for(&q, ev.entity) else {
        return;
    };
    let unit = units.0.get(&key).copied().unwrap_or(Unit::None);
    if let Some(v) = parse_num(&ev.text, unit) {
        out.write(FieldChange {
            key,
            value: Value::Number(v),
        });
    } else {
        // Unreadable: the dialog shows the old value again.
        out.write(FieldChange {
            key,
            value: Value::Text(ev.text.clone()),
        });
    }
}

fn on_select(ev: On<cadrs_ui::SelectChange>, q: Query<&Name>, mut out: MessageWriter<FieldChange>) {
    if let Some(key) = key_for(&q, ev.entity) {
        out.write(FieldChange {
            key,
            value: Value::Index(ev.index),
        });
    }
}

fn on_check(
    ev: On<cadrs_ui::CheckboxChange>,
    q: Query<&Name>,
    mut out: MessageWriter<FieldChange>,
) {
    if let Some(key) = key_for(&q, ev.entity) {
        out.write(FieldChange {
            key,
            value: Value::Bool(ev.checked),
        });
    }
}

fn on_text(ev: On<cadrs_ui::TextSubmit>, q: Query<&Name>, mut out: MessageWriter<FieldChange>) {
    // (The event bubbles from the inner `-field` entity; take it once, there.)
    let Ok(n) = q.get(ev.entity) else { return };
    let Some(key) = key_of(n.as_str()).and_then(|k| k.strip_suffix("-field")) else {
        return;
    };
    out.write(FieldChange {
        key: key.to_string(),
        value: Value::Text(ev.value.clone()),
    });
}

fn on_activate(
    ev: On<bevy::ui_widgets::Activate>,
    q: Query<&Name>,
    mut out: MessageWriter<FieldChange>,
) {
    if let Ok(n) = q.get(ev.entity)
        && let Some(key) = key_of(n.as_str())
    {
        out.write(FieldChange {
            key: key.to_string(),
            value: Value::Activate,
        });
    }
}

fn on_list_activate(
    ev: On<cadrs_ui::SelectionListActivate>,
    q: Query<&Name>,
    mut out: MessageWriter<FieldChange>,
) {
    if let Some(key) = key_for(&q, ev.entity) {
        out.write(FieldChange {
            key,
            value: Value::Activate,
        });
    }
}

fn on_list_remove(
    ev: On<cadrs_ui::SelectionListRemove>,
    q: Query<&Name>,
    mut out: MessageWriter<FieldChange>,
) {
    if let Some(key) = key_for(&q, ev.entity) {
        out.write(FieldChange {
            key,
            value: Value::Remove(ev.index),
        });
    }
}

fn on_tab(ev: On<cadrs_ui::TabStripSelect>, q: Query<&Name>, mut out: MessageWriter<FieldChange>) {
    if let Some(key) = key_for(&q, ev.entity) {
        out.write(FieldChange {
            key,
            value: Value::Index(ev.index),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_back() {
        assert_eq!(parse_num("6 mm", Unit::Mm), Some(6.0));
        assert_eq!(parse_num("0.25 in", Unit::Mm), Some(6.35));
        assert_eq!(parse_num("1/4in", Unit::Mm), Some(6.35));
        assert_eq!(parse_num("1200 mm/min", Unit::Feed), Some(1200.0));
        assert_eq!(parse_num("45°", Unit::Deg), Some(45.0));
        assert_eq!(parse_num("x", Unit::Mm), None);
        assert_eq!(fmt_num(6.0, Unit::Mm), "6 mm");
        assert_eq!(fmt_num(0.25, Unit::Mm), "0.25 mm");
        assert_eq!(fmt_num(12000.4, Unit::Rpm), "12000 rpm");
    }
}
