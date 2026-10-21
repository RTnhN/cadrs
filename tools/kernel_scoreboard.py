#!/usr/bin/env python3
"""Scoreboard of the kernel's generated test suites (crates/cadrs_kernel/tests): for each file,
how many cases pass, how many are known failures (an `#[ignore = "known failure…"]` case, or one
asserting `is_err()` with "a known failure"), and why the other ignored cases don't run
("needs <command>", missing data file, other).

usage: tools/kernel_scoreboard.py [--detail]"""
import collections, glob, os, re, sys

here = os.path.dirname(os.path.abspath(__file__))
tests = os.path.join(here, "../crates/cadrs_kernel/tests")
detail = "--detail" in sys.argv

def bucket(reason):
    if reason.startswith("known failure"):
        return "known failure"
    if reason.startswith("needs the data file"):
        return "missing data file"
    m = re.match(r"needs (\S+)", reason)
    if m and not reason.startswith("needs Tcl"):
        return "needs " + m.group(1)
    return reason or "ignored"

total = collections.Counter()
print(f"{'file':28} {'cases':>6} {'pass':>6} {'known':>6} {'missing':>8}  top missing features")
for path in sorted(glob.glob(os.path.join(tests, "*.rs"))):
    src = open(path).read()
    # Each test: the attributes before `fn`, and its body up to the next #[test].
    parts = re.split(r"\n#\[test\]\n", src)
    if len(parts) < 2:
        continue
    c = collections.Counter()
    feats = collections.Counter()
    for p in parts[1:]:
        head = p.split("fn ", 1)[0]
        m = re.search(r'#\[ignore(?: = "((?:[^"\\]|\\.)*)")?\]', head)
        body = p.split("\n#[test]", 1)[0]
        if m:
            b = bucket(m.group(1) or "")
            if b == "known failure":
                c["known"] += 1
            else:
                c["missing"] += 1
                feats[b] += 1
        elif "a known failure now passes" in body or "known_failure(" in body:
            c["known"] += 1
        else:
            c["pass"] += 1
    n = sum(c.values())
    total.update(c)
    total["cases"] += n
    top = ", ".join(f"{k} {v}" for k, v in feats.most_common(3 if not detail else 50))
    print(f"{os.path.basename(path):28} {n:6} {c['pass']:6} {c['known']:6} {c['missing']:8}  {top}")
print(f"{'TOTAL':28} {total['cases']:6} {total['pass']:6} {total['known']:6} {total['missing']:8}")
