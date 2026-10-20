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
    "checktrinfo", "dtyp", "whatis", "tolerance", "checkcontinuity", "bfillds", "setflags",
    "brunparallel", "bnondestructive", "bfuzzyvalue", "bglue", "bcheckinverted", "buseobb", "bdrawwarnshapes", "bsimplify",
    "setfillhistory", "savehistory", "tclean", "incmesh", "tolblend", "checkfaults", "dversion", "dsetsignal", "testgrid", "datadir", "locate_data_file",
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
        self.objects = []
        self.tools = []
        self.n_tmp = 0
        self.todo = set()  # checks OCCT itself fails (its TODO markers): left out
        self.offset_join = "i"
        self.offset_load = None

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
        elif c in ("ellipse", "parabola") and len(a) in ((6, 9, 12) if c == "ellipse" else (5, 8, 11)):
            nv = 2 if c == "ellipse" else 1
            geo, vals = a[1:len(a) - nv], a[len(a) - nv:]
            o = f"[{L(geo[0])}, {L(geo[1])}, {L(geo[2])}]"
            n = f"Some([{L(geo[3])}, {L(geo[4])}, {L(geo[5])}])" if len(geo) >= 6 else "None"
            x = f"Some([{L(geo[6])}, {L(geo[7])}, {L(geo[8])}])" if len(geo) >= 9 else "None"
            if c == "ellipse":
                self.bind(a[0], f"k.ellipse({o}, {n}, {x}, {L(vals[0])}, {L(vals[1])})")
            else:
                self.bind(a[0], f"k.parabola({o}, {n}, {x}, {L(vals[0])})?")
        elif c == "beziercurve" and len(a) >= 2:
            np_ = int(self.num(a[1]))
            rest = a[2:]
            if np_ <= 0 or len(rest) % np_ or len(rest) // np_ not in (3, 4):
                raise Unsupported("beziercurve with these arguments")
            st = len(rest) // np_
            poles = [f"[{L(rest[i * st])}, {L(rest[i * st + 1])}, {L(rest[i * st + 2])}]" for i in range(np_)]
            w = f"Some(&[{', '.join(L(rest[i * st + 3]) for i in range(np_))}])" if st == 4 else "None"
            self.bind(a[0], f"k.beziercurve(&[{', '.join(poles)}], {w})?")
        elif c in ("bsplinecurve", "pbsplinecurve") and len(a) >= 3:
            deg, nk = int(self.num(a[1])), int(self.num(a[2]))
            ks = a[3:3 + 2 * nk]
            if len(ks) != 2 * nk:
                raise Unsupported(f"{c} with these arguments")
            knots = [L(ks[2 * i]) for i in range(nk)]
            mults = [int(self.num(ks[2 * i + 1])) for i in range(nk)]
            rest = a[3 + 2 * nk:]
            if len(rest) % 4:
                raise Unsupported(f"{c} with these arguments")
            poles = [f"[{', '.join(L(x) for x in rest[i:i + 4])}]" for i in range(0, len(rest), 4)]
            per = "true" if c == "pbsplinecurve" else "false"
            self.bind(a[0], f"k.bsplinecurve({deg}, &[{', '.join(knots)}], &[{', '.join(map(str, mults))}], &[{', '.join(poles)}], {per})?")
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
            self.bind(a[1], f"k.add({self.shape(a[0])}, {self.shape(a[1])})?")
        elif c == "profile" and len(a) >= 1:
            face, code, i = "None", [], 1
            while i < len(a):
                w = a[i]
                if w.upper() == "S" and i + 1 < len(a):
                    face = f"Some({self.shape(a[i + 1])})"
                    i += 2
                    continue
                if w.upper() in ("O", "P", "F", "X", "Y", "L", "XX", "YY", "T", "TT", "R", "RR", "D", "IX", "IY", "C", "W", "WW") and w not in self.consts:
                    code.append(w.upper())
                else:
                    code.append(repr(self.num(w)))
                i += 1
            self.bind(a[0], f'k.profile({face}, "{" ".join(code)}")?')
        elif c == "thrusections" and len(a) >= 5:
            args = [x for x in a if x not in ("-safe", "-N")]
            if any(x.startswith("-") and not re.fullmatch(r"-?[0-9.eE+]+", x) for x in args):
                raise Unsupported("thrusections with options")
            secs = ", ".join(self.shape(x) for x in args[3:])
            self.bind(args[0], f"k.thrusections({'true' if self.num(args[1]) else 'false'}, {'true' if self.num(args[2]) else 'false'}, &[{secs}])?")
        elif c == "compound" and len(a) >= 1:
            self.bind(a[-1], f"k.compound_shapes(&[{', '.join(self.shape(x) for x in a[:-1])}])")
        elif c == "offsetparameter" and len(a) >= 3:
            if a[1] != "c":
                raise Unsupported("offsetparameter in pipe mode")
            self.offset_join = a[2]
        elif c == "offsetload" and len(a) >= 2:
            self.offset_load = (a[0], a[1], a[2:])
        elif c == "offsetperform" and len(a) == 1:
            if self.offset_load is None:
                raise Unsupported("offsetperform without offsetload")
            s_, d_, fs = self.offset_load
            arc = "true" if self.offset_join == "a" else "false"
            self.bind(a[0], f"k.offsetperform({self.shape(s_)}, {L(d_)}, &[{', '.join(self.shape(f) for f in fs)}], {arc})?")
        elif c == "depouille" and len(a) >= 13 and (len(a) - 5) % 8 == 0:
            groups = []
            for i in range(5, len(a), 8):
                g = a[i:i + 8]
                groups.append(f"({self.shape(g[0])}, {L(g[1])}, [{L(g[2])}, {L(g[3])}, {L(g[4])}], [{L(g[5])}, {L(g[6])}, {L(g[7])}])")
            self.bind(a[0], f"k.depouille({self.shape(a[1])}, [{L(a[2])}, {L(a[3])}, {L(a[4])}], &[{', '.join(groups)}])?")
        elif c == "box" and len(a) in (4, 7):
            if len(a) == 4:
                self.bind(a[0], f"k.box_({', '.join(L(x) for x in a[1:])})?")
            else:
                self.bind(a[0], f"k.box_at({', '.join(L(x) for x in a[1:])})?")
        elif c in ("pcylinder", "pcone", "psphere", "ptorus"):
            want = {"pcylinder": 2, "pcone": 3, "psphere": 1, "ptorus": 2}[c]
            rest = a[1:]
            pl = None
            if rest and rest[0] in self.vars:
                pl, rest = rest[0], rest[1:]
            nums = [L(x) for x in rest]
            ax = f"Some({self.shape(pl)})" if pl else "None"
            if len(rest) == want:
                if pl:
                    self.bind(a[0], f"k.{c}_on({self.shape(pl)}, {', '.join(nums)})?")
                else:
                    self.bind(a[0], f"k.{c}({', '.join(nums)})?")
            elif len(rest) == want + 1 and c != "psphere" or (c == "psphere" and len(rest) == 2):
                self.bind(a[0], f"k.{c}_a({ax}, {', '.join(nums)})?")
            elif c == "ptorus" and len(rest) in (4, 5):
                ang = nums[4] if len(rest) == 5 else "360.0"
                self.bind(a[0], f"k.ptorus_zone({ax}, {nums[0]}, {nums[1]}, {nums[2]}, {nums[3]}, {ang})?")
            elif c == "psphere" and len(rest) in (3, 4):
                ang = nums[3] if len(rest) == 4 else "360.0"
                self.bind(a[0], f"k.psphere_zone({ax}, {nums[0]}, {nums[1]}, {nums[2]}, {ang})?")
            else:
                raise Unsupported(f"{c} with these arguments")
        elif c == "prism" and len(a) == 5:
            self.bind(a[0], f"k.prism({self.shape(a[1])}, {L(a[2])}, {L(a[3])}, {L(a[4])})?")
        elif c == "revol" and len(a) == 9:
            self.bind(a[0], f"k.revol({self.shape(a[1])}, {', '.join(L(x) for x in a[2:])})?")
        elif c in ("bfuse", "bcut", "bcommon") and len(a) == 3:
            self.bind(a[0], f"k.{c}({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "bclearobjects":
            self.objects = []
        elif c == "bcleartools":
            self.tools = []
        elif c == "baddobjects":
            self.objects += [self.shape(x) for x in a]
        elif c == "baddtools":
            self.tools += [self.shape(x) for x in a]
        elif c == "bbop" and len(a) == 2:
            if not self.objects or not self.tools:
                raise Unsupported("bbop without objects and tools")
            def fused(lst, tag):
                acc = lst[0]
                for i, x in enumerate(lst[1:]):
                    t = f"{tag}_{self.n_tmp}"
                    self.n_tmp += 1
                    self.lines.append(f"let {t} = k.bfuse({acc}, {x})?;")
                    acc = "&" + t
                return acc
            x, y = fused(self.objects, "objs"), fused(self.tools, "tools")
            op = a[1]
            f = {"0": ("bcommon", x, y), "1": ("bfuse", x, y), "2": ("bcut", x, y), "3": ("bcut", y, x)}.get(op)
            if not f:
                raise Unsupported(f"bbop {op}")
            self.bind(a[0], f"k.{f[0]}({f[1]}, {f[2]})?")
        elif c == "bop" and len(a) == 2:
            self.bop_args = (self.shape(a[0]), self.shape(a[1]))
        elif c == "bsection" and len(a) == 3:
            self.bind(a[0], f"k.bsection({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "bopsection" and len(a) == 1:
            if not self.bop_args:
                raise Unsupported("bopsection without bop")
            x, y = self.bop_args
            self.bind(a[0], f"k.bsection({x}, {y})?")
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
        elif c == "explode" and (len(a) == 1 or (len(a) == 2 and a[1].lower() in ("v", "vertex", "w", "wire", "sh", "shell", "so", "solid", "edge", "face"))):
            kind = a[1].lower() if len(a) == 2 else ""
            vec = ident(a[0]) + "_x" + (kind or "sub")
            self.lines.append(f"let {vec} = k.explode_kind({self.shape(a[0])}, {kind!r})?;".replace("'", '"'))
            self.exploded[a[0]] = (vec, kind)
        elif c in ("explode", "nexplode") and len(a) == 2 and a[1].lower() in ("e", "f"):
            kind = a[1].lower()
            vec = ident(a[0]) + ("_edges" if kind == "e" else "_faces")
            fn = ("n" if c == "nexplode" else "") + ("explode_edges" if kind == "e" else "explode_faces")
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
                if o in ("-s", "-v", "-l") and i + 1 < len(a) and o in self.todo:
                    i += 2
                    done = True
                elif o in ("-s", "-v", "-l") and i + 1 < len(a):
                    if a[i + 1] == "empty":
                        self.lines.append(f"k.checkprops_empty({s})?;")
                    else:
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
            if "nb" in self.todo:
                return
            self.lines.append(f"draw::check_nb(k.nbshapes({s})?, {args})?;")
        elif c == "checkshape":
            if "shape" in self.todo:
                return
            self.lines.append(f"k.checkshape({self.shape(a[0])})?;")
        elif c == "checkfreebounds" and len(a) >= 2:
            if len(a) > 2:
                raise Unsupported("checkfreebounds with options")
            if "free" in self.todo:
                return
            self.lines.append(f"k.checkfreebounds({self.shape(a[0])}, {int(self.num(a[1]))})?;")
        elif c == "sewing" and len(a) >= 2:
            # sewing result [tol] [options] shapes…
            rest = a[1:]
            tol = 1e-6
            try:
                tol = self.num(rest[0])
                rest = rest[1:]
            except Unsupported:
                pass
            shapes = [w for w in rest if not w.startswith("-") and not w.startswith("+")]
            if not shapes:
                raise Unsupported("sewing without shapes")
            args = ", ".join(self.shape(w) for w in shapes)
            self.bind(a[0], f"k.sewing({repr(float(tol))}, &[{args}])?")
        elif c == "restore" and len(a) >= 2:
            if not a[0].startswith("@data:"):
                raise Unsupported("restore of a file that isn't in OCCT's data")
            name = a[0][len("@data:"):]
            path = DATA.get(name)
            if path is None:
                raise Unsupported(f"the data file {name}")
            if b"CASCADE Topology V" not in open(path, "rb").read(400):
                raise Unsupported(f"reading {name}")
            self.bind(a[1], f'k.restore("{path}")?')
        else:
            raise Unsupported(c)

    def end_checks(self, tvars):
        """The checks of the group's end script (it runs on command output, so it isn't flattened):
        the offset group's mkoffset of `s` by its length times off_param."""
        if tvars.get("command") == "mkoffset" and "s" in self.vars and "off_param" in tvars:
            off = float(tvars["off_param"])
            self.lines.append(f"let sm = k.lprops({self.shape('s')})?;")
            self.lines.append(f"let result = k.mkoffset({self.shape('s')}, sm * {off!r})?;")
            self.vars["result"] = "result"
            if "length" in tvars and "-l" not in self.todo:
                if tvars["length"] == "empty":
                    self.lines.append("k.checkprops_empty(&result)?;")
                else:
                    self.lines.append(f"k.checkprops_l(&result, {self.num(tvars['length'])!r})?;")
            if "nb" not in self.todo:
                nb = [tvars.get(x) for x in ("nbsh_v", "nbsh_e", "nbsh_w")]
                if any(x is not None for x in nb):
                    args = ", ".join(f"Some({int(x)})" if x is not None else "None" for x in nb)
                    self.lines.append(f"k.check_offset_counts(&result, {args})?;")

    def raise_(self, name):
        raise Unsupported(f"the curve {name}")


def tokens(cmd):
    if any(ch in cmd for ch in "{}[]$\""):
        raise Unsupported("Tcl substitution or blocks")
    return cmd.split()


def tcl_list(text):
    """The words of a Tcl list (braces group, backslashes escape, quotes group)."""
    words, i, n = [], 0, len(text)
    while i < n:
        while i < n and text[i].isspace():
            i += 1
        if i >= n:
            break
        if text[i] == "{":
            depth, j = 1, i + 1
            while j < n and depth:
                if text[j] == "\\":
                    j += 2
                    continue
                depth += {"{": 1, "}": -1}.get(text[j], 0)
                j += 1
            words.append(text[i + 1:j - 1])
            i = j
        elif text[i] == '"':
            j, w = i + 1, []
            while j < n and text[j] != '"':
                if text[j] == "\\" and j + 1 < n:
                    w.append(text[j + 1])
                    j += 2
                    continue
                w.append(text[j])
                j += 1
            words.append("".join(w))
            i = j + 1
        else:
            w = []
            while i < n and not text[i].isspace():
                if text[i] == "\\" and i + 1 < n:
                    w.append(text[i + 1])
                    i += 2
                    continue
                w.append(text[i])
                i += 1
            words.append("".join(w))
    return words


FLATTEN = os.path.join(os.path.dirname(os.path.abspath(__file__)), "occt_flatten.tcl")


def flatten(files):
    """The DRAW commands the scripts run, loops and substitutions expanded (tclsh)."""
    import subprocess
    try:
        r = subprocess.run(["tclsh", FLATTEN, *files], capture_output=True, text=True, timeout=10)
    except subprocess.TimeoutExpired:
        raise Unsupported("Tcl evaluation (it doesn't finish)")
    if r.returncode != 0:
        first = (r.stderr.strip().splitlines() or ["?"])[0][:80]
        raise Unsupported(f"Tcl evaluation ({first})")
    cmds = [tcl_list(l[4:]) for l in r.stdout.splitlines() if l.startswith("CMD ")]
    tvars = dict(l[4:].split(" ", 1) for l in r.stdout.splitlines() if l.startswith("VAR ") and " " in l[4:])
    return cmds, tvars


def convert(path, consts, begins=()):
    text = open(path, encoding="utf-8", errors="replace").read()
    if re.search(r"stepread|igesread|testreadstep|ReadStep|readstl|brestore", text):
        return None, "needs reading STEP, IGES or STL", text
    if "locate_data_file" in text and not DATA:
        return None, "needs OCCT's data files (tools/occt_to_rust.py --data <dir>)", text
    case = Case(consts)
    # OCCT's TODO markers on all platforms (or Linux): the checks OCCT doesn't meet itself.
    for m in re.finditer(r'^\s*puts\s+"TODO\s+(?:[^\s:]+\s+)?(?:ALL|All|all|Linux)\s*:(.*)', text, re.M):
        what = m.group(1).lower()
        if "area" in what:
            case.todo.add("-s")
        if "volume" in what:
            case.todo.add("-v")
        if "length" in what:
            case.todo.add("-l")
        if ("number of" in what and "wrong" in what) or "resulting shape is wrong" in what:
            case.todo.add("nb")
        if "faults" in what or "faulty shapes" in what:
            case.todo.add("shape")
        if "free edges" in what:
            case.todo.add("free")
    if re.search(r'^\s*puts\s+"TODO\s+\S+\s+(?:ALL|All|all|Linux)\s*:.*(cannot be built|is not done)', text, re.M):
        return None, "OCCT can't build it either (its TODO)", text
    try:
        cmds, tvars = flatten([*begins, path])
        for words in cmds:
            if not words or words[0] in IGNORED:
                continue
            case.emit(words)
        case.end_checks(tvars)
    except Unsupported as e:
        return None, f"needs {e}", text.replace("[locate_data_file ", "[locate_data_file ")
    except KeyError as e:
        return None, f"needs the name {e}", text
    if not any(l.startswith("k.check") or l.startswith("draw::check") for l in case.lines):
        return None, "checks nothing we measure", text
    # Every result is also checked for validity (OCCT's scripts mostly measure only), unless
    # OCCT's own TODO says its result is faulty.
    if "result" in case.vars and "shape" not in case.todo and not any(l.startswith("k.checkshape(&result)") for l in case.lines):
        case.lines.append("k.checkshape(&result)?;")
    return case.lines, None, text


def group_consts(tests, group):
    consts = {}
    for f in (os.path.join(tests, group, "begin"),):
        if os.path.exists(f):
            for m in re.finditer(r"^\s*dset\s+(\w+)\s+([-0-9.eE+]+)", open(f).read(), re.M):
                consts[m.group(1)] = float(m.group(2))
    return consts


def known_failures():
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "occt_known_failures.txt")
    out = set()
    if os.path.exists(path):
        for line in open(path):
            line = line.strip()
            if line and not line.startswith("#"):
                g, n = line.split()
                out.add((g, n))
    return out


# Groups whose known failures are slow to run: they run only with --ignored.
SLOW_GROUPS = {"blend", "offset"}

# OCCT's test data by file name (`--data <dir>`: the unpacked opencascade-dataset).
DATA = {}


def main():
    args = sys.argv[1:]
    if "--data" in args:
        i = args.index("--data")
        for root, _, files in os.walk(os.path.abspath(args[i + 1])):
            for f in files:
                DATA.setdefault(f, os.path.join(root, f))
        del args[i:i + 2]
    tests, out, groups = args[0], args[1], args[2:]
    known = known_failures()
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
                begins = [b for b in (os.path.join(gdir, "begin"), os.path.join(gdir, grid, "begin")) if os.path.isfile(b)]
                lines, why, text = convert(p, gconsts, begins)
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
                    if (group, name) in known:
                        # Two-sided: a known failure must still fail (and says so when it
                        # passes). Slow groups run them only with --ignored.
                        if group in SLOW_GROUPS:
                            body.append('#[ignore = "known failure, slow (tools/occt_known_failures.txt)"]')
                        body.append(f"fn {name}() {{")
                        body.append("    let run = || -> Result<()> {")
                        body.append("        #[allow(unused_mut)]")
                        body.append("        let mut k = NativeKernel::new();")
                        for l in lines:
                            body.append("        " + l)
                        body.append("        Ok(())")
                        body.append("    };")
                        body.append("    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(run));")
                        body.append("    if let Ok(Err(e)) = &r {")
                        body.append('        eprintln!("known failure: {e:?}");')
                        body.append("    }")
                        body.append("    let passed = matches!(r, Ok(Ok(())));")
                        body.append(f'    assert!(!passed, "a known failure now passes: remove `{group} {name}` from tools/occt_known_failures.txt");')
                        body.append("}")
                        continue
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
            "#![allow(unused_variables, unused_imports, non_snake_case, clippy::all)]",
            "",
            "use cadrs_kernel::native::{draw, NativeKernel};",
            "use cadrs_kernel::Result;",
        ]
        with open(os.path.join(out, f"occt_{group}.rs"), "w") as f:
            f.write("\n".join(head + body) + "\n")
        print(f"{group}: {n_ok} converted, {n_ign} waiting")


if __name__ == "__main__":
    main()
