//! Layout-aware keyboard shortcuts (issue #32).
//!
//! Bevy's [`KeyCode`] is the physical key, named after its place on a US QWERTY keyboard, so
//! matching shortcuts on it breaks other layouts: on AZERTY the key labelled Z sends `KeyW`.
//! [`ShortcutKey::shortcut_code`] gives the key a shortcut should match instead, still as a
//! [`KeyCode`] so handlers keep matching `KeyCode::KeyZ`:
//!
//! - a letter or US punctuation the layout types (`z`, `[`, `?`) → that character's US key
//!   (AZERTY Z → `KeyZ`); winit applies every modifier but Ctrl, so Ctrl+Z still reads `z`;
//! - the number row and numpad → the physical key (AZERTY types `&é"'` there unshifted, but
//!   Shift+1 should still be the front view);
//! - a character outside ASCII (Cyrillic, Greek, …) → the physical key, so non-Latin layouts
//!   keep their QWERTY positions, as browsers do;
//! - named keys (arrows, Enter, Escape, F-keys, modifiers) → the physical key.

use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};

pub trait ShortcutKey {
    /// The key a shortcut should match on the user's keyboard layout.
    fn shortcut_code(&self) -> KeyCode;
}

impl ShortcutKey for KeyboardInput {
    fn shortcut_code(&self) -> KeyCode {
        shortcut_code(self.key_code, &self.logical_key)
    }
}

/// [`ShortcutKey::shortcut_code`] of a physical key and the logical key it typed.
pub fn shortcut_code(physical: KeyCode, logical: &Key) -> KeyCode {
    if is_number_key(physical) {
        return physical;
    }
    let Key::Character(s) = logical else {
        return physical;
    };
    let mut chars = s.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return physical;
    };
    if !c.is_ascii() {
        return physical;
    }
    // ASCII the shortcuts don't name (`!`, `&`, a digit off the number row) matches nothing,
    // so a symbol key can't set off the letter shortcut at its QWERTY position.
    char_code(c).unwrap_or(KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified))
}

fn is_number_key(code: KeyCode) -> bool {
    use KeyCode as K;
    matches!(
        code,
        K::Digit0
            | K::Digit1
            | K::Digit2
            | K::Digit3
            | K::Digit4
            | K::Digit5
            | K::Digit6
            | K::Digit7
            | K::Digit8
            | K::Digit9
            | K::Numpad0
            | K::Numpad1
            | K::Numpad2
            | K::Numpad3
            | K::Numpad4
            | K::Numpad5
            | K::Numpad6
            | K::Numpad7
            | K::Numpad8
            | K::Numpad9
            | K::NumpadDecimal
            | K::NumpadAdd
            | K::NumpadSubtract
            | K::NumpadMultiply
            | K::NumpadDivide
            | K::NumpadEqual
            | K::NumpadComma
    )
}

/// The US key that types `c` (shifted punctuation included: `?` is the `/` key).
fn char_code(c: char) -> Option<KeyCode> {
    use KeyCode as K;
    const LETTERS: [KeyCode; 26] = [
        K::KeyA, K::KeyB, K::KeyC, K::KeyD, K::KeyE, K::KeyF, K::KeyG, K::KeyH, K::KeyI, K::KeyJ,
        K::KeyK, K::KeyL, K::KeyM, K::KeyN, K::KeyO, K::KeyP, K::KeyQ, K::KeyR, K::KeyS, K::KeyT,
        K::KeyU, K::KeyV, K::KeyW, K::KeyX, K::KeyY, K::KeyZ,
    ];
    if c.is_ascii_alphabetic() {
        return Some(LETTERS[(c.to_ascii_lowercase() as u8 - b'a') as usize]);
    }
    Some(match c {
        '[' | '{' => K::BracketLeft,
        ']' | '}' => K::BracketRight,
        '\\' | '|' => K::Backslash,
        ';' | ':' => K::Semicolon,
        '\'' | '"' => K::Quote,
        ',' | '<' => K::Comma,
        '.' | '>' => K::Period,
        '/' | '?' => K::Slash,
        '-' | '_' => K::Minus,
        '=' | '+' => K::Equal,
        '`' | '~' => K::Backquote,
        ' ' => K::Space,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(s: &str) -> Key {
        Key::Character(s.into())
    }

    #[test]
    fn qwerty_is_unchanged() {
        assert_eq!(shortcut_code(KeyCode::KeyZ, &ch("z")), KeyCode::KeyZ);
        assert_eq!(shortcut_code(KeyCode::KeyQ, &ch("Q")), KeyCode::KeyQ);
        assert_eq!(shortcut_code(KeyCode::Slash, &ch("?")), KeyCode::Slash);
        assert_eq!(shortcut_code(KeyCode::BracketLeft, &ch("[")), KeyCode::BracketLeft);
        assert_eq!(shortcut_code(KeyCode::Digit1, &ch("!")), KeyCode::Digit1);
        assert_eq!(shortcut_code(KeyCode::Escape, &Key::Escape), KeyCode::Escape);
        assert_eq!(shortcut_code(KeyCode::Space, &Key::Space), KeyCode::Space);
    }

    #[test]
    fn azerty_letters_follow_the_labels() {
        // The keys labelled Z, Q, A, W, M on AZERTY.
        assert_eq!(shortcut_code(KeyCode::KeyW, &ch("z")), KeyCode::KeyZ);
        assert_eq!(shortcut_code(KeyCode::KeyA, &ch("q")), KeyCode::KeyQ);
        assert_eq!(shortcut_code(KeyCode::KeyQ, &ch("A")), KeyCode::KeyA);
        assert_eq!(shortcut_code(KeyCode::KeyZ, &ch("w")), KeyCode::KeyW);
        assert_eq!(shortcut_code(KeyCode::Semicolon, &ch("m")), KeyCode::KeyM);
        // The comma sits where QWERTY has M: it must not trim.
        assert_eq!(shortcut_code(KeyCode::KeyM, &ch(",")), KeyCode::Comma);
        // Shift+, types ? on AZERTY: the shortcuts dialog.
        assert_eq!(shortcut_code(KeyCode::KeyM, &ch("?")), KeyCode::Slash);
        // ! sits on QWERTY's / key and names no shortcut.
        assert!(matches!(shortcut_code(KeyCode::Slash, &ch("!")), KeyCode::Unidentified(_)));
    }

    #[test]
    fn azerty_number_row_stays_physical() {
        assert_eq!(shortcut_code(KeyCode::Digit1, &ch("&")), KeyCode::Digit1);
        assert_eq!(shortcut_code(KeyCode::Digit2, &ch("é")), KeyCode::Digit2);
        assert_eq!(shortcut_code(KeyCode::Digit7, &ch("7")), KeyCode::Digit7);
    }

    #[test]
    fn non_latin_layouts_keep_qwerty_positions() {
        // Russian ЙЦУКЕН: я on QWERTY's Z, й on Q.
        assert_eq!(shortcut_code(KeyCode::KeyZ, &ch("я")), KeyCode::KeyZ);
        assert_eq!(shortcut_code(KeyCode::KeyQ, &ch("Й")), KeyCode::KeyQ);
        // A dead key falls back to its position.
        assert_eq!(shortcut_code(KeyCode::BracketLeft, &Key::Dead(Some('^'))), KeyCode::BracketLeft);
    }
}
