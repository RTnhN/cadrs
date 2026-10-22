#!/usr/bin/env bash
# Builds the site and exports it as static files for GitHub Pages.
#
#   site/export.sh [OUT_DIR] [BASE_PATH]     defaults: site/dist, /cadrs
#
# Needs the `proa` CLI from our Proa fork (static export is not upstream yet) and the
# Tailwind CLI v4 on PATH:
#   cargo install --git https://github.com/rvdende/proa --branch feat/static-export --locked proa-cli
set -euo pipefail
cd "$(dirname "$0")"
out="${1:-dist}"
base_path="${2-/cadrs}"

# The cadrs repo's .cargo/config.toml passes nightly-only rustflags (and mold) for the app;
# RUSTFLAGS replaces them for this stable-toolchain build.
export RUSTFLAGS=""

# Download links point at the latest release when its tag is known.
if [[ -z "${CADRS_RELEASE:-}" ]]; then
    auth=()
    [[ -n "${GITHUB_TOKEN:-}" ]] && auth=(-H "Authorization: Bearer $GITHUB_TOKEN")
    CADRS_RELEASE="$(curl -fsSL "${auth[@]}" https://api.github.com/repos/rvdende/cadrs/releases/latest \
        | sed -n 's/^  "tag_name": "\(.*\)",$/\1/p')" || true
fi
export CADRS_RELEASE="${CADRS_RELEASE:-}"
echo "release: ${CADRS_RELEASE:-unknown (links go to the releases page)}"

proa build --locked --artifact-out target/site/cadrs-site
proa export --bin target/site/cadrs-site --out "$out" --base-path "$base_path"
