//! The light, dark and system theme switcher.
//!
//! The choice is stored under the `theme` localStorage key, as Proa UI's own theme toggle does:
//! `light` or `dark`, or nothing to follow the operating system. [`THEME_SCRIPT`] runs in the
//! head, before first paint: it sets the `dark` class Proa UI's recipes key on, mirrors
//! `color-scheme`, records the mode in `data-theme-mode` (which picks the button's icon in
//! styles/input.css), and cycles the mode when a `[data-theme-toggle]` button is clicked.

use proa_core::{DataLoader, WebContext, WebRenderSync, WriteError};
use proa_macros::html_sync;

use crate::components::{
    button_recipe, ButtonRecipeProps, ButtonSize, ButtonVariant, Icon, IconSize,
};
use crate::icons::IconRs;

pub const THEME_SCRIPT: &str = "<script>(function(){\
var d=document.documentElement,m=window.matchMedia(\"(prefers-color-scheme: dark)\"),\
names={system:\"System\",light:\"Light\",dark:\"Dark\"},next={system:\"light\",light:\"dark\",dark:\"system\"};\
function mode(){try{var t=localStorage.getItem(\"theme\");if(t===\"light\"||t===\"dark\")return t}catch(e){}return\"system\"}\
function apply(){var t=mode(),k=t===\"dark\"||(t===\"system\"&&m.matches);\
d.classList.toggle(\"dark\",k);d.style.colorScheme=k?\"dark\":\"light\";d.setAttribute(\"data-theme-mode\",t);\
var l=\"Theme: \"+names[t]+\". Switch to \"+names[next[t]]+\".\",b=document.querySelectorAll(\"[data-theme-toggle]\");\
for(var i=0;i<b.length;i++){b[i].setAttribute(\"aria-label\",l);b[i].title=l}}\
apply();m.addEventListener(\"change\",apply);document.addEventListener(\"DOMContentLoaded\",apply);\
document.addEventListener(\"click\",function(e){var b=e.target.closest&&e.target.closest(\"[data-theme-toggle]\");if(!b)return;\
var t=next[mode()];try{if(t===\"system\")localStorage.removeItem(\"theme\");else localStorage.setItem(\"theme\",t)}catch(e){}apply()})\
})()</script>";

/// An icon button that cycles System → Light → Dark, showing the current mode's icon.
pub struct ThemeSwitcher;

impl<L: DataLoader> WebRenderSync<L> for ThemeSwitcher {
    fn render(self, cx: &mut WebContext<L>) -> Result<(), WriteError> {
        let style = button_recipe().resolve(ButtonRecipeProps {
            size: Some(ButtonSize::Icon),
            variant: Some(ButtonVariant::Ghost),
            ..ButtonRecipeProps::default()
        });
        html_sync! {
            <button
                type="button"
                data-theme-toggle
                aria-label="Theme: System. Switch to Light."
                title="Theme: System. Switch to Light."
                class={style}
            >
                <span data-theme-icon="system">
                    {Icon {
                        icon: IconRs(icon_rs::THEME_SYSTEM),
                        size: Some(IconSize::Sm),
                        class: None,
                    }}
                </span>
                <span data-theme-icon="light">
                    {Icon {
                        icon: IconRs(icon_rs::THEME_LIGHT),
                        size: Some(IconSize::Sm),
                        class: None,
                    }}
                </span>
                <span data-theme-icon="dark">
                    {Icon {
                        icon: IconRs(icon_rs::THEME_DARK),
                        size: Some(IconSize::Sm),
                        class: None,
                    }}
                </span>
            </button>
        }
        .render(cx)
    }
}
