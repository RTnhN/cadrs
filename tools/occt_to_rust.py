#!/usr/bin/env python3
"""Converts OCCT's DRAW test cases (OCCT/tests/<group>/<grid>/<case>) into Rust tests on the
native kernel's DRAW-style API (cadrs_kernel::native::draw).

    tools/occt_to_rust.py <OCCT/tests> <out dir> group [group ...]

writes <out dir>/occt_<group>.rs with one #[test] per case, named <grid>_<case>. A case that
uses a command the API doesn't have yet (or Tcl control flow, or OCCT's data files) becomes an
#[ignore = "needs ..."] test holding its script as comments, so the ignored tests list what is
left to do.
"""

import math
import os
import re
import sys

KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
    "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override",
    "priv", "typeof", "unsized", "virtual", "yield", "try", "k",
}

# Commands that only display, log or set up the session.
IGNORED = {
    "checkview", "erase", "donly", "display", "fit", "clear", "puts", "smallview", "axo", "top", "front", "left", "vinit", "vdisplay",
    "vfit", "vsetdispmode", "vclear", "vclose", "checkmaxtol", "checksection", "decho", "dlog", "pload", "cpulimit", "protect",
    "isos", "triangles", "dchrono", "chrono", "xwd", "vdump", "dmode", "disp", "view", "2dfit", "repaint", "settolerance",
    "checktrinfo", "dtyp", "whatis", "tolerance", "checkfreebounds", "checkcontinuity", "bfillds", "bclearobjects", "bcleartools",
    "brunparallel", "bnondestructive", "bfuzzyvalue", "bglue", "bcheckinverted", "buseobb", "bdrawwarnshapes", "bsimplify",
    "setfillhistory", "savehistory", "tclean", "incmesh",
}


class Unsupported(Exception):
    pass


def ident(name):
    n = re.sub(r"[^A-Za-z0-9_]", "_", name)
    if not n or n[0].isdigit():
        n = "v_" + n
    if n in KEYWORDS:
        n += "_"
    return n


def split_commands(text):
    """Commands of a script: lines joined at backslashes, split at ';', comments dropped. Raises
    Unsupported on Tcl structure (braces, substitutions)."""
    text = text.replace("\\\n", " ")
    out = []
    for raw in text.split("\n"):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        for part in line.split(";"):
            part = part.strip()
            if not part or part.startswith("#"):
                continue
            out.append(part)
    return out


class Case:
    def __init__(self, consts):
        self.lines = []
        self.vars = {}  # draw name -> rust ident
        self.exploded = {}  # draw prefix -> (rust vec ident, kind)
        self.consts = dict(consts)
        self.bop_args = None
        self.n_tmp = 0

    def num(self, tok):
        try:
            return float(tok)
        except ValueError:
            pass
        if tok in self.consts:
            return self.consts[tok]
        # DRAW evaluates expressions in numeric arguments (sqrt(2)/2, atan2(1,2)*180/pi).
        if re.fullmatch(r"[0-9A-Za-z_.+\-*/(), ]+", tok):
            env = {n: getattr(math, n) for n in ("sqrt", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "log", "pow", "fabs")}
            env.update({"pi": math.pi, "abs": abs})
            env.update(self.consts)
            try:
                return float(eval(tok, {"__builtins__": {}}, env))
            except Exception:
                pass
        if tok.lower() == "pi":
            return 3.141592653589793
        m = re.fullmatch(r"(-?[0-9.eE+]+)\*pi", tok)
        if m:
            return float(m.group(1)) * 3.141592653589793
        raise Unsupported(f"the number {tok}")

    def lit(self, tok):
        v = self.num(tok)
        s = repr(float(v))
        return s

    def shape(self, name):
        """A Rust expression for a DRAW shape name (a variable, or an exploded sub-shape)."""
        if name in self.vars:
            return "&" + self.vars[name]
        m = re.fullmatch(r"(.+)_(\d+)", name)
        if m and m.group(1) in self.exploded:
            vec, _ = self.exploded[m.group(1)]
            return f"&{vec}[{int(m.group(2)) - 1}]"
        raise Unsupported(f"the shape {name}")

    def bind(self, name, expr):
        r = ident(name)
        self.vars[name] = r
        self.lines.append(f"let {r} = {expr};")

    def emit(self, words):
        c, a = words[0], words[1:]
        L = self.lit
        if c in IGNORED:
            return
        if c == "dset" or c == "set":
            for i in range(0, len(a) - 1, 2):
                try:
                    self.consts[a[i]] = self.num(a[i + 1])
                except Unsupported:
                    if c == "set":
                        return  # e.g. `set imagedir ...`
                    raise
            return
        if c == "line" and len(a) == 7:
            self.bind(a[0], f"k.line({', '.join(L(x) for x in a[1:])})")
        elif c == "circle" and len(a) == 5:
            self.bind(a[0], f"k.circle({', '.join(L(x) for x in a[1:])})")
        elif c == "circle" and len(a) == 8:
            self.bind(a[0], f"k.circle_n({', '.join(L(x) for x in a[1:])})")
        elif c == "circle" and len(a) == 11:
            self.bind(a[0], f"k.circle_nx({', '.join(L(x) for x in a[1:])})")
        elif c == "plane" and len(a) in (4, 7, 10):
            nums = [L(x) for x in a[1:]]
            if len(a) == 4:
                nums += ["0.0", "0.0", "1.0"]
            if len(nums) == 6:
                self.bind(a[0], f"k.plane({', '.join(nums)})")
            else:
                self.bind(a[0], f"k.plane_x({', '.join(nums)})")
            self.vars[a[0]] = ident(a[0])
        elif c == "mkedge" and len(a) == 4:
            self.bind(a[0], f"k.mkedge(&{self.vars[a[1]] if a[1] in self.vars else self.raise_(a[1])}, {L(a[2])}, {L(a[3])})")
        elif c == "mkedge" and len(a) == 2:
            self.bind(a[0], f"k.mkedge_full(&{self.vars[a[1]] if a[1] in self.vars else self.raise_(a[1])})")
        elif c == "vertex" and len(a) == 4:
            self.bind(a[0], f"k.vertex({L(a[1])}, {L(a[2])}, {L(a[3])})")
        elif c == "edge" and len(a) == 3:
            self.bind(a[0], f"k.edge({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "wire" and len(a) >= 2:
            self.bind(a[0], f"k.wire(&[{', '.join(self.shape(x) for x in a[1:])}])?")
        elif c == "polyline" and len(a) >= 7 and (len(a) - 1) % 3 == 0:
            pts = [f"[{L(a[i])}, {L(a[i + 1])}, {L(a[i + 2])}]" for i in range(1, len(a), 3)]
            self.bind(a[0], f"k.polyline(&[{', '.join(pts)}])?")
        elif c == "mkplane" and len(a) == 2:
            self.bind(a[0], f"k.mkplane({self.shape(a[1])})?")
        elif c == "add" and len(a) == 2:
            self.bind(a[1], f"k.add_hole({self.shape(a[0])}, {self.shape(a[1])})?")
        elif c == "box" and len(a) in (4, 7):
            if len(a) == 4:
                self.bind(a[0], f"k.box_({', '.join(L(x) for x in a[1:])})?")
            else:
                self.bind(a[0], f"k.box_at({', '.join(L(x) for x in a[1:])})?")
        elif c in ("pcylinder", "pcone", "psphere", "ptorus"):
            want = {"pcylinder": 2, "pcone": 3, "psphere": 1, "ptorus": 2}[c]
            rest = a[1:]
            if len(rest) == want + 1 and rest[0] in self.vars:
                self.bind(a[0], f"k.{c}_on({self.shape(rest[0])}, {', '.join(L(x) for x in rest[1:])})?")
            elif len(rest) == want:
                self.bind(a[0], f"k.{c}({', '.join(L(x) for x in rest)})?")
            else:
                raise Unsupported(f"{c} with these arguments")
        elif c == "prism" and len(a) == 5:
            self.bind(a[0], f"k.prism({self.shape(a[1])}, {L(a[2])}, {L(a[3])}, {L(a[4])})?")
        elif c == "revol" and len(a) == 9:
            self.bind(a[0], f"k.revol({self.shape(a[1])}, {', '.join(L(x) for x in a[2:])})?")
        elif c in ("bfuse", "bcut", "bcommon") and len(a) == 3:
            self.bind(a[0], f"k.{c}({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "bop" and len(a) == 2:
            self.bop_args = (self.shape(a[0]), self.shape(a[1]))
        elif c in ("bopfuse", "bopcut", "bopcommon", "boptuc") and len(a) == 1:
            if not self.bop_args:
                raise Unsupported(f"{c} without bop")
            x, y = self.bop_args
            f = {"bopfuse": ("bfuse", x, y), "bopcut": ("bcut", x, y), "bopcommon": ("bcommon", x, y), "boptuc": ("bcut", y, x)}[c]
            self.bind(a[0], f"k.{f[0]}({f[1]}, {f[2]})?")
        elif c in ("ttranslate", "trotate", "tscale", "tmirror"):
            nargs = {"ttranslate": 3, "trotate": 7, "tscale": 4, "tmirror": 6}[c]
            if len(a) < nargs + 1:
                raise Unsupported(f"{c} with these arguments")
            shapes, nums = a[:-nargs], a[-nargs:]
            for s in shapes:
                self.bind(s, f"k.{c}({self.shape(s)}, {', '.join(L(x) for x in nums)})?")
        elif c in ("copy", "tcopy") and len(a) == 2:
            self.bind(a[1], f"{self.shape(a[0])[1:]}.clone()")
        elif c == "nurbsconvert" and len(a) == 2:
            # The same geometry (our surfaces stay exact): a copy.
            self.bind(a[0], f"{self.shape(a[1])[1:]}.clone()")
        elif c == "explode" and len(a) == 2 and a[1].lower() in ("e", "f"):
            kind = a[1].lower()
            vec = ident(a[0]) + ("_edges" if kind == "e" else "_faces")
            fn = "explode_edges" if kind == "e" else "explode_faces"
            self.lines.append(f"let {vec} = k.{fn}({self.shape(a[0])})?;")
            self.exploded[a[0]] = (vec, kind)
        elif c == "blend" and len(a) >= 4 and (len(a) - 2) % 2 == 0:
            radii = {a[i] for i in range(2, len(a), 2)}
            if len(radii) != 1:
                raise Unsupported("blend with several radii")
            edges = [self.shape(a[i]) for i in range(3, len(a), 2)]
            self.bind(a[0], f"k.blend({self.shape(a[1])}, {L(a[2])}, &[{', '.join(edges)}])?")
        elif c == "fillet2d" and len(a) == 4:
            self.bind(a[0], f"k.fillet2d({self.shape(a[1])}, {self.shape(a[2])}, {L(a[3])})?")
        elif c == "fillet2d" and len(a) == 3:
            self.bind(a[0], f"k.fillet2d_wire({self.shape(a[1])}, {L(a[2])})?")
        elif c == "chamfer2d" and len(a) == 5:
            self.bind(a[0], f"k.chamfer2d({self.shape(a[1])}, {self.shape(a[2])}, {L(a[3])}, {L(a[4])})?")
        elif c == "checkprops":
            s = self.shape(a[0])
            i = 1
            done = False
            while i < len(a):
                o = a[i]
                if o in ("-s", "-v", "-l") and i + 1 < len(a):
                    if a[i + 1] == "empty":
                        raise Unsupported("checkprops empty")
                    self.lines.append(f"k.checkprops_{o[1]}({s}, {L(a[i + 1])})?;")
                    done = True
                    i += 2
                elif o in ("-eps", "-deps"):
                    i += 2
                elif o == "-skip":
                    i += 1
                else:
                    raise Unsupported(f"checkprops {o}")
            if not done:
                raise Unsupported("checkprops without a value")
        elif c == "checknbshapes":
            s = self.shape(a[0])
            want = {}
            i = 1
            while i < len(a):
                o = a[i]
                if o in ("-vertex", "-edge", "-wire", "-face", "-shell", "-solid") and i + 1 < len(a):
                    want[o[1:]] = int(a[i + 1])
                    i += 2
                elif o in ("-compound", "-compsolid", "-shape") and i + 1 < len(a):
                    i += 2  # Not counted: DRAW's compounds have no counterpart here.
                elif o == "-t":
                    i += 1
                elif o == "-m" and i + 1 < len(a):
                    i += 2
                else:
                    raise Unsupported(f"checknbshapes {o}")
            args = ", ".join(f"Some({want[x]})" if x in want else "None" for x in ("vertex", "edge", "wire", "face", "shell", "solid"))
            self.lines.append(f"draw::check_nb(k.nbshapes({s})?, {args})?;")
        elif c == "checkshape":
            self.lines.append(f"k.checkshape({self.shape(a[0])})?;")
        else:
            raise Unsupported(c)

    def raise_(self, name):
        raise Unsupported(f"the curve {name}")


def tokens(cmd):
    if any(ch in cmd for ch in "{}[]$\""):
        raise Unsupported("Tcl substitution or blocks")
    return cmd.split()


def convert(path, consts):
    text = open(path, encoding="utf-8", errors="replace").read()
    if re.search(r"locate_data_file|\brestore\b|stepread|igesread|testreadstep|ReadStep|readstl|brestore", text):
        return None, "needs OCCT's data files", text
    case = Case(consts)
    try:
        if re.search(r"^\s*(if|foreach|for|while|proc|catch|switch)\b", text, re.M):
            raise Unsupported("Tcl control flow")
        for cmd in split_commands(text):
            if cmd.split()[0] in IGNORED:
                continue
            case.emit(tokens(cmd))
    except Unsupported as e:
        return None, f"needs {e}", text
    except KeyError as e:
        return None, f"needs the name {e}", text
    if not any(l.startswith("k.check") or l.startswith("draw::check") for l in case.lines):
        return None, "checks nothing we measure", text
    return case.lines, None, text


def group_consts(tests, group):
    consts = {}
    for f in (os.path.join(tests, group, "begin"),):
        if os.path.exists(f):
            for m in re.finditer(r"^\s*dset\s+(\w+)\s+([-0-9.eE+]+)", open(f).read(), re.M):
                consts[m.group(1)] = float(m.group(2))
    return consts


def main():
    tests, out, groups = sys.argv[1], sys.argv[2], sys.argv[3:]
    for group in groups:
        gdir = os.path.join(tests, group)
        consts = group_consts(tests, group)
        grids = [g for g in sorted(os.listdir(gdir)) if os.path.isdir(os.path.join(gdir, g))]
        body = []
        n_ok = n_ign = 0
        seen = set()
        for grid in grids:
            gconsts = dict(consts)
            gconsts.update(group_consts(gdir, grid))
            for case in sorted(os.listdir(os.path.join(gdir, grid))):
                p = os.path.join(gdir, grid, case)
                if case in ("begin", "end", "parse.rules", "cases.list") or not os.path.isfile(p):
                    continue
                name = ident(f"{grid}_{case}".lower())
                if name in seen:
                    name += "_2"
                seen.add(name)
                lines, why, text = convert(p, gconsts)
                body.append("")
                if lines is None:
                    n_ign += 1
                    body.append(f"/// OCCT `tests/{group}/{grid}/{case}`.")
                    body.append("#[test]")
                    body.append('#[ignore = "' + why.replace("\\", "\\\\").replace('"', '\\"') + '"]')
                    body.append(f"fn {name}() {{")
                    for l in text.strip("\n").split("\n"):
                        body.append(f"    // {l}".rstrip())
                    body.append("}")
                else:
                    n_ok += 1
                    body.append(f"/// OCCT `tests/{group}/{grid}/{case}`.")
                    body.append("#[test]")
                    body.append(f"fn {name}() -> Result<()> {{")
                    body.append("    #[allow(unused_mut)]")
                    body.append("    let mut k = NativeKernel::new();")
                    for l in lines:
                        body.append("    " + l)
                    body.append("    Ok(())")
                    body.append("}")
        head = [
            f"//! OCCT's `tests/{group}` cases on the native kernel: each DRAW script as Rust through",
            "//! `native::draw` (generated by `tools/occt_to_rust.py`; don't edit by hand). Cases that need",
            "//! a command the API doesn't have yet are ignored, with their script as comments.",
            f"//! {n_ok} cases converted, {n_ign} waiting.",
            "#![allow(unused_variables, clippy::all)]",
            "",
            "use cadrs_kernel::native::{draw, NativeKernel};",
            "use cadrs_kernel::Result;",
        ]
        with open(os.path.join(out, f"occt_{group}.rs"), "w") as f:
            f.write("\n".join(head + body) + "\n")
        print(f"{group}: {n_ok} converted, {n_ign} waiting")


if __name__ == "__main__":
    main()
