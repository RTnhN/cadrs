#!/bin/sh
# The kernel's gate: every cadrs_kernel test binary run on a clean tree, each case classified
# (pass, known failure, STALE known failure, REGRESSION, HANG, CRASH; slow over 60 s), with a
# result file the scoreboard (tools/kernel_scoreboard.py) reads. Exits 1 when anything is STALE,
# a REGRESSION, a HANG or a CRASH, or when `cargo check -p cadrs_kernel` (no features: the
# library without the tests' `testkit`) or `cargo check -p cadrs_core` fails. See
# tools/kernel_gate.py for the classification.
#
# usage: tools/kernel_gate.sh [--rev REV | --tree DIR] [--out DIR] [kernel_gate.py options]
#
#   --rev REV   test `git archive REV` (default HEAD), exported to <out>/tree with the gitignored
#               OCCT suites regenerated there (tools/occt_to_rust.py of that tree, its
#               tools/occt_known_failures.txt)
#   --tree DIR  test DIR as it is (e.g. the working tree): nothing exported or regenerated
#   --out DIR   results.json, summary.txt and stderr.log (default $CARGO_TARGET_DIR/kernel_gate,
#               else target/kernel_gate)
#   other options go to tools/kernel_gate.py: --only occt_blend,smlib_fillets, --skip ...,
#   --hang 900 (seconds before a test is a hang), --binary-timeout 14400, --slow 60
#
# Environment: OCCT_TESTS (OCCT's tests/ folder, default ../opencascade-rs/crates/occt-sys/OCCT/tests
# beside this checkout; OCCT 7.8.1), CADRS_OCCT_DATA (the unpacked opencascade-dataset-7.8.0,
# default ../occt-dataset/opencascade-dataset-7.8.0), OCCT_GROUPS (the OCCT groups to generate),
# CADRS_SMLIB_ORACLE (default target/smlib_oracle/smlib_oracle when built), RUST_TEST_THREADS (3),
# CARGO_TARGET_DIR, CARGO_BUILD_JOBS.
set -eu
repo=$(cd "$(dirname "$0")/.." && pwd)
rev=HEAD
tree=
out=
while [ $# -gt 0 ]; do
    case "$1" in
        --rev) rev=$2; shift 2 ;;
        --tree) tree=$(cd "$2" && pwd); shift 2 ;;
        --out) out=$2; shift 2 ;;
        *) break ;;
    esac
done
target=${CARGO_TARGET_DIR:-$repo/target}
case "$target" in /*) ;; *) target=$repo/$target ;; esac
export CARGO_TARGET_DIR=$target
out=${out:-$target/kernel_gate}
mkdir -p "$out"
out=$(cd "$out" && pwd)
export CADRS_OCCT_DATA=${CADRS_OCCT_DATA:-$repo/../occt-dataset/opencascade-dataset-7.8.0}
export RUST_TEST_THREADS=${RUST_TEST_THREADS:-3}
# The SMLib oracle (tools/smlib_oracle/README.md), when it's built: smlib_oracle's tests use it.
if [ -z "${CADRS_SMLIB_ORACLE:-}" ] && [ -x "$repo/target/smlib_oracle/smlib_oracle" ]; then
    export CADRS_SMLIB_ORACLE="$repo/target/smlib_oracle/smlib_oracle"
fi
occt_tests=${OCCT_TESTS:-$repo/../opencascade-rs/crates/occt-sys/OCCT/tests}
occt_groups=${OCCT_GROUPS:-blend boolean draft evolved feat fillet2d heal mkface offset pipe prism sewing thrusection}

commit=
if [ -z "$tree" ]; then
    commit=$(git -C "$repo" rev-parse "$rev")
    for d in "$occt_tests" "$CADRS_OCCT_DATA"; do
        [ -d "$d" ] || { echo "kernel_gate: $d is missing (set OCCT_TESTS and CADRS_OCCT_DATA)" >&2; exit 2; }
    done
    tree=$out/tree
    rm -rf "$tree"
    mkdir -p "$tree"
    git -C "$repo" archive "$commit" | tar -x -C "$tree"
    echo "kernel_gate: $commit exported to $tree" >&2
    # shellcheck disable=SC2086 # the groups are words
    python3 -I "$tree/tools/occt_to_rust.py" --data "$CADRS_OCCT_DATA" "$occt_tests" "$tree/crates/cadrs_kernel/tests" $occt_groups >&2
else
    commit=$(git -C "$tree" rev-parse HEAD 2>/dev/null || true)
    [ -z "$commit" ] || [ -z "$(git -C "$tree" status --porcelain --untracked-files=no 2>/dev/null)" ] || commit="$commit+dirty"
fi
exec python3 -I "$repo/tools/kernel_gate.py" --tree "$tree" --out "$out" --commit "$commit" "$@"
