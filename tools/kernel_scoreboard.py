#!/usr/bin/env python3
"""Scoreboard of the kernel's test suites (crates/cadrs_kernel/tests).

usage: tools/kernel_scoreboard.py [--results FILE] [--static] [--detail]

Measured (the default when a result file exists): reads the result file of tools/kernel_gate.sh
(--results, else $KERNEL_GATE_RESULTS, else $CARGO_TARGET_DIR/kernel_gate/results.json, else
target/kernel_gate/results.json) and gives, per test binary, the cases run and how they ended:
passing, known failures still failing, STALE known failures (they pass now), STALE-IGNOREs
(ignored tests that pass now), REGRESSIONs, HANGs, CRASHes and SLOW-REGRESSIONs (much slower
than the gate's baseline), the cases not run and why (the top ignore reasons), and the slow ones.

Static (--static, or no result file): reads the generated suites' source only and runs nothing.
It counts a case as passing when it isn't ignored and isn't a known failure, so it can't see a
regression, a stale known failure or a hang: the gate is the scoreboard; this is a fallback.
"""
import collections
import glob
import json
import os
import re
import subprocess
import sys

here = os.path.dirname(os.path.abspath(__file__))
repo = os.path.dirname(here)
tests = os.path.join(repo, "crates/cadrs_kernel/tests")
args = sys.argv[1:]
detail = "--detail" in args


def bucket(reason):
    if reason.startswith("known failure"):
        return "known failure"
    if reason.startswith("needs the data file"):
        return "missing data file"
    m = re.match(r"needs (\S+)", reason)
    if m and not reason.startswith("needs Tcl"):
        return "needs " + m.group(1)
    return reason or "ignored"


def results_path():
    if "--results" in args:
        return args[args.index("--results") + 1]
    if os.environ.get("KERNEL_GATE_RESULTS"):
        return os.environ["KERNEL_GATE_RESULTS"]
    for t in (os.environ.get("CARGO_TARGET_DIR"), "target"):
        if t:
            p = os.path.join(t if os.path.isabs(t) else os.path.join(repo, t), "kernel_gate/results.json")
            if os.path.exists(p):
                return p
    return None


def rate(c):
    return f"{100.0 * (c['pass'] + c['STALE'] + c['STALE-IGNORE']) / c['run']:.1f}" if c["run"] else "-"


def measured(path):
    res = json.load(open(path))
    r = subprocess.run(["git", "-C", repo, "rev-parse", "--verify", "-q", "HEAD"], capture_output=True, text=True)
    head = r.stdout.strip() if r.returncode == 0 else ""
    print(f"MEASURED by tools/kernel_gate.sh: {path}")
    print(f"commit {res.get('commit') or '?'}, finished {res.get('finished') or 'no (partial results)'}, "
          f"hang limit {res.get('hang_limit')} s, {res.get('threads')} test threads")
    if res.get("commit") and head and not res["commit"].startswith(head):
        print(f"note: HEAD is {head}, not the commit measured")
    print()
    slow = res.get("slow_limit", 60.0)
    by = collections.defaultdict(list)
    for t in res["tests"]:
        by[t["binary"]].append(t)
    print(f"{'binary':26} {'cases':>6} {'run':>6} {'pass':>6} {'known':>6} {'stale':>6} {'st-ign':>6} {'regr':>5} {'hang':>5} {'crash':>5} {'slowr':>5} {'not run':>8} {'slow':>5} {'pass %':>7}  top reasons not run")
    total = collections.Counter()
    for b in sorted(by):
        c = collections.Counter()
        why = collections.Counter()
        for t in by[b]:
            c["cases"] += 1
            c[t["class"]] += 1
            if t["class"] == "ignored":
                why[bucket(t.get("ignore") or "")] += 1
            else:
                c["run"] += 1
            if t["duration"] > slow:
                c["slow"] += 1
            if t.get("slow_regression"):
                c["SLOWR"] += 1
        total.update(c)
        top = ", ".join(f"{k} {v}" for k, v in why.most_common(50 if detail else 3))
        print(f"{b:26} {c['cases']:6} {c['run']:6} {c['pass']:6} {c['known']:6} {c['STALE']:6} {c['STALE-IGNORE']:6} {c['REGRESSION']:5} {c['HANG']:5} {c['CRASH']:5} {c['SLOWR']:5} {c['ignored']:8} {c['slow']:5} {rate(c):>7}  {top}")
    print(f"{'TOTAL':26} {total['cases']:6} {total['run']:6} {total['pass']:6} {total['known']:6} {total['STALE']:6} {total['STALE-IGNORE']:6} {total['REGRESSION']:5} {total['HANG']:5} {total['CRASH']:5} {total['SLOWR']:5} {total['ignored']:8} {total['slow']:5} {rate(total):>7}")
    print("(pass % = (pass + stale + stale-ignore) / run; known failures, regressions, hangs and crashes count as failures)")
    for c in res.get("checks", []):
        print(f"\ncheck {'ok' if c['ok'] else 'FAILED'}: {c['name']}")
    for cls in ("STALE", "STALE-IGNORE", "REGRESSION", "HANG", "CRASH"):
        bad = [t for t in res["tests"] if t["class"] == cls]
        if bad:
            print(f"\n{cls}:")
            for t in bad:
                print(f"  {t['binary']} {t['name']}" + (f": {t['message']}" if t.get("message") and cls in ("REGRESSION", "HANG", "CRASH") else ""))
    regs = [t for t in res["tests"] if t.get("slow_regression")]
    if regs:
        print(f"\nSLOW-REGRESSION (baseline {res.get('baseline')}, load ratio {res.get('load_ratio')}):")
        for t in regs:
            print(f"  {t['binary']} {t['name']}: {t['duration']:.1f} s, baseline {t['slow_regression']['baseline']:.1f} s")


def static():
    print("STATIC (fallback): the suites' source only, nothing run (no tools/kernel_gate.sh results).")
    print("'pass?' means 'not ignored and not a known failure', not that the case passes.")
    print()
    total = collections.Counter()
    print(f"{'file':28} {'cases':>6} {'pass?':>6} {'known':>6} {'missing':>8}  top missing features")
    for path in sorted(glob.glob(os.path.join(tests, "*.rs"))):
        src = open(path).read()
        # Each test: the attributes before `fn`, and its body up to the next #[test] (at any
        # indentation: tests inside a `mod` count too).
        parts = re.split(r"\n[ \t]*#\[test\][ \t]*\n", src)
        if len(parts) < 2:
            continue
        c = collections.Counter()
        feats = collections.Counter()
        for p in parts[1:]:
            head = p.split("fn ", 1)[0]
            m = re.search(r'#\[ignore(?: = "((?:[^"\\]|\\.)*)")?\]', head)
            if m:
                b = bucket(m.group(1) or "")
                if b == "known failure":
                    c["known"] += 1
                else:
                    c["missing"] += 1
                    feats[b] += 1
            elif "a known failure now passes" in p or "known_failure(" in p:
                c["known"] += 1
            else:
                c["pass"] += 1
        n = sum(c.values())
        total.update(c)
        total["cases"] += n
        top = ", ".join(f"{k} {v}" for k, v in feats.most_common(3 if not detail else 50))
        print(f"{os.path.basename(path):28} {n:6} {c['pass']:6} {c['known']:6} {c['missing']:8}  {top}")
    print(f"{'TOTAL':28} {total['cases']:6} {total['pass']:6} {total['known']:6} {total['missing']:8}")


path = None if "--static" in args else results_path()
if path:
    measured(path)
else:
    static()
