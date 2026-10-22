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
    "setfillhistory", "tclean", "incmesh", "tolblend", "checkfaults", "dversion", "dsetsignal", "testgrid", "datadir", "locate_data_file",
    "fsameparameter", "encoderegularity", "bopcheck", "don",
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
        self.hyperbolas = set()  # names of hyperbolas (an edge of one takes its parameters)
        self.curves = set()  # names of curves
        self.surfs = set()  # names of surfaces
        self.planes = set()  # names of placements (`plane`)
        self.sweep = None  # mksweep's spine, setsweep's mode and the sections added
        self.evol = None  # mkevol's result name, shape and the edges' radius laws
        self.feat = None  # the form feature featprism/featrevol/featdprism defined
        self.eval_compounds = {}  # `eval compound [explode s e] … name`: name -> the shapes
        self.object_names, self.tool_names = [], []
        self.cells = None  # bcbuild's arguments: (expressions, names)
        self.cells_result = None  # the name bcadd builds the result under
        self.face_surfs = {}  # faces of whole surfaces (mkface without bounds): name -> surface
        self.surfaces = {}  # mksurface's surfaces of faces: name -> the face
        self.offset_history = False  # an offset ran (its history is what savehistory saves)
        self.history_queries = False  # the script asks the history (isdeleted, generated)
        self.isdeleted_wants = {}  # isdeleted's shapes: name -> whether the script wants it deleted

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
        self.curves.discard(name)
        self.surfs.discard(name)
        self.hyperbolas.discard(name)
        self.planes.discard(name)
        self.lines.append(f"let {r} = {expr};")

    def surf(self, name):
        """A Rust expression for a DRAW surface (a surface, or a plane's placement)."""
        if name in self.surfs:
            return "&" + self.vars[name]
        if name in self.planes:
            return f"&k.plane_surface(&{self.vars[name]})"
        raise Unsupported(f"the surface {name}")

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
            self.curves.add(a[0])
        elif c == "circle" and len(a) == 5:
            self.bind(a[0], f"k.circle({', '.join(L(x) for x in a[1:])})")
            self.curves.add(a[0])
        elif c == "circle" and len(a) == 8:
            self.bind(a[0], f"k.circle_n({', '.join(L(x) for x in a[1:])})")
            self.curves.add(a[0])
        elif c == "circle" and len(a) == 11:
            self.bind(a[0], f"k.circle_nx({', '.join(L(x) for x in a[1:])})")
            self.curves.add(a[0])
        elif c == "plane" and len(a) in (4, 7, 10):
            nums = [L(x) for x in a[1:]]
            if len(a) == 4:
                nums += ["0.0", "0.0", "1.0"]
            if len(nums) == 6:
                self.bind(a[0], f"k.plane({', '.join(nums)})")
            else:
                self.bind(a[0], f"k.plane_x({', '.join(nums)})")
            self.vars[a[0]] = ident(a[0])
            self.planes.add(a[0])
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
            self.curves.add(a[0])
        elif c == "beziercurve" and len(a) >= 2:
            np_ = int(self.num(a[1]))
            rest = a[2:]
            if np_ <= 0 or len(rest) % np_ or len(rest) // np_ not in (3, 4):
                raise Unsupported("beziercurve with these arguments")
            st = len(rest) // np_
            poles = [f"[{L(rest[i * st])}, {L(rest[i * st + 1])}, {L(rest[i * st + 2])}]" for i in range(np_)]
            w = f"Some(&[{', '.join(L(rest[i * st + 3]) for i in range(np_))}])" if st == 4 else "None"
            self.bind(a[0], f"k.beziercurve(&[{', '.join(poles)}], {w})?")
            self.curves.add(a[0])
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
            self.curves.add(a[0])
        elif c == "hyperbola" and len(a) in (6, 9, 12):
            geo, vals = a[1:len(a) - 2], a[len(a) - 2:]
            o = f"[{L(geo[0])}, {L(geo[1])}, {L(geo[2])}]"
            n = f"Some([{L(geo[3])}, {L(geo[4])}, {L(geo[5])}])" if len(geo) >= 6 else "None"
            x = f"Some([{L(geo[6])}, {L(geo[7])}, {L(geo[8])}])" if len(geo) >= 9 else "None"
            self.bind(a[0], f"k.hyperbola({o}, {n}, {x}, {L(vals[0])}, {L(vals[1])})")
            self.hyperbolas.add(a[0])
        elif c == "mkedge" and len(a) == 4 and a[1] in self.hyperbolas:
            self.bind(a[0], f"k.mkedge_hyperbola(&{self.vars[a[1]]}, {L(a[2])}, {L(a[3])})?")
            self.hyperbolas.discard(a[0])
        elif c == "mkedge" and len(a) == 2 and a[1] in self.hyperbolas:
            raise Unsupported("a whole hyperbola")
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
        elif c == "DT_SplitByNumber" and len(a) in (3, 4):
            if len(a) == 3:
                self.bind(a[0], f"k.split_by_number({self.shape(a[1])}, {int(self.num(a[2]))}, None)?")
            else:
                self.bind(a[0], f"k.split_by_number({self.shape(a[1])}, 0, Some(({int(self.num(a[2]))}, {int(self.num(a[3]))})))?")
        elif c == "polyvertex" and len(a) >= 3:
            self.bind(a[0], f"k.polyvertex(&[{', '.join(self.shape(x) for x in a[1:])}])?")
        elif c == "fixshape" and len(a) >= 2:
            ang, wid, i = 0.0, -1.0, 2
            while i < len(a):
                if a[i] == "-maxtaila" and i + 1 < len(a):
                    ang = self.num(a[i + 1])
                elif a[i] == "-maxtailw" and i + 1 < len(a):
                    wid = self.num(a[i + 1])
                else:
                    raise Unsupported(f"fixshape {a[i]}")
                i += 2
            if wid < 0:
                raise Unsupported("fixshape without its tail fix")
            self.bind(a[0], f"k.fixshape_tails({self.shape(a[1])}, {ang!r}, {wid!r})?")
        elif c == "halfspace" and len(a) == 5:
            self.bind(a[0], f"k.halfspace({self.shape(a[1])}, {', '.join(L(x) for x in a[2:])})?")
        elif c == "mkplane" and len(a) == 3:
            # (OnlyPlane: our faces of wires are planar ones anyway.)
            self.bind(a[0], f"k.mkplane({self.shape(a[1])})?")
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
        elif c == "compound" and len(a) == 1 and a[0] in self.eval_compounds:
            # `eval compound [explode s e] … name`: the sub-shapes of each explode.
            vecs = []
            for x in self.eval_compounds[a[0]]:
                if x not in self.exploded:
                    raise Unsupported(f"the shape {x}")
                vecs.append(self.exploded[x][0])
            parts = " ".join(f".chain({v}.iter())" for v in vecs[1:])
            self.bind(a[0], f"draw::Shape::Compound({vecs[0]}.iter(){parts}.cloned().collect())")
        elif c == "edgestofaces" and len(a) == 2:
            self.bind(a[0], f"k.edgestofaces({self.shape(a[1])})?")
        elif c == "compound" and len(a) >= 1:
            self.bind(a[-1], f"k.compound_shapes(&[{', '.join(self.shape(x) for x in a[:-1])}])")
        elif c == "offsetparameter" and len(a) >= 3:
            if a[1] != "c":
                raise Unsupported("offsetparameter in pipe mode")
            self.offset_join = a[2]
        elif c == "offsetload" and len(a) >= 2:
            self.offset_load = (a[0], a[1], a[2:])
            self.offset_on = []
        elif c == "offsetonface" and len(a) >= 2 and len(a) % 2 == 0:
            if self.offset_load is None:
                raise Unsupported("offsetonface without offsetload")
            self.offset_on += [(a[i], a[i + 1]) for i in range(0, len(a), 2)]
        elif c == "offsetperform" and len(a) == 1:
            self.offset_history = True
            if self.offset_load is None:
                raise Unsupported("offsetperform without offsetload")
            s_, d_, fs = self.offset_load
            arc = "true" if self.offset_join == "a" else "false"
            on = getattr(self, "offset_on", [])
            if on:
                if fs:
                    raise Unsupported("offsetonface with faces removed")
                own = ", ".join(f"({self.shape(f)}, {L(x)})" for f, x in on)
                self.bind(a[0], f"k.offsetperform_on({self.shape(s_)}, {L(d_)}, &[{own}], {arc})?")
            else:
                self.bind(a[0], f"k.offsetperform({self.shape(s_)}, {L(d_)}, &[{', '.join(self.shape(f) for f in fs)}], {arc})?")
        elif c == "savehistory" and len(a) == 1:
            # An offset's history, when the script asks it something (else nothing to do).
            if self.offset_history and self.history_queries:
                self.bind(a[0], "k.savehistory()?")
        elif c == "isdeleted" and len(a) == 2:
            if not (self.offset_history and a[1] in self.isdeleted_wants):
                raise Unsupported("isdeleted")
            self.lines.append(f"k.isdeleted({self.shape(a[0])}, {self.shape(a[1])}, {'true' if self.isdeleted_wants[a[1]] else 'false'})?;")
        elif c == "generated" and len(a) == 3:
            if not self.offset_history:
                raise Unsupported("generated")
            self.bind(a[0], f"k.generated({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "offsetshape" and len(a) >= 3:
            # `offsetshape r s d [tol] [face ...]`: as offsetcompshape (OCCT runs both through one
            # function, arc joins), its optional tolerance skipped.
            rest = a[3:]
            try:
                float(rest[0])
                rest = rest[1:]
            except (IndexError, ValueError):
                pass
            self.bind(a[0], f"k.offsetcompshape({self.shape(a[1])}, {L(a[2])}, &[{', '.join(self.shape(f) for f in rest)}], false)?")
        elif c == "offsetcompshape" and len(a) >= 3:
            self.offset_history = True
            opts = [x for x in a[3:] if x.startswith("-")]
            if any(x not in ("-i", "-t") for x in opts):
                raise Unsupported("offsetcompshape options")
            fs = [x for x in a[3:] if not x.startswith("-")]
            inter = "true" if "-i" in opts else "false"
            self.bind(a[0], f"k.offsetcompshape({self.shape(a[1])}, {L(a[2])}, &[{', '.join(self.shape(f) for f in fs)}], {inter})?")
        elif c == "thickshell" and 3 <= len(a) <= 5:
            if len(a) >= 4 and a[3] not in ("i", "a"):
                raise Unsupported("thickshell join type")
            self.bind(a[0], f"k.thickshell({self.shape(a[1])}, {L(a[2])})?")
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
        elif c == "pipe" and len(a) in (3, 4, 5):
            self.bind(a[0], f"k.pipe({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "mksweep" and len(a) == 1:
            self.sweep = {"spine": self.shape(a[0]), "mode": "CF", "dir": "[0.0, 0.0, 1.0]", "sections": []}
        elif c == "setsweep" and len(a) >= 1:
            if self.sweep is None:
                raise Unsupported("setsweep without mksweep")
            o = a[0]
            if o in ("-FR", "-CF", "-DT") and len(a) == 1:
                self.sweep["mode"] = o[1:]
            elif o == "-CN" and len(a) == 4:
                self.sweep["mode"] = "CN"
                self.sweep["dir"] = f"[{L(a[1])}, {L(a[2])}, {L(a[3])}]"
            elif o == "-FX" and len(a) in (4, 7):
                self.sweep["mode"] = "FX"
            elif o in ("-SM", "-DM") and len(a) == 2:
                pass  # the approximation's degree and segments: our sweeps are exact
            else:
                raise Unsupported(f"setsweep {o}")
        elif c == "addsweep" and len(a) >= 1:
            if self.sweep is None:
                raise Unsupported("addsweep without mksweep")
            rest = a[1:]
            if rest and not rest[0].startswith("-"):
                raise Unsupported("addsweep at a vertex or with a law")
            t, r = "-T" in rest, "-R" in rest
            if any(x not in ("-T", "-R") for x in rest):
                raise Unsupported("addsweep with a law")
            self.sweep["sections"].append(f"({self.shape(a[0])}, {'true' if t else 'false'}, {'true' if r else 'false'})")
        elif c == "buildsweep" and len(a) >= 1:
            if self.sweep is None or not self.sweep["sections"]:
                raise Unsupported("buildsweep without mksweep and addsweep")
            opts = a[1:]
            tr = "M"
            if opts and opts[0] in ("-M", "-C", "-R"):
                tr = opts[0][1]
                opts = opts[1:]
            solid = bool(opts) and opts[0] == "-S"
            sw = self.sweep
            self.bind(a[0], f"k.buildsweep({sw['spine']}, &[{', '.join(sw['sections'])}], \"{sw['mode']}\", {sw['dir']}, '{tr}', {'true' if solid else 'false'})?")
        elif c == "mkevol" and len(a) in (2, 3):
            self.evol = (a[0], self.shape(a[1]), [])
        elif c == "updatevol" and len(a) >= 3 and len(a) % 2 == 1:
            if self.evol is None:
                raise Unsupported("updatevol without mkevol")
            pairs = [f"({L(a[i])}, {L(a[i + 1])})" for i in range(1, len(a), 2)]
            self.evol[2].append(f"({self.shape(a[0])}, &[{', '.join(pairs)}][..])")
        elif c == "buildevol" and len(a) == 0:
            if self.evol is None or not self.evol[2]:
                raise Unsupported("buildevol without mkevol and updatevol")
            name, s_, laws = self.evol
            self.bind(name, f"k.buildevol({s_}, &[{', '.join(laws)}])?")
        elif c == "extsurf" and len(a) == 5 and a[1] in self.curves:
            self.bind(a[0], f"k.extsurf(&{self.vars[a[1]]}, {', '.join(L(x) for x in a[2:])})?")
            self.surfs.add(a[0])
        elif c == "revsurf" and len(a) == 8 and a[1] in self.curves:
            self.bind(a[0], f"k.revsurf(&{self.vars[a[1]]}, {', '.join(L(x) for x in a[2:])})?")
            self.surfs.add(a[0])
        elif c in ("cylinder", "cone", "sphere", "torus") and len(a) >= 2:
            nv = {"cylinder": 1, "cone": 2, "sphere": 1, "torus": 2}[c]
            geo, vals = a[1:len(a) - nv], a[len(a) - nv:]
            if len(geo) not in (0, 3, 6, 9):
                raise Unsupported(f"{c} with these arguments")
            o = f"[{L(geo[0])}, {L(geo[1])}, {L(geo[2])}]" if geo else "[0.0, 0.0, 0.0]"
            n = f"Some([{L(geo[3])}, {L(geo[4])}, {L(geo[5])}])" if len(geo) >= 6 else "None"
            x = f"Some([{L(geo[6])}, {L(geo[7])}, {L(geo[8])}])" if len(geo) >= 9 else "None"
            self.bind(a[0], f'k.elementary("{c}", {o}, {n}, {x}, &[{", ".join(L(v) for v in vals)}])?')
            self.surfs.add(a[0])
        elif c == "beziersurf" and len(a) >= 3:
            nu, nv = int(self.num(a[1])), int(self.num(a[2]))
            rest = a[3:]
            if nu * nv == 0 or len(rest) % (nu * nv) or len(rest) // (nu * nv) not in (3, 4):
                raise Unsupported("beziersurf with these arguments")
            st = len(rest) // (nu * nv)
            poles = [f"[{L(rest[i * st])}, {L(rest[i * st + 1])}, {L(rest[i * st + 2])}, {L(rest[i * st + 3]) if st == 4 else '1.0'}]" for i in range(nu * nv)]
            self.bind(a[0], f"k.beziersurf({nu}, {nv}, &[{', '.join(poles)}])?")
            self.surfs.add(a[0])
        elif c in ("bsplinesurf", "upbsplinesurf", "vpbsplinesurf", "uvpbsplinesurf") and len(a) >= 3:
            uper, vper = c.startswith("u"), c.startswith("v") or c.startswith("uv")
            i = 1
            def knots(i):
                deg, nk = int(self.num(a[i])), int(self.num(a[i + 1]))
                ks = a[i + 2:i + 2 + 2 * nk]
                if len(ks) != 2 * nk:
                    raise Unsupported(f"{c} with these arguments")
                return deg, [L(ks[2 * j]) for j in range(nk)], [str(int(self.num(ks[2 * j + 1]))) for j in range(nk)], i + 2 + 2 * nk
            du, uk, um, i = knots(i)
            dv, vk, vm, i = knots(i)
            rest = a[i:]
            if len(rest) % 4:
                raise Unsupported(f"{c} with these arguments")
            poles = [f"[{', '.join(L(x) for x in rest[j:j + 4])}]" for j in range(0, len(rest), 4)]
            self.bind(a[0], f"k.bsplinesurf({du}, &[{', '.join(uk)}], &[{', '.join(um)}], {str(uper).lower()}, {dv}, &[{', '.join(vk)}], &[{', '.join(vm)}], {str(vper).lower()}, &[{', '.join(poles)}])?")
            self.surfs.add(a[0])
        elif c == "offset" and len(a) == 3 and (a[1] in self.surfs or a[1] in self.planes):
            self.bind(a[0], f"k.offset_surf({self.surf(a[1])}, {L(a[2])})?")
            self.surfs.add(a[0])
        elif c == "trim" and len(a) == 6 and (a[1] in self.surfs or a[1] in self.planes):
            self.bind(a[0], f"k.trim_surf({self.surf(a[1])}, {', '.join(L(x) for x in a[2:])})")
            self.surfs.add(a[0])
        elif c == "mkface" and len(a) in (2, 6) and (a[1] in self.surfs or a[1] in self.planes):
            uv = f"Some([{', '.join(L(x) for x in a[2:])}])" if len(a) == 6 else "None"
            surf = self.surf(a[1])
            if a[1] in self.planes:
                # The plane's surface held (mkvolume uses it again).
                tmp = ident(a[0]) + "_surf"
                self.lines.append(f"let {tmp} = k.plane_surface(&{self.vars[a[1]]});")
                surf = "&" + tmp
            self.bind(a[0], f"k.mkface({surf}, {uv})?")
            self.face_surfs[a[0]] = surf
        elif c == "wedge" and len(a) in (5, 8):
            nums = [L(x) for x in a[1:]]
            top = f"[0.0, 0.0, {nums[3]}, {nums[2]}]" if len(a) == 5 else f"[{', '.join(nums[3:])}]"
            self.bind(a[0], f"k.wedge({nums[0]}, {nums[1]}, {nums[2]}, {top})?")
        elif c == "mksurface" and len(a) == 2:
            self.surfaces[a[0]] = self.shape(a[1])
        elif c == "mkface" and len(a) == 2 and a[1] in self.surfaces:
            # The face's whole surface: the face itself stands for it (as a limit of a feature,
            # only its surface counts).
            self.bind(a[0], f"{self.surfaces[a[1]][1:]}.clone()")
        elif c in ("featprism", "featrevol", "featdprism") and len(a) in (6, 8, 11):
            kind = c[4:]
            want = {"prism": 8, "revol": 11, "dprism": 6}[kind]
            if len(a) != want:
                raise Unsupported(f"{c} with these arguments")
            nums = [L(x) for x in a[3:-2]] + ["0.0"] * 6
            self.feat = (kind, self.shape(a[0]), self.shape(a[1]), f"[{', '.join(nums[:6])}]", int(self.num(a[-2])))
        elif c == "featlf" and len(a) == 11:
            if a[2] not in self.planes:
                raise Unsupported(f"the plane {a[2]}")
            self.feat = ("lf", self.shape(a[0]), self.shape(a[1]), f"&{self.vars[a[2]]}, [{', '.join(L(x) for x in a[3:6])}], [{', '.join(L(x) for x in a[6:9])}]", int(self.num(a[9])))
        elif c == "addslide" and len(a) >= 3 and len(a) % 2 == 1:
            # The faces the feature slides along (gluing hints for OCCT's local operation): the
            # same shape results without them.
            if self.feat is None or a[0].lower() != self.feat[0]:
                raise Unsupported("addslide without its feature")
            for x in a[1:]:
                self.shape(x)
        elif c == "featrf" and len(a) == 13:
            if a[2] not in self.planes:
                raise Unsupported(f"the plane {a[2]}")
            self.feat = ("rf", self.shape(a[0]), self.shape(a[1]), f"&{self.vars[a[2]]}, [{', '.join(L(x) for x in a[3:9])}], {L(a[9])}, {L(a[10])}", int(self.num(a[11])))
        elif c == "featperform" and len(a) == 2 and self.feat is not None and self.feat[0] in ("lf", "rf") and a[0].lower() == self.feat[0]:
            kind, base, wire, args, fuse = self.feat
            self.bind(a[1], f"k.feat{kind}({base}, {wire}, {args}, {fuse})?")
        elif c in ("featperform", "featperformval") and len(a) >= 2:
            if self.feat is None or a[0].lower() != self.feat[0]:
                raise Unsupported(f"{c} {a[0]} without its definition")
            kind, base, pbase, params, fuse = self.feat
            if c == "featperformval":
                if len(a) == 4:
                    self.bind(a[1], f'k.featperformval_until({base}, {pbase}, "{kind}", {params}, {fuse}, {L(a[2])}, {self.shape(a[3])})?')
                    return
                if len(a) != 3:
                    raise Unsupported("featperformval with these arguments")
                self.bind(a[1], f'k.featperformval({base}, {pbase}, "{kind}", {params}, {fuse}, {L(a[2])})?')
            else:
                lim = [f"Some({self.shape(x)})" for x in a[2:]]
                if len(lim) > 2:
                    raise Unsupported("featperform with these arguments")
                fr, un = ("None", "None") if not lim else (("None", lim[0]) if len(lim) == 1 else (lim[0], lim[1]))
                self.bind(a[1], f'k.featperform({base}, {pbase}, "{kind}", {params}, {fuse}, {fr}, {un})?')
        elif c == "orientation" and len(a) == 2:
            self.bind(a[0], f"k.orientation({self.shape(a[0])}, \"{a[1].upper()}\")?")
        elif c == "treverse" and len(a) >= 1:
            for x in a:
                self.bind(x, f"k.orientation({self.shape(x)}, \"R\")?")
        elif c == "renamevar" and len(a) == 2:
            if a[0] in self.exploded:
                self.exploded[a[1]] = self.exploded[a[0]]
            elif a[0] in self.vars:
                kinds = [k for k in (self.curves, self.surfs, self.hyperbolas, self.planes) if a[0] in k]
                self.vars[a[1]] = self.vars[a[0]]
                for k in kinds:
                    k.add(a[1])
            else:
                self.bind(a[1], f"{self.shape(a[0])[1:]}.clone()")
        elif c == "revol" and len(a) == 9:
            self.bind(a[0], f"k.revol({self.shape(a[1])}, {', '.join(L(x) for x in a[2:])})?")
        elif c in ("bfuse", "bcut", "bcommon") and len(a) == 3:
            self.bind(a[0], f"k.{c}({self.shape(a[1])}, {self.shape(a[2])})?")
        elif c == "bclearobjects":
            self.objects = []
            self.object_names = []
        elif c == "bcleartools":
            self.tools = []
            self.tool_names = []
        elif c == "baddobjects":
            self.objects += [self.shape(x) for x in a]
            self.object_names += a
        elif c == "baddtools":
            self.tools += [self.shape(x) for x in a]
            self.tool_names += a
        elif c == "compound" and a and a[-1] == "TheFeatureC":
            return  # removefeatures' CheckIsFeatureRemoved (history checks we don't keep)
        elif c == "unifysamedom" and len(a) == 2:
            self.bind(a[0], f"k.unifysamedom({self.shape(a[1])})?")
        elif c == "removefeatures" and len(a) >= 3:
            self.bind(a[0], f"k.removefeatures({self.shape(a[1])}, &[{', '.join(self.shape(x) for x in a[2:])}])?")
        elif c in ("bsplit", "bapisplit") and len(a) == 1:
            if not self.objects:
                raise Unsupported(f"{c} without objects")
            self.bind(a[0], f"k.bsplit(&[{', '.join(self.objects)}], &[{', '.join(self.tools)}])?")
        elif c == "bcbuild" and len(a) == 1:
            self.cells = (self.objects + self.tools, self.object_names + self.tool_names)
            self.cells_result = None
        elif c == "bcremoveall" and len(a) == 0:
            self.cells_result = None
        elif c in ("bcadd", "bcaddall") and len(a) >= 1:
            if self.cells is None:
                raise Unsupported(f"{c} without bcbuild")
            exprs, names = self.cells
            spec = []
            mat, upd = 0, False
            if len(a) >= 3 and a[-3] == "-m":
                mat, upd, a = int(self.num(a[-2])), a[-1] == "-u", a[:-3]
            elif len(a) >= 2 and a[-2] == "-m":
                mat, a = int(self.num(a[-1])), a[:-2]
            if c == "bcadd":
                if len(a) % 2 != 1:
                    raise Unsupported("bcadd with these arguments")
                for i in range(1, len(a), 2):
                    if a[i] not in names:
                        raise Unsupported(f"bcadd of {a[i]}, not an argument")
                    spec.append(f"({names.index(a[i])}, {'true' if int(self.num(a[i + 1])) else 'false'})")
            elif len(a) > 1:
                raise Unsupported("bcaddall with options")
            prev = f"Some({self.shape(self.cells_result)})" if self.cells_result else "None"
            self.bind(a[0], f"k.bcadd(&[{', '.join(exprs)}], &[{', '.join(spec)}], {prev}, {mat}, {'true' if upd else 'false'})?")
            self.cells_result = a[0]
        elif c == "bcremoveint" and len(a) == 1:
            self.bind(a[0], f"k.bcremoveint({self.shape(a[0])})?")
        elif c == "mkvolume" and len(a) >= 2:
            names = [x for x in a[1:] if not x.startswith("-")]
            if any(x.startswith("-") for x in a[1:]):
                raise Unsupported("mkvolume with options")
            if not all(x in self.face_surfs for x in names):
                raise Unsupported("mkvolume of shapes other than faces of whole surfaces")
            self.bind(a[0], f"k.mkvolume(&[{', '.join(self.face_surfs[x] for x in names)}])?")
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
        elif c == "nexplode" and len(a) >= 2 and a[1][:1].lower() in ("e", "f", "v"):
            kind = a[1][0].lower()
            vec = ident(a[0]) + {"e": "_edges", "f": "_faces", "v": "_verts"}[kind]
            fn = {"e": "nexplode_edges", "f": "nexplode_faces", "v": "nexplode_vertices"}[kind]
            self.lines.append(f"let {vec} = k.{fn}({self.shape(a[0])})?;")
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
                pairs = [f"({L(a[i])}, {self.shape(a[i + 1])})" for i in range(2, len(a), 2)]
                self.bind(a[0], f"k.blend_radii({self.shape(a[1])}, &[{', '.join(pairs)}])?")
                return
            edges = [self.shape(a[i]) for i in range(3, len(a), 2)]
            self.bind(a[0], f"k.blend({self.shape(a[1])}, {L(a[2])}, &[{', '.join(edges)}])?")
        elif c in ("bfuseblend", "bcutblend") and len(a) in (4, 5):
            if len(a) == 5 and a[4] != "-d":
                raise Unsupported(f"{c} with these arguments")
            self.bind(a[0], f"k.bfuseblend({self.shape(a[1])}, {self.shape(a[2])}, {L(a[3])}, {'true' if c == 'bfuseblend' else 'false'})?")
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
            rel = os.path.relpath(path, DATA_ROOT[0]).replace(os.sep, "/")
            self.bind(a[1], f'k.restore(&data("{rel}"))?')
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
    # History queries: `if {[regexp "Not deleted" [isdeleted h s]]} {puts "Error…"}` wants s
    # deleted, `regexp "Deleted"` wants it kept.
    case.history_queries = bool(re.search(r"\b(isdeleted|generated)\b", text))
    for m in re.finditer(r'regexp\s+"(Not deleted|Deleted)"\s+\[isdeleted\s+\w+\s+(\w+)\]', text):
        case.isdeleted_wants[m.group(2)] = m.group(1) == "Not deleted"
    for m in re.finditer(r"eval\s+compound((?:\s*\[explode\s+\w+\s+\w+\])+)\s+(\w+)", text):
        case.eval_compounds[m.group(2)] = re.findall(r"\[explode\s+(\w+)\s+\w+\]", m.group(1))
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
        # A group whose end script checks the result's validity (and only what we can't
        # measure besides, as OCCT's tolerances): that check is the case's.
        if not (group_of(path) in END_CHECKSHAPE_GROUPS and "result" in case.vars and "shape" not in case.todo):
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


# Groups whose end script's `checkshape result` is a case's check when it measures nothing else
# (mkface: the faces made are only checked for validity and OCCT's tolerances).
END_CHECKSHAPE_GROUPS = {"mkface"}


def group_of(path):
    """The group of a case's path (…/tests/<group>/<grid>/<case>)."""
    return os.path.basename(os.path.dirname(os.path.dirname(os.path.abspath(path))))


# Cases whose reference value OCCT itself doesn't have: the volume their script checks is the
# input's own ("real volume of result shape is unknown yet"), which no thick solid can have. They
# run without that check (the result must still be built, and be valid where the script checks
# its shape); why each reference is wrong, and what ours is checked against, is noted here.
WRONG_REFERENCE = {
    # A 100-cube with a 10 × 10 lip, its x = 0 face open, walls 10 thick: the cavity is
    # 90 × 80 × 80 (the lip lies within 10 of its faces; the tube round its concave edge only
    # touches the cavity), so 1010000 − 576000 = 434000; the reference is the input's 1.01e+06.
    ("offset", "compshape_a1"): "the input's volume (ours: 434000, the closed form)",
    # OCCT's offset fails inside a `catch`; the reference is the input's 534256 (ours 67007).
    ("offset", "compshape_a4"): "the input's volume, after OCCT's command fails (ours: 67007)",
    # Walls 5 thick round a solid of 2.8e+07: the reference is the input's 28031240; ours,
    # 2128159, is 2.6 % under the faces kept × 5 (2185475), the walls' overlap at its corners.
    ("offset", "shape_a1"): "the input's volume (ours: 2128159, the kept faces' area × 5 less the corners' overlap)",
}

# Groups whose known failures are slow to run: they run only with --ignored.
SLOW_GROUPS = {"blend", "offset"}

# OCCT's test data by file name (`--data <dir>`: the unpacked opencascade-dataset).
DATA = {}
DATA_ROOT = [""]


def main():
    args = sys.argv[1:]
    if "--data" in args:
        i = args.index("--data")
        DATA_ROOT[0] = os.path.abspath(args[i + 1])
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
                    if (group, name) in WRONG_REFERENCE:
                        note = WRONG_REFERENCE[(group, name)]
                        lines = [f"// (Not checked: OCCT's reference volume is {note}.)" if "checkprops_v(" in l else l for l in lines]
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
                        body.append(f'        eprintln!("known failure {group} {name}: {{e:?}}");')
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
        if DATA_ROOT[0]:
            crate = os.path.dirname(os.path.abspath(out))
            default = os.path.relpath(DATA_ROOT[0], crate).replace(os.sep, "/")
            head += [
                "",
                "/// A file of OCCT's dataset (github.com/Open-Cascade-SAS/OCCT-data): under",
                f"/// `$CADRS_OCCT_DATA`, else `{default}` from this crate.",
                "#[allow(dead_code)]",
                "fn data(rel: &str) -> String {",
                "    let root = std::env::var(\"CADRS_OCCT_DATA\").unwrap_or_else(|_| format!(\"{}/" + default + "\", env!(\"CARGO_MANIFEST_DIR\")));",
                "    let p = format!(\"{root}/{rel}\");",
                "    assert!(std::path::Path::new(&p).exists(), \"OCCT's dataset file {p} is missing: set CADRS_OCCT_DATA to the dataset's folder\");",
                "    p",
                "}",
            ]
        with open(os.path.join(out, f"occt_{group}.rs"), "w") as f:
            f.write("\n".join(head + body) + "\n")
        print(f"{group}: {n_ok} converted, {n_ign} waiting")


if __name__ == "__main__":
    main()
