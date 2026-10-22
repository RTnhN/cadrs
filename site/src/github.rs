//! The GitHub button, with the repository's star count.
//!
//! `export.sh` passes the count at export time in `CADRS_STARS`, so the static page has it;
//! [`STARS_SCRIPT`] then fetches the current count from the GitHub API when the page loads
//! (kept for ten minutes per browser session, as unauthenticated calls are rate-limited).

use std::sync::OnceLock;

use proa_core::{perf_ok, DataLoader, WebContext, WebRenderSync, WriteError};
use proa_macros::html_sync;

use crate::components::{
    button_recipe, ButtonRecipeProps, ButtonSize, ButtonVariant, Icon, IconSize,
};
use crate::icons::IconRs;

const REPO: &str = "https://github.com/rvdende/cadrs";

pub const STARS_SCRIPT: &str = "<script>(function(){\
var key=\"cadrs-stars\";\
function compact(n){return n<1000?String(n):(n/1000).toFixed(n<10000?1:0).replace(/\\.0$/,\"\")+\"k\"}\
function show(n){var e=document.querySelectorAll(\"[data-github-stars]\");for(var i=0;i<e.length;i++){\
e[i].textContent=compact(n);var w=e[i].closest(\"[data-github-stars-wrap]\");if(w)w.hidden=false;\
var a=e[i].closest(\"a\");if(a)a.setAttribute(\"aria-label\",\"cadrs on GitHub, \"+n+(n===1?\" star\":\" stars\"))}}\
function ready(f){if(document.readyState===\"loading\")document.addEventListener(\"DOMContentLoaded\",f);else f()}\
try{var c=JSON.parse(sessionStorage.getItem(key));if(c&&Date.now()-c.t<6e5){ready(function(){show(c.n)});return}}catch(e){}\
fetch(\"https://api.github.com/repos/rvdende/cadrs\").then(function(r){return r.ok?r.json():null}).then(function(j){\
if(!j||typeof j.stargazers_count!==\"number\")return;var n=j.stargazers_count;\
try{sessionStorage.setItem(key,JSON.stringify({n:n,t:Date.now()}))}catch(e){}ready(function(){show(n)})})\
.catch(function(){})\
})()</script>";

/// The star count at export time: its compact form (`1.2k`) and the button's label.
struct Stars {
    count: String,
    label: String,
}

fn stars() -> Option<&'static Stars> {
    static STARS: OnceLock<Option<Stars>> = OnceLock::new();
    STARS
        .get_or_init(|| {
            let n: u64 = std::env::var("CADRS_STARS").ok()?.trim().parse().ok()?;
            // Built once per process, not per render.
            let count = if n < 1000 {
                perf_ok(n.to_string())
            } else if n < 10_000 {
                let tenths = perf_ok(format!("{:.1}", n as f64 / 1000.0));
                perf_ok(format!("{}k", tenths.trim_end_matches(".0")))
            } else {
                perf_ok(format!("{}k", (n as f64 / 1000.0).round()))
            };
            let unit = if n == 1 { "star" } else { "stars" };
            let label = perf_ok(format!("cadrs on GitHub, {n} {unit}"));
            Some(Stars { count, label })
        })
        .as_ref()
}

/// A link to the repository: the GitHub mark, "GitHub", and the star count.
pub struct GithubButton;

impl<L: DataLoader> WebRenderSync<L> for GithubButton {
    fn render(self, cx: &mut WebContext<L>) -> Result<(), WriteError> {
        let style = button_recipe().resolve(ButtonRecipeProps {
            size: Some(ButtonSize::Sm),
            variant: Some(ButtonVariant::Outline),
            ..ButtonRecipeProps::default()
        });
        match stars() {
            Some(stars) => html_sync! {
                <a href={REPO} aria-label={stars.label.as_str()} class={style}>
                    {Icon { icon: IconRs(icon_rs::GITHUB), size: Some(IconSize::Sm), class: None }}
                    "GitHub"
                    <span
                        data-github-stars-wrap
                        class="ml-1 flex items-center gap-1 border-l border-border pl-2 tabular-nums"
                    >
                        {Icon {
                            icon: IconRs(icon_rs::STAR),
                            size: Some(IconSize::Xs),
                            class: None,
                        }}
                        <span data-github-stars>{stars.count.as_str()}</span>
                    </span>
                </a>
            }
            .render(cx),
            // No count at export time: the script fills it in and shows it.
            None => html_sync! {
                <a href={REPO} aria-label="cadrs on GitHub" class={style}>
                    {Icon { icon: IconRs(icon_rs::GITHUB), size: Some(IconSize::Sm), class: None }}
                    "GitHub"
                    <span
                        data-github-stars-wrap
                        hidden
                        class="ml-1 flex items-center gap-1 border-l border-border pl-2 tabular-nums"
                    >
                        {Icon {
                            icon: IconRs(icon_rs::STAR),
                            size: Some(IconSize::Xs),
                            class: None,
                        }}
                        <span data-github-stars></span>
                    </span>
                </a>
            }
            .render(cx),
        }
    }
}
