"""Helpers shared by the SMLib intersection, section, silhouette, tessellation and stitch suite
modules (not a suite: the driver skips modules starting with `_`)."""
import json
import os
import subprocess

from smlib_to_rust import rust_f64


def oracle_run(args, timeout=600):
    """SMLib's report (the oracle's JSON) for one command, or None without the oracle
    (CADRS_SMLIB_ORACLE unset)."""
    ob = os.environ.get("CADRS_SMLIB_ORACLE")
    if not ob or not os.path.exists(ob):
        return None
    r = subprocess.run([ob] + [str(a) for a in args], capture_output=True, text=True, timeout=timeout)
    lines = [l for l in r.stdout.splitlines() if l.startswith("{")]
    return json.loads(lines[-1]) if lines else {"ok": False, "status": "no output (exit %d)" % r.returncode}


def opt_f64(x):
    """`Some(x)` or `None` as Rust."""
    return "None" if x is None else "Some(%s)" % rust_f64(x)


def points_literal(pts):
    """A Rust `&[[f64; 3]]` literal."""
    return "&[%s]" % ", ".join("[%s, %s, %s]" % tuple(rust_f64(c) for c in p) for p in pts)


def want_literal(any_, length, samples):
    """An `intersect::Want` literal."""
    return "&Want { any: %s, length: %s, samples: %s }" % ("true" if any_ else "false", opt_f64(length), points_literal(samples))
