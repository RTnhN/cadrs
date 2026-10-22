#!/usr/bin/env bash
# Formats the site's own code. Not `cargo fmt`: rustfmt follows `mod` declarations into the Proa
# UI sources in src/components, which must stay exactly as `proa ui add` installed them (it
# refuses to update files that changed). `#[rustfmt::skip]` on `mod components` would keep
# rustfmt out, but it breaks the components' `#[macro_export]` re-exports.
set -euo pipefail
cd "$(dirname "$0")"

proa fmt src/pages src/layouts src/theme.rs

# Files whose modules are all the site's own.
rustfmt --edition 2021 build.rs src/app.rs src/config.rs src/error.rs src/icons.rs \
    src/routes.rs src/static_assets.rs src/theme.rs src/endpoints/*.rs src/layouts/*.rs src/pages/*.rs \
    tests/smoke.rs

# main.rs and the representations test declare `mod components`; only rustfmt's unstable
# skip_children leaves it alone, so these two use the repository's nightly.
nightly="$(sed -n 's/^channel = "\(.*\)"/\1/p' ../rust-toolchain.toml)"
rustfmt "+$nightly" --edition 2021 --unstable-features --config skip_children=true \
    src/main.rs tests/representations.rs
