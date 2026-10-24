//! Light and dark color schemes.
//!
//! The interface is always built from [`Theme::light`] (the `Theme` resource), and so are the
//! colors code writes later: hover states, selected rows, fixed swatches. While the dark scheme
//! is on, [`apply_scheme`] maps every UI color it finds to the dark one just before the frame is
//! laid out, and puts the light color back when the scheme turns light again. A color is mapped
//! by what it paints ([`ColorRole`]: a fill, a raised surface, text and icons, a line):
//!
//! - A [`Theme`] token becomes its counterpart in [`Theme::dark`].
//! - Any other color keeps its hue and has its lightness moved along the curve the theme's own
//!   greys define for that role: light fills and lines turn dark, dark text turns light. Strong
//!   colors (a part's swatch, an accent) are left as they are; white text stays white.
//!
//! So nothing that draws the interface needs to know the scheme. What isn't a UI color (the 3D
//! view's background, its planes and sketches) reads [`ActiveTheme`], the theme of the scheme in
//! force, or picks its own pair with [`by_scheme`].
//!
//! The scheme is the user's [`SchemePreference`]: light, dark, or the system's, which is read
//! from the window (Windows, macOS) or the desktop's settings portal (Linux) and again whenever
//! the window gets the focus back.

use bevy::color::Oklcha;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::text::TextCursorStyle;
use bevy::window::{WindowFocused, WindowThemeChanged};

use crate::segmented::{Segmented, SegmentedSelect, SegmentedState};
use crate::style::Selected;
use crate::theme::{ColorRole, Theme};

pub struct ColorSchemePlugin;

impl Plugin for ColorSchemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Appearance>()
            .init_resource::<ActiveTheme>()
            .init_resource::<SchemeMap>()
            .init_resource::<SystemSchemeProbe>()
            .add_systems(PreUpdate, (detect_system_scheme, resolve_scheme).chain())
            .add_systems(Update, sync_toggles)
            .add_observer(on_toggle)
            .add_systems(
                PostUpdate,
                (
                    apply_scheme::<BackgroundColor>,
                    apply_scheme::<BorderColor>,
                    apply_scheme::<Outline>,
                    apply_scheme::<TextColor>,
                    apply_scheme::<ImageNode>,
                    apply_scheme::<TextCursorStyle>,
                    finish_scheme,
                )
                    .chain()
                    .after(crate::icon::sync_icon_layers)
                    .after(crate::style::apply_visuals)
                    .before(bevy::ui::UiSystems::Prepare),
            );
    }
}

/// A color scheme in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ColorScheme {
    #[default]
    Light,
    Dark,
}

/// The scheme the user asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum SchemePreference {
    /// Follow the system's light or dark setting.
    #[default]
    System,
    Light,
    Dark,
}

impl SchemePreference {
    /// The scheme this preference resolves to when the system's is `system` (light when the
    /// system doesn't say).
    pub fn resolve(self, system: Option<ColorScheme>) -> ColorScheme {
        match self {
            Self::System => system.unwrap_or_default(),
            Self::Light => ColorScheme::Light,
            Self::Dark => ColorScheme::Dark,
        }
    }
}

/// The preferred scheme, the system's, and the one in force.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    pub preference: SchemePreference,
    /// The system's scheme, when known.
    pub system: Option<ColorScheme>,
    /// Whether to ask the system at all. Off for scripted and headless runs, which must look
    /// the same on every machine.
    pub follow_system: bool,
    scheme: ColorScheme,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            preference: SchemePreference::System,
            system: None,
            follow_system: true,
            scheme: ColorScheme::Light,
        }
    }
}

impl Appearance {
    /// An appearance that never asks the system (a scripted run).
    pub fn without_system() -> Self {
        Self {
            follow_system: false,
            ..default()
        }
    }

    /// The scheme in force.
    pub fn scheme(&self) -> ColorScheme {
        self.scheme
    }

    pub fn is_dark(&self) -> bool {
        self.scheme == ColorScheme::Dark
    }
}

static DARK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the dark scheme is in force (it changes in `PreUpdate`, never mid-frame).
pub fn dark_in_force() -> bool {
    DARK.load(std::sync::atomic::Ordering::Relaxed)
}

/// `light` or `dark` by the scheme in force: for what is drawn outside the UI (gizmos, 3D
/// materials) by code that has no [`ActiveTheme`] at hand. A UI node's colors are mapped by
/// [`apply_scheme`] instead, so they are always given in light.
pub fn by_scheme(light: Color, dark: Color) -> Color {
    if dark_in_force() { dark } else { light }
}

/// The theme of the scheme in force, for colors that aren't UI nodes (the 3D view's background
/// and overlays). The UI itself is built from the `Theme` resource.
#[derive(Resource, Debug, Clone, Default, Deref)]
pub struct ActiveTheme(pub Theme);

/// How light-scheme colors become dark ones.
#[derive(Resource, Debug, Clone)]
pub struct SchemeMap {
    tokens: HashMap<(ColorRole, [u8; 3]), Srgba>,
    /// Per role, (light lightness, dark lightness) of the theme's greys, by light lightness.
    curves: HashMap<ColorRole, Vec<(f32, f32)>>,
}

impl Default for SchemeMap {
    fn default() -> Self {
        Self::new(&Theme::light(), &Theme::dark())
    }
}

/// Chroma below which a color counts as grey (OKLCH).
const GREY: f32 = 0.025;
/// Chroma above which a fill or line counts as a strong color and isn't changed.
const STRONG: f32 = 0.08;
/// Lightness below which an opaque grey fill or line counts as ink.
const INKY: f32 = 0.4;

fn key(c: Srgba) -> [u8; 3] {
    let [r, g, b, _] = c.to_u8_array();
    [r, g, b]
}

impl SchemeMap {
    /// Pairs the tokens of `light` with those of `dark`.
    pub fn new(light: &Theme, dark: &Theme) -> Self {
        let mut tokens = HashMap::default();
        let mut curves: HashMap<ColorRole, Vec<(f32, f32)>> = HashMap::default();
        for ((role, l), (_, d)) in light.colors().into_iter().zip(dark.colors()) {
            let (l, d) = (l.to_srgba(), d.to_srgba());
            if tokens.contains_key(&(role, key(l))) {
                continue;
            }
            tokens.insert((role, key(l)), d);
            let (lo, dk) = (Oklcha::from(l), Oklcha::from(d));
            if lo.chroma < GREY && dk.chroma < GREY {
                curves
                    .entry(role)
                    .or_default()
                    .push((lo.lightness, dk.lightness));
            }
        }
        for points in curves.values_mut() {
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            // Keep it monotonic (lighter in, darker out): drop points that would bend it back.
            let mut kept: Vec<(f32, f32)> = Vec::with_capacity(points.len());
            for &p in points.iter() {
                if kept
                    .last()
                    .is_none_or(|last| p.0 > last.0 + 1e-4 && p.1 <= last.1)
                {
                    kept.push(p);
                }
            }
            *points = kept;
        }
        Self { tokens, curves }
    }

    /// The dark-scheme color for `color` painted in `role`. Alpha is kept.
    pub fn dark(&self, color: Color, role: ColorRole) -> Color {
        let c = color.to_srgba();
        if c.alpha == 0.0 {
            return color;
        }
        if let Some(d) = self.tokens.get(&(role, key(c))) {
            return Color::Srgba(d.with_alpha(c.alpha));
        }
        let o = Oklcha::from(c);
        let Some(curve) = self.curves.get(&role).filter(|c| c.len() >= 2) else {
            return color;
        };
        let lightness = match role {
            ColorRole::Fill | ColorRole::Raised | ColorRole::Line
                if o.chroma < GREY && o.lightness < INKY && c.alpha > 0.99 =>
            {
                // Near-black drawn as a fill or line is ink (a marker, a dot, a rule): it turns
                // light like text. Translucent black (a shadow, a backdrop) stays.
                match self.curves.get(&ColorRole::Ink) {
                    Some(ink) if ink.len() >= 2 => along(ink, o.lightness),
                    _ => return color,
                }
            }
            ColorRole::Fill | ColorRole::Raised | ColorRole::Line => {
                if o.chroma > STRONG {
                    return color;
                }
                // Mid and dark fills (a badge, a shadow) already read on dark: blend into
                // themselves below the curve's darkest light grey.
                let start = curve[0].0;
                let floor = (start - 0.3).max(0.0);
                if o.lightness >= start {
                    along(curve, o.lightness)
                } else if o.lightness <= floor {
                    return color;
                } else {
                    let t = (o.lightness - floor) / (start - floor);
                    o.lightness + (curve[0].1 - o.lightness) * t
                }
            }
            ColorRole::Ink => {
                if o.lightness > 0.6 && o.chroma > STRONG {
                    return color;
                }
                // Light text is on a strong fill already (white on a primary button): fade out
                // the curve towards white.
                let end = curve[curve.len() - 1].0;
                let top = 0.97;
                if o.lightness <= end {
                    along(curve, o.lightness)
                } else if o.lightness >= top {
                    return color;
                } else {
                    let t = (o.lightness - end) / (top - end);
                    let at_end = curve[curve.len() - 1].1;
                    at_end + (o.lightness - at_end) * t
                }
            }
        };
        let out = Oklcha { lightness, ..o };
        Color::Srgba(Srgba::from(out).with_alpha(c.alpha))
    }

    /// `color` in the given scheme.
    pub fn resolve(&self, color: Color, role: ColorRole, scheme: ColorScheme) -> Color {
        match scheme {
            ColorScheme::Light => color,
            ColorScheme::Dark => self.dark(color, role),
        }
    }
}

/// Piecewise-linear interpolation, flat past the ends.
fn along(curve: &[(f32, f32)], x: f32) -> f32 {
    let (first, last) = (curve[0], curve[curve.len() - 1]);
    if x <= first.0 {
        return first.1;
    }
    if x >= last.0 {
        return last.1;
    }
    let i = curve.partition_point(|p| p.0 <= x);
    let (a, b) = (curve[i - 1], curve[i]);
    a.1 + (b.1 - a.1) * (x - a.0) / (b.0 - a.0)
}

// ---------------------------------------------------------------------------------------------
// The system's scheme

/// The scheme the desktop reports, read off the main thread (Linux asks the settings portal,
/// which can take a moment).
#[derive(Resource, Default)]
struct SystemSchemeProbe {
    pending: Option<std::sync::Mutex<std::sync::mpsc::Receiver<Option<ColorScheme>>>>,
    asked: bool,
}

fn from_window(theme: bevy::window::WindowTheme) -> ColorScheme {
    match theme {
        bevy::window::WindowTheme::Light => ColorScheme::Light,
        bevy::window::WindowTheme::Dark => ColorScheme::Dark,
    }
}

fn detect_system_scheme(
    mut appearance: ResMut<Appearance>,
    mut probe: ResMut<SystemSchemeProbe>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut theme_changed: MessageReader<WindowThemeChanged>,
    mut focused: MessageReader<WindowFocused>,
) {
    let refocused = focused.read().any(|f| f.focused);
    let changed = theme_changed.read().last().map(|e| from_window(e.theme));
    if !appearance.follow_system {
        return;
    }
    let mut system = appearance.system;
    if let Some(s) = changed {
        system = Some(s);
    }
    if !probe.asked || refocused {
        probe.asked = true;
        if let Some(t) = windows.single().ok().and_then(|w| w.window_theme) {
            system = Some(from_window(t));
        }
        if probe.pending.is_none() {
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(desktop_scheme());
            });
            probe.pending = Some(std::sync::Mutex::new(rx));
        }
    }
    if let Some(rx) = &probe.pending
        && let Ok(answer) = rx
            .lock()
            .map_err(drop)
            .and_then(|rx| rx.try_recv().map_err(drop))
    {
        probe.pending = None;
        if answer.is_some() {
            system = answer;
        }
    }
    if appearance.system != system {
        appearance.system = system;
    }
}

/// The desktop's preference where the window can't tell (Linux): the freedesktop settings
/// portal, then GNOME's own setting.
#[cfg(target_os = "linux")]
fn desktop_scheme() -> Option<ColorScheme> {
    use std::process::{Command, Stdio};
    let run = |cmd: &str, args: &[&str]| -> Option<String> {
        let out = Command::new(cmd)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    // `color-scheme`: 0 no preference, 1 dark, 2 light.
    if let Some(out) = run(
        "gdbus",
        &[
            "call",
            "--session",
            "--timeout=2",
            "--dest=org.freedesktop.portal.Desktop",
            "--object-path=/org/freedesktop/portal/desktop",
            "--method=org.freedesktop.portal.Settings.ReadOne",
            "org.freedesktop.appearance",
            "color-scheme",
        ],
    ) {
        match out
            .split_whitespace()
            .last()
            .map(|w| w.trim_matches(|c| ">,)".contains(c)))
        {
            Some("1") => return Some(ColorScheme::Dark),
            Some("2") => return Some(ColorScheme::Light),
            _ => {}
        }
    }
    let gtk = run(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "color-scheme"],
    )?;
    if gtk.contains("dark") {
        Some(ColorScheme::Dark)
    } else if gtk.contains("light") {
        Some(ColorScheme::Light)
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn desktop_scheme() -> Option<ColorScheme> {
    None
}

fn resolve_scheme(
    mut appearance: ResMut<Appearance>,
    mut active: ResMut<ActiveTheme>,
    mut clear: ResMut<ClearColor>,
) {
    if !appearance.is_changed() {
        return;
    }
    let scheme = appearance.preference.resolve(appearance.system);
    if appearance.scheme != scheme || appearance.is_added() {
        appearance.scheme = scheme;
        DARK.store(
            scheme == ColorScheme::Dark,
            std::sync::atomic::Ordering::Relaxed,
        );
        active.0 = match scheme {
            ColorScheme::Light => Theme::light(),
            ColorScheme::Dark => Theme::dark(),
        };
        clear.0 = active.viewport_background;
    }
}

// ---------------------------------------------------------------------------------------------
// Mapping the UI's colors

/// A UI component whose colors the scheme maps.
pub trait SchemeColors: Component<Mutability = bevy::ecs::component::Mutable> {
    type Value: Copy + PartialEq + Send + Sync + 'static;
    fn get(&self) -> Self::Value;
    fn set(&mut self, v: Self::Value);
    /// The value in the dark scheme. `raised` is a surface above the page (a menu or dialog).
    fn dark(v: Self::Value, map: &SchemeMap, raised: bool) -> Self::Value;
    /// When `current` is the mapped `dark` value with only its alpha changed (code fading the
    /// color it read back), the light value with that alpha.
    fn refade(
        _light: Self::Value,
        _dark: Self::Value,
        _current: Self::Value,
    ) -> Option<Self::Value> {
        None
    }
}

/// `light` with `current`'s alpha, if `current` is `dark` with another alpha.
fn refade(light: Color, dark: Color, current: Color) -> Option<Color> {
    let (d, c) = (dark.to_srgba(), current.to_srgba());
    (d.with_alpha(0.0) == c.with_alpha(0.0)).then(|| light.with_alpha(c.alpha))
}

impl SchemeColors for BackgroundColor {
    type Value = Color;
    fn refade(light: Color, dark: Color, current: Color) -> Option<Color> {
        refade(light, dark, current)
    }
    fn get(&self) -> Color {
        self.0
    }
    fn set(&mut self, v: Color) {
        self.0 = v;
    }
    fn dark(v: Color, map: &SchemeMap, raised: bool) -> Color {
        map.dark(
            v,
            if raised {
                ColorRole::Raised
            } else {
                ColorRole::Fill
            },
        )
    }
}

impl SchemeColors for BorderColor {
    type Value = [Color; 4];
    fn get(&self) -> [Color; 4] {
        [self.top, self.right, self.bottom, self.left]
    }
    fn set(&mut self, [top, right, bottom, left]: [Color; 4]) {
        *self = BorderColor {
            top,
            right,
            bottom,
            left,
        };
    }
    fn dark(v: [Color; 4], map: &SchemeMap, _: bool) -> [Color; 4] {
        v.map(|c| map.dark(c, ColorRole::Line))
    }
}

impl SchemeColors for Outline {
    type Value = Color;
    fn refade(light: Color, dark: Color, current: Color) -> Option<Color> {
        refade(light, dark, current)
    }
    fn get(&self) -> Color {
        self.color
    }
    fn set(&mut self, v: Color) {
        self.color = v;
    }
    fn dark(v: Color, map: &SchemeMap, _: bool) -> Color {
        map.dark(v, ColorRole::Line)
    }
}

impl SchemeColors for TextColor {
    type Value = Color;
    fn refade(light: Color, dark: Color, current: Color) -> Option<Color> {
        refade(light, dark, current)
    }
    fn get(&self) -> Color {
        self.0
    }
    fn set(&mut self, v: Color) {
        self.0 = v;
    }
    fn dark(v: Color, map: &SchemeMap, _: bool) -> Color {
        map.dark(v, ColorRole::Ink)
    }
}

impl SchemeColors for ImageNode {
    type Value = Color;
    fn refade(light: Color, dark: Color, current: Color) -> Option<Color> {
        refade(light, dark, current)
    }
    fn get(&self) -> Color {
        self.color
    }
    fn set(&mut self, v: Color) {
        self.color = v;
    }
    fn dark(v: Color, map: &SchemeMap, _: bool) -> Color {
        map.dark(v, ColorRole::Ink)
    }
}

impl SchemeColors for TextCursorStyle {
    type Value = (Color, Option<Color>);
    fn get(&self) -> Self::Value {
        (self.color, self.selected_text_color)
    }
    fn set(&mut self, (color, selected): Self::Value) {
        self.color = color;
        self.selected_text_color = selected;
    }
    fn dark((color, selected): Self::Value, map: &SchemeMap, _: bool) -> Self::Value {
        (map.dark(color, ColorRole::Ink), selected)
    }
}

/// The light color a component was given and the dark one the scheme wrote over it.
#[derive(Component, Debug, Clone, Copy)]
pub struct Mapped<T: SchemeColors> {
    light: T::Value,
    dark: T::Value,
}

/// The light color of `c`, as the interface gave it, for code that keeps a color it reads
/// back (to restore it later): with the dark scheme on, `c` holds the mapped color.
pub fn authored<T: SchemeColors>(c: &T, mapped: Option<&Mapped<T>>) -> T::Value {
    match mapped {
        Some(m) if c.get() == m.dark => m.light,
        _ => c.get(),
    }
}

/// Marks a UI node whose colors are already the scheme's own, such as
/// the scheme toggle's preview swatches: [`apply_scheme`] leaves it alone.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct KeepColors;

/// The scheme the UI's colors were last mapped for.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MappedScheme(ColorScheme);

type Raised = Or<(With<GlobalZIndex>, With<crate::dialog::DialogRoot>)>;
type Item<'a, T> = (Entity, Mut<'a, T>, Option<Mut<'a, Mapped<T>>>, bool);

/// Maps the colors of `T` to the scheme in force: the ones changed since the last frame, or all
/// of them when the scheme changes.
#[allow(clippy::type_complexity)]
pub fn apply_scheme<T: SchemeColors>(
    appearance: Res<Appearance>,
    mapped_for: Option<Res<MappedScheme>>,
    map: Res<SchemeMap>,
    mut q: ParamSet<(
        Query<(Entity, &mut T, Option<&mut Mapped<T>>, Has<KeepColors>), Changed<T>>,
        Query<(Entity, &mut T, Option<&mut Mapped<T>>, Has<KeepColors>)>,
    )>,
    raised: Query<(), Raised>,
    mut commands: Commands,
) {
    let scheme = appearance.scheme();
    let switched = mapped_for.is_none_or(|m| m.0 != scheme);
    let mut visit = |(e, mut c, mapped, keep): Item<T>| {
        if keep {
            return;
        }
        let current = c.get();
        // Our own write from the last pass: nothing new.
        let light = match &mapped {
            Some(m) if current == m.dark => {
                if !switched {
                    return;
                }
                m.light
            }
            Some(m) => T::refade(m.light, m.dark, current).unwrap_or(current),
            None => current,
        };
        match scheme {
            ColorScheme::Light => {
                if mapped.is_some() {
                    if current != light {
                        c.set(light);
                    }
                    commands.entity(e).remove::<Mapped<T>>();
                }
            }
            ColorScheme::Dark => {
                // Code that writes its light color every frame: the same dark one.
                let dark = match &mapped {
                    Some(m) if m.light == light && !switched => m.dark,
                    _ => T::dark(light, &map, raised.contains(e)),
                };
                if current != dark {
                    c.set(dark);
                }
                match mapped {
                    Some(mut m) => *m = Mapped { light, dark },
                    None => {
                        commands.entity(e).try_insert(Mapped::<T> { light, dark });
                    }
                }
            }
        }
    };
    if switched {
        q.p1().iter_mut().for_each(&mut visit);
    } else if scheme == ColorScheme::Dark {
        q.p0().iter_mut().for_each(&mut visit);
    }
}

fn finish_scheme(
    appearance: Res<Appearance>,
    mapped_for: Option<ResMut<MappedScheme>>,
    mut commands: Commands,
) {
    let scheme = appearance.scheme();
    match mapped_for {
        Some(mut m) if m.0 != scheme => m.0 = scheme,
        Some(_) => {}
        None => commands.insert_resource(MappedScheme(scheme)),
    }
}

// ---------------------------------------------------------------------------------------------
// The toggle

/// The scheme toggle of a top bar.
#[derive(Component, Debug, Clone, Copy)]
pub struct SchemeToggle;

const CHOICES: [SchemePreference; 3] = [
    SchemePreference::Light,
    SchemePreference::Dark,
    SchemePreference::System,
];

/// The light / dark / system toggle for a top bar: three icon segments named
/// `color-scheme-0` (light), `-1` (dark) and `-2` (the system's), the [`Appearance`]
/// preference's one on.
pub fn scheme_toggle(theme: &Theme) -> impl Bundle {
    (
        Segmented::new("color-scheme")
            .option("", Some("sun"), "Light")
            .option("", Some("moon"), "Dark")
            .option("", Some("contrast"), "Match the system")
            .selected(2)
            .size(22.0, 14.0)
            .build(theme),
        SchemeToggle,
    )
}

fn on_toggle(
    ev: On<SegmentedSelect>,
    toggles: Query<(), With<SchemeToggle>>,
    mut appearance: ResMut<Appearance>,
) {
    if toggles.contains(ev.entity)
        && let Some(&choice) = CHOICES.get(ev.index)
        && appearance.preference != choice
    {
        appearance.preference = choice;
    }
}

/// Shows the preference on every toggle (it can change elsewhere: the saved preferences, a
/// script).
fn sync_toggles(
    appearance: Res<Appearance>,
    mut toggles: Query<(&mut SegmentedState, &Children), With<SchemeToggle>>,
    added: Query<(), Added<SchemeToggle>>,
    selected: Query<Has<Selected>>,
    mut commands: Commands,
) {
    if !appearance.is_changed() && added.is_empty() {
        return;
    }
    let want = CHOICES
        .iter()
        .position(|&c| c == appearance.preference)
        .unwrap_or(2);
    for (mut state, children) in &mut toggles {
        if state.selected != want {
            state.selected = want;
        }
        for (i, c) in children.iter().enumerate() {
            match (selected.get(c), i == want) {
                (Ok(false), true) => {
                    commands.entity(c).insert(Selected);
                }
                (Ok(true), false) => {
                    commands.entity(c).remove::<Selected>();
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn l(c: Color) -> f32 {
        Oklcha::from(c.to_srgba()).lightness
    }

    #[test]
    fn tokens_map_to_their_dark_counterparts() {
        let (light, dark) = (Theme::light(), Theme::dark());
        let map = SchemeMap::default();
        assert_eq!(map.dark(light.background, ColorRole::Fill), dark.background);
        assert_eq!(map.dark(light.popover, ColorRole::Raised), dark.popover);
        assert_eq!(map.dark(light.foreground, ColorRole::Ink), dark.foreground);
        assert_eq!(map.dark(light.border, ColorRole::Line), dark.border);
        // White text on a primary button stays white.
        assert_eq!(
            map.dark(light.primary_foreground, ColorRole::Ink),
            dark.primary_foreground
        );
    }

    #[test]
    fn other_greys_follow_the_curve() {
        let map = SchemeMap::default();
        let dark = Theme::dark();
        // A light grey fill the theme doesn't name lands among the dark surfaces.
        let fill = map.dark(Color::srgb_u8(0xf4, 0xf4, 0xf4), ColorRole::Fill);
        assert!(l(fill) < 0.4, "{fill:?}");
        assert!(l(fill) >= l(dark.background) - 0.01);
        // Near-black text turns light, mid grey text stays mid.
        assert!(l(map.dark(Color::srgb_u8(0x44, 0x44, 0x44), ColorRole::Ink)) > 0.75);
        let mid = map.dark(Color::srgb_u8(0x99, 0x99, 0x99), ColorRole::Ink);
        assert!((0.4..0.7).contains(&l(mid)), "{mid:?}");
        // A light grey line turns into a dark one.
        assert!(l(map.dark(Color::srgb_u8(0xe2, 0xe2, 0xe2), ColorRole::Line)) < 0.45);
        // Opaque near-black fills and lines are ink; a backdrop stays dark.
        let light = Theme::light();
        assert!(l(map.dark(light.foreground, ColorRole::Fill)) > 0.75);
        assert!(l(map.dark(Color::BLACK, ColorRole::Line)) > 0.75);
        assert_eq!(map.dark(light.overlay, ColorRole::Fill), light.overlay);
    }

    #[test]
    fn strong_colors_and_alpha_are_kept() {
        let map = SchemeMap::default();
        let orange = Color::srgb(0.91, 0.66, 0.22);
        assert_eq!(map.dark(orange, ColorRole::Fill), orange);
        assert_eq!(map.dark(Color::NONE, ColorRole::Fill), Color::NONE);
        let veil = map.dark(Color::srgba(1.0, 1.0, 1.0, 0.4), ColorRole::Fill);
        assert_eq!(veil.alpha(), 0.4);
        assert_eq!(
            map.dark(Color::WHITE, ColorRole::Ink).to_srgba(),
            Srgba::WHITE
        );
    }

    #[test]
    fn the_preference_resolves() {
        assert_eq!(SchemePreference::System.resolve(None), ColorScheme::Light);
        assert_eq!(
            SchemePreference::System.resolve(Some(ColorScheme::Dark)),
            ColorScheme::Dark
        );
        assert_eq!(
            SchemePreference::Light.resolve(Some(ColorScheme::Dark)),
            ColorScheme::Light
        );
    }

    #[test]
    fn switching_maps_and_restores() {
        let mut app = App::new();
        app.insert_resource(Appearance::default())
            .init_resource::<SchemeMap>()
            .init_resource::<ActiveTheme>()
            .init_resource::<ClearColor>()
            .add_systems(PreUpdate, resolve_scheme)
            .add_systems(
                PostUpdate,
                (apply_scheme::<BackgroundColor>, finish_scheme).chain(),
            );
        let light = Theme::light();
        let e = app
            .world_mut()
            .spawn((Node::default(), BackgroundColor(light.background)))
            .id();
        app.update();
        let bg = |app: &App| app.world().get::<BackgroundColor>(e).unwrap().0;
        assert_eq!(bg(&app), light.background);
        app.world_mut().resource_mut::<Appearance>().preference = SchemePreference::Dark;
        app.update();
        assert_eq!(bg(&app), Theme::dark().background);
        // A fade of the color read back is a fade of the light one.
        let faded = Theme::dark().background.with_alpha(0.5);
        app.world_mut().get_mut::<BackgroundColor>(e).unwrap().0 = faded;
        app.update();
        assert_eq!(bg(&app), faded);
        // A light color written while dark is mapped too.
        app.world_mut().get_mut::<BackgroundColor>(e).unwrap().0 = light.list_hover;
        app.update();
        assert_eq!(bg(&app), Theme::dark().list_hover);
        app.update();
        assert_eq!(bg(&app), Theme::dark().list_hover);
        app.world_mut().resource_mut::<Appearance>().preference = SchemePreference::Light;
        app.update();
        assert_eq!(bg(&app), light.list_hover);
        assert_eq!(
            app.world().resource::<ClearColor>().0,
            light.viewport_background
        );
    }
}
