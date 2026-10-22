#!/usr/bin/env python3
"""Converts SMLib's prog_test suites (NVIDIA usd-brep, tests/prog_test/src/*.cpp, Apache-2.0)
into the kernel's tests: tests/smlib_<suite>.rs for each suite of tools/smlib/<suite>.py.

usage: tools/smlib_to_rust.py <usd-brep checkout> [suite ...]   (all suites without any)

A suite module (one per suite, so suites can be added side by side) defines
    SOURCE = "smFilletTest.cpp"        # its prog_test file, for the generated header
    TITLE = "fillet"                   # what its tests test, for the generated header
    PRELUDE = '''...'''                # Rust put after `use smlib_common::*;` (its runner)
    def extract(ctx) -> [Case]         # its cases
and may define NOT_CONVERTED (a list of (block, reason) for blocks it leaves out).

The driver splits blocks (if_blocks, case_blocks), copies each case's models into
tests/data/smlib/<suite>/ (with a README carrying the licence), marks known failures
(tools/smlib_known_failures/<suite>.txt: `name` or `name slow`; a known failure must still fail,
so one that passes has to be taken off the list; `slow` ones only run with --ignored) and, when
CADRS_SMLIB_ORACLE names the SMLib oracle (tools/smlib_oracle), runs each case's oracle command
once and bakes SMLib's volume, area, face and edge counts and solidity into the case (volume and
area to a relative 1e-4; a result SMLib doesn't call a manifold solid isn't used). Without the oracle a case checks SMLib's asserted counts and the
invariants (valid, closed, Euler characteristic, positive volume). Face and edge counts are
compared only for planar models (see tests/smlib_common/mod.rs).

Points on a model (edge middles and ends, face centroids, in SMLib's file order) come from the
kernel's own reader (examples/smlib_points.rs), built once.
"""
import concurrent.futures
import hashlib
import importlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
CRATE = os.path.join(REPO, "crates/cadrs_kernel")
DATA = os.path.join(CRATE, "tests/data/smlib")
KNOWN = os.path.join(HERE, "smlib_known_failures")
sys.path.insert(0, HERE)


# --- Helpers for suite modules ------------------------------------------------------------

def strip_comments(code):
    """C++ source without its comments (strings kept)."""
    out, i, n = [], 0, len(code)
    while i < n:
        if code.startswith("//", i):
            j = code.find("\n", i)
            i = n if j < 0 else j
        elif code.startswith("/*", i):
            j = code.find("*/", i + 2)
            i = n if j < 0 else j + 2
        elif code[i] == '"':
            j = i + 1
            while j < n and code[j] != '"':
                j += 2 if code[j] == "\\" else 1
            out.append(code[i:j + 1])
            i = j + 1
        else:
            out.append(code[i])
            i += 1
    return "".join(out)


def _balanced(code, open_at):
    """The index just past the brace matching the `{` at `open_at`."""
    depth, i = 0, open_at
    while i < len(code):
        c = code[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        elif c == '"':
            i += 1
            while i < len(code) and code[i] != '"':
                i += 2 if code[i] == "\\" else 1
        i += 1
    return len(code)


def drop_dead(code):
    """`code` without its `if (0) { ... }` and `if (FALSE) { ... }` branches (an `else` after
    one is kept as plain code)."""
    while True:
        m = re.search(r"\bif\s*\(\s*(?:0|FALSE|false)\s*\)\s*\{", code)
        if not m:
            return code
        end = _balanced(code, m.end() - 1)
        rest = code[end:]
        e = re.match(r"\s*else\b", rest)
        code = code[:m.start()] + (rest[e.end():] if e else rest)


def if_blocks(src, var="lCount"):
    """[(N, body)] for each `if (var == N) { ... }` of `src` (comments stripped), in order. A
    number with several blocks gets all of them, joined."""
    code = strip_comments(src)
    out = []
    for m in re.finditer(r"\bif\s*\(\s*%s\s*==\s*(\d+)\s*\)\s*\{" % re.escape(var), code):
        end = _balanced(code, m.end() - 1)
        out.append((int(m.group(1)), drop_dead(code[m.end():end - 1])))
    merged = {}
    for n, b in out:
        merged[n] = merged.get(n, "") + "\n" + b if n in merged else b
    return sorted(merged.items())


def case_blocks(src):
    """[(N, body)] for each `case N:` of `src` (comments stripped), up to the next `case`,
    `default` or the switch's end."""
    code = strip_comments(src)
    marks = list(re.finditer(r"\bcase\s+(\d+)\s*:|\bdefault\s*:", code))
    out = []
    for k, m in enumerate(marks):
        if m.group(1) is None:
            continue
        end = marks[k + 1].start() if k + 1 < len(marks) else len(code)
        out.append((int(m.group(1)), code[m.end():end]))
    return out


def num(s):
    """A C number literal."""
    return float(s.rstrip("fFlL"))


def rust_f64(x):
    """A float as a Rust literal."""
    r = repr(float(x))
    return r if ("." in r or "e" in r or "inf" in r or "nan" in r) else r + ".0"


class Fixture:
    """A model of the usd-brep checkout (path relative to it, `#k` for brep k of a part)."""

    def __init__(self, rel):
        self.rel = rel

    @property
    def file(self):
        return self.rel.split("#")[0]

    @property
    def name(self):
        """Its name in tests/data/smlib/<suite> (and `#k`)."""
        base = os.path.basename(self.file)
        return base + (self.rel[len(self.file):])


class Out:
    """An output file the oracle writes (a fresh temporary path)."""


class Case:
    """One generated test.

    name: the test function's name; doc: its doc comment (one line); call: Rust code giving a
    `Result<(), String>`, with `{expect}` where the expectations go (a `&Expect`); fixtures:
    the Fixtures it reads; expect: SMLib's asserted values (keys of Expect: volume, area,
    faces, edges, solid); oracle: (subcommand, [args]) with Fixture and Out among the args, or
    None; slow: run only with --ignored even when it passes."""

    def __init__(self, name, doc, call, fixtures=(), expect=None, oracle=None, slow=False):
        self.name, self.doc, self.call = name, doc, call
        self.fixtures = list(fixtures)
        self.expect = dict(expect or {})
        self.oracle = oracle
        self.slow = slow
        self.note = None


class Ctx:
    """What a suite module's extract() gets: the checkout, the suite's source and points on
    models."""

    def __init__(self, root, suite, points):
        self.root, self.suite = root, suite
        self._points = points

    def source(self, name):
        return open(os.path.join(self.root, "tests/prog_test/src", name), encoding="latin-1").read()

    def path(self, rel):
        return os.path.join(self.root, rel)

    def exists(self, rel):
        return os.path.exists(self.path(rel.split("#")[0]))

    def points(self, fixture):
        """{edges, ends, faces, planar, solid} of a Fixture (see examples/smlib_points.rs), or
        {error}."""
        return self._points(fixture)


# --- Points on models (the kernel's reader) ------------------------------------------------

class Points:
    def __init__(self, root):
        self.root = root
        self.cache = {}
        self.bin = None

    def _build(self):
        target = os.environ.get("CARGO_TARGET_DIR", os.path.join(REPO, "target"))
        subprocess.run(["cargo", "build", "-r", "-q", "-p", "cadrs_kernel", "--example", "smlib_points"], cwd=REPO, check=True)
        self.bin = os.path.join(target, "release/examples/smlib_points")

    def prefetch(self, fixtures):
        todo = sorted({f.rel for f in fixtures if f.rel not in self.cache})
        if not todo:
            return
        if self.bin is None:
            self._build()
        args = [os.path.join(self.root, r) for r in todo]
        with concurrent.futures.ThreadPoolExecutor(8) as ex:
            outs = ex.map(lambda a: subprocess.run([self.bin, a], capture_output=True, text=True), args)
            for r, o in zip(todo, outs):
                line = o.stdout.strip().splitlines()[-1] if o.stdout.strip() else json.dumps({"error": o.stderr.strip()[-300:]})
                self.cache[r] = json.loads(line)

    def __call__(self, fixture):
        self.prefetch([fixture])
        return self.cache[fixture.rel]


# --- The oracle -----------------------------------------------------------------------------

class Oracle:
    """SMLib run on a case: `smlib_oracle <subcommand> args...` printing one line of JSON.
    Results are cached (tools' cache under the target dir) by the command and the models'
    contents."""

    def __init__(self, binary, root):
        self.binary, self.root = binary, root
        target = os.environ.get("CARGO_TARGET_DIR", os.path.join(REPO, "target"))
        self.cache_path = os.path.join(target, "smlib_oracle_cache.json")
        try:
            self.cache = json.load(open(self.cache_path))
        except (OSError, ValueError):
            self.cache = {}
        st = os.stat(binary)
        self.stamp = "%d-%d" % (st.st_mtime, st.st_size)
        self.tmp = tempfile.mkdtemp(prefix="smlib_oracle_")

    def _key(self, sub, args):
        h = hashlib.sha1((self.stamp + sub).encode())
        for a in args:
            if isinstance(a, Fixture):
                h.update(open(os.path.join(self.root, a.file), "rb").read())
                h.update(a.rel.encode())
            elif isinstance(a, Out):
                h.update(b"<out>")
            else:
                h.update(repr(a).encode())
        return h.hexdigest()

    def run(self, sub, args, timeout=600):
        """The JSON dict SMLib's run prints, or {"ok": False, "status": why}."""
        key = self._key(sub, args)
        if key in self.cache:
            return self.cache[key]
        argv = [self.binary, sub]
        for k, a in enumerate(args):
            if isinstance(a, Fixture):
                # (`file.smp#k`: the oracle reads B-rep k of a part.)
                argv.append(os.path.join(self.root, a.rel))
            elif isinstance(a, Out):
                argv.append(os.path.join(self.tmp, "%s_%d.smb" % (key[:12], k)))
            elif isinstance(a, float):
                argv.append(repr(a))
            else:
                argv.append(str(a))
        try:
            r = subprocess.run(argv, capture_output=True, text=True, timeout=timeout)
            lines = [l for l in r.stdout.splitlines() if l.strip().startswith("{")]
            res = json.loads(lines[-1]) if lines else {"ok": False, "status": "no output (exit %d): %s" % (r.returncode, r.stderr.strip()[-200:])}
        except subprocess.TimeoutExpired:
            res = {"ok": False, "status": "timed out after %ds" % timeout}
        except ValueError as e:
            res = {"ok": False, "status": "bad output: %s" % e}
        self.cache[key] = res
        return res

    def save(self):
        os.makedirs(os.path.dirname(self.cache_path), exist_ok=True)
        json.dump(self.cache, open(self.cache_path, "w"), indent=0, sort_keys=True)
        shutil.rmtree(self.tmp, ignore_errors=True)


# --- Known failures ------------------------------------------------------------------------

# Cases whose SMLib volume and area are SMLib's own approximations, off by more than the 1e-4
# they are compared to: compared on the counts, solidity and validity only. Why, with the exact
# values ours are checked against elsewhere (tests/offset_local.rs), in each note.
APPROXIMATE = {
    ("offset", "offset_06_off_extrusion"): "SMLib's volume (407.3193) and area are its approximation: it measures the input 6.4e-4 over its exact 591.836 (a 59.1836 section, 10 deep) and gives the inset's two ends different areas (42.244, 41.931) where the erosion's are both 41.95986, its volume 41.95986 × 9.7 = 407.0106 (ours 407.0099, the inset spline a fit): volume and area not compared",
}


def known_failures(suite):
    """{name: slow} from tools/smlib_known_failures/<suite>.txt."""
    out = {}
    p = os.path.join(KNOWN, suite + ".txt")
    if os.path.exists(p):
        for line in open(p):
            w = line.split()
            if w and not w[0].startswith("#"):
                out[w[0]] = "slow" in w[1:]
    return out


# --- Generation ----------------------------------------------------------------------------

def expect_literal(e):
    fields = []
    for k in ("volume", "area"):
        if e.get(k) is not None:
            fields.append("%s: Some(%s)" % (k, rust_f64(e[k])))
    for k in ("faces", "edges"):
        if e.get(k) is not None:
            fields.append("%s: Some(%d)" % (k, e[k]))
    if e.get("solid") is not None:
        fields.append("solid: Some(%s)" % ("true" if e["solid"] else "false"))
    return "&Expect { %s }" % ", ".join(fields + ["..Expect::default()"]) if fields else "&Expect::default()"


def copy_fixtures(root, suite, cases):
    d = os.path.join(DATA, suite)
    if not any(c.fixtures for c in cases):
        # A suite that builds all its models in code (sweeps, primitives) has no data.
        shutil.rmtree(d, ignore_errors=True)
        return 0
    os.makedirs(d, exist_ok=True)
    seen = {}
    for c in cases:
        for f in c.fixtures:
            src = os.path.join(root, f.file)
            base = os.path.basename(f.file)
            if seen.get(base, src) != src:
                raise SystemExit("%s: two models named %s (%s, %s)" % (suite, base, seen[base], src))
            seen[base] = src
    for base, src in sorted(seen.items()):
        shutil.copy(src, os.path.join(d, base))
    dirs = sorted({os.path.relpath(os.path.dirname(s), root) for s in seen.values()})
    open(os.path.join(d, "README.md"), "w").write(
        "SMLib models from NVIDIA's usd-brep (https://github.com/NVIDIA-Omniverse/usd-brep,\n"
        "%s), Copyright (c) 2020-2026 NVIDIA CORPORATION & AFFILIATES, licensed under the\n"
        "Apache License 2.0 (http://www.apache.org/licenses/LICENSE-2.0). Unmodified. Used by\n"
        "`tests/smlib_%s.rs` (generated by `tools/smlib_to_rust.py`).\n" % (", ".join("`%s`" % x for x in dirs), suite))
    # Models no case uses any more.
    for f in os.listdir(d):
        if f != "README.md" and f not in seen:
            os.remove(os.path.join(d, f))
    return len(seen)


def generate(root, suite, oracle):
    mod = importlib.import_module("smlib." + suite)
    points = Points(root)
    ctx = Ctx(root, suite, points)
    cases = mod.extract(ctx)
    names = [c.name for c in cases]
    dup = {n for n in names if names.count(n) > 1}
    if dup:
        raise SystemExit("%s: duplicate case names %s" % (suite, sorted(dup)))
    known = known_failures(suite)
    stale = sorted(set(known) - set(names))
    if stale:
        print("%s: known failures without a case: %s" % (suite, " ".join(stale)), file=sys.stderr)
    # SMLib's own runs.
    if oracle:
        todo = [c for c in cases if c.oracle]
        with concurrent.futures.ThreadPoolExecutor(int(os.environ.get("CADRS_SMLIB_ORACLE_JOBS", "8"))) as ex:
            res = list(ex.map(lambda c: oracle.run(c.oracle[0], c.oracle[1]), todo))
        for c, r in zip(todo, res):
            if r.get("ok") and r.get("solid") is False and c.expect.get("solid", True):
                # A case that should give a solid where SMLib's own result isn't a manifold
                # solid (most enclose no volume at all): SMLib went wrong, its values aren't a
                # reference.
                c.note = "SMLib's own result isn't a manifold solid (volume %s, %s faces): not compared" % (r.get("volume"), r.get("faces"))
            elif r.get("ok"):
                if c.expect.get("edges") is not None and r.get("edges") is not None and c.expect["edges"] != r["edges"]:
                    c.note = (c.note + "; " if c.note else "") + "SMLib's test asserts %d edges, the oracle gives %d" % (c.expect["edges"], r["edges"])
                for k in ("volume", "area", "faces", "edges", "solid"):
                    if k in r:
                        c.expect[k] = r[k]
            else:
                c.note = "SMLib (the oracle) fails: %s" % r.get("status", "?")
        oracle.save()
    for c in cases:
        why = APPROXIMATE.get((suite, c.name))
        if why:
            c.expect.pop("volume", None)
            c.expect.pop("area", None)
            c.note = (c.note + "; " if c.note else "") + why
    nmodels = copy_fixtures(root, suite, cases)
    out = [
        "//! SMLib's %s tests (NVIDIA usd-brep `tests/prog_test/src/%s`, Apache-2.0; %s" % (
            mod.TITLE, mod.SOURCE, "models in" if nmodels else "models built in"),
        "//! %s) on the native kernel. Generated by `tools/smlib_to_rust.py` from" % ("`tests/data/smlib/%s`" % suite if nmodels else "code"),
        "//! `tools/smlib/%s.py`%s; known failures (`tools/smlib_known_failures/%s.txt`)" % (suite, " with SMLib's own results (the oracle)" if oracle else "", suite),
        "//! must still fail.",
    ]
    if getattr(mod, "NOT_CONVERTED", None):
        out.append("//!")
        out.append("//! Not converted:")
        for block, why in mod.NOT_CONVERTED:
            out.append("//! - %s: %s" % (block, why))
    out += ["#![allow(clippy::all)]", "", "mod smlib_common;", "#[allow(unused_imports)]", "use smlib_common::*;", ""]
    out += [l.rstrip() for l in mod.PRELUDE.strip("\n").split("\n")]
    nknown = 0
    for c in cases:
        call = c.call.replace("{expect}", expect_literal(c.expect))
        out.append("")
        out.append("/// %s" % c.doc)
        if c.note:
            out.append("/// %s." % c.note)
        out.append("#[test]")
        if c.name in known:
            nknown += 1
            if known[c.name]:
                out.append('#[ignore = "known failure, slow (tools/smlib_known_failures/%s.txt)"]' % suite)
            out.append("fn %s() {" % c.name)
            out.append('    known_failure(%s, "%s", "%s");' % (call, suite, c.name))
        else:
            if c.slow:
                out.append('#[ignore = "slow"]')
            out.append("fn %s() {" % c.name)
            out.append("    %s.unwrap();" % call)
        out.append("}")
    open(os.path.join(CRATE, "tests/smlib_%s.rs" % suite), "w").write("\n".join(out) + "\n")
    print("%s: %d cases (%d known failures), %d models%s" % (suite, len(cases), nknown, nmodels, ", oracle" if oracle else ""))


def main():
    # Suite modules import this file by name: one module, not a second copy of __main__.
    sys.modules.setdefault("smlib_to_rust", sys.modules[__name__])
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    root = os.path.abspath(sys.argv[1])
    suites = sys.argv[2:] or sorted(f[:-3] for f in os.listdir(os.path.join(HERE, "smlib")) if f.endswith(".py") and not f.startswith("_"))
    ob = os.environ.get("CADRS_SMLIB_ORACLE")
    oracle = Oracle(ob, root) if ob and os.path.exists(ob) else None
    if ob and not oracle:
        print("CADRS_SMLIB_ORACLE=%s doesn't exist: no oracle values" % ob, file=sys.stderr)
    for s in suites:
        generate(root, s, oracle)


if __name__ == "__main__":
    main()
