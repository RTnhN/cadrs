#!/bin/sh
# Gates clippy on the kernel: every target (library, tests), warnings denied. The lint levels
# are in crates/cadrs_kernel/Cargo.toml ([lints]). CI runs this; run it before committing kernel
# changes. Extra arguments go to cargo (e.g. a target dir).
set -eu
cd "$(dirname "$0")/.."
exec cargo clippy -r -p cadrs_kernel --all-targets "$@" -- -D warnings
