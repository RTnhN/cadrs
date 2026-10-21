#!/usr/bin/env python3
"""Runs every test binary of cadrs_kernel and classifies each case: the measured scoreboard and
the gate. Normally started by tools/kernel_gate.sh, which exports a clean tree first.

usage: tools/kernel_gate.py --tree DIR --out DIR [--commit SHA] [--only a,b] [--skip a,b]
                            [--hang S] [--binary-timeout S] [--slow S] [--no-build]

Builds the tree's tests (`cargo test --profile kernel -p cadrs_kernel --no-run`; the target dir
is $CARGO_TARGET_DIR), lists each binary's tests (libtest JSON, `-Z unstable-options`), and runs
the ones that aren't ignored plus the ignored known failures (`#[ignore = "known failure…"]`, the
slow ones), $RUST_TEST_THREADS at a time (3 by default). Other ignored tests (cases waiting for a
command, debugging tools) are listed, not run.

A test that runs longer than --hang seconds is killed as a hang, and the binary goes on with the
tests it hadn't finished; a binary that dies (a stack overflow, an abort) has the tests it was
running retried one at a time to find the one that crashes. Each case is classified:
  pass        a case that passes
  known       a known failure (tools/occt_known_failures.txt, tools/smlib_known_failures/) that
              still fails
  STALE       a known failure that now passes: take it off the list and regenerate the suite
  REGRESSION  a failure that isn't listed
  HANG        killed after --hang seconds (or the binary's --binary-timeout ran out first)
  CRASH       killed its test binary
  ignored     not run (the ignore reason is kept)
and flagged slow above --slow seconds (60). Writes <out>/results.json (one record per case) and
<out>/summary.txt (a table per suite, the STALE, REGRESSION, HANG and CRASH lists, the slowest 20
cases), and exits 1 when anything is STALE, REGRESSION, HANG or CRASH (2 on a setup error).

First it runs the CHECKS (`cargo check -p cadrs_kernel` with no features, so the library builds
without the tests' `testkit` feature; `cargo check -p cadrs_core`, its real consumer); a failed
check also exits 1 (--no-checks skips them).
"""
import argparse
import datetime
import json
import os
import queue
import signal
import subprocess
import sys
import threading
import time

BAD = ("STALE", "REGRESSION", "HANG", "CRASH")
CURRENT = []  # the running test binary, killed with the gate


def stop(signum, _frame):
    for p in CURRENT:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except OSError:
            pass
    sys.exit(128 + signum)
STALE_MARK = "a known failure now passes"


def log(msg):
    print(msg, file=sys.stderr, flush=True)


# Builds that must pass besides the tests: the library without the test-only `testkit` feature
# (every test enables it, so no test sees code that only compiles with it), and cadrs_core, the
# kernel's real consumer.
CHECKS = [
    ("cargo check -p cadrs_kernel (no features)", ["cargo", "check", "-p", "cadrs_kernel"]),
    ("cargo check -p cadrs_core", ["cargo", "check", "-p", "cadrs_core"]),
]


def checks(tree):
    """[{name, ok, output}] of CHECKS run in the tree."""
    out = []
    for name, cmd in CHECKS:
        log("check: " + name)
        r = subprocess.run(cmd, cwd=tree, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        errs, in_error = [], False
        for l in r.stdout.splitlines():
            if l.startswith(("error", "warning")):
                in_error = l.startswith("error")
            if in_error and (l.startswith("error") or l.lstrip().startswith("-->")):
                errs.append(l)
        out.append({"name": name, "ok": r.returncode == 0, "output": "\n".join(errs[:20])})
        if r.returncode != 0:
            log("  FAILED:\n" + "\n".join(errs[:20]))
    return out


def build(tree, profile):
    """{binary name: executable} of the kernel's test targets ("lib" for the unit tests)."""
    cmd = ["cargo", "test", "--profile", profile, "-p", "cadrs_kernel", "--no-run", "--message-format=json-render-diagnostics"]
    log("build: " + " ".join(cmd))
    r = subprocess.run(cmd, cwd=tree, stdout=subprocess.PIPE, text=True)
    if r.returncode != 0:
        log("build failed (exit %d)" % r.returncode)
        sys.exit(2)
    out = {}
    for line in r.stdout.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") != "compiler-artifact" or not m.get("executable") or not m.get("profile", {}).get("test"):
            continue
        t = m["target"]
        if not t["src_path"].startswith(os.path.join(os.path.abspath(tree), "crates/cadrs_kernel")):
            continue
        out["lib" if "lib" in t["kind"] else t["name"]] = m["executable"]
    return out


def known_lists(tree):
    """{binary: {test names}} of the known-failure lists in the tree."""
    known = {}
    p = os.path.join(tree, "tools/occt_known_failures.txt")
    if os.path.exists(p):
        for line in open(p):
            w = line.split()
            if len(w) >= 2 and not w[0].startswith("#"):
                known.setdefault("occt_" + w[0], set()).add(w[1])
    d = os.path.join(tree, "tools/smlib_known_failures")
    if os.path.isdir(d):
        for f in sorted(os.listdir(d)):
            if f.endswith(".txt"):
                for line in open(os.path.join(d, f)):
                    w = line.split()
                    if w and not w[0].startswith("#"):
                        known.setdefault("smlib_" + f[:-4], set()).add(w[0])
    return known


def discover(exe, cwd, env):
    """[(name, ignored, ignore message)] of a test binary."""
    r = subprocess.run([exe, "--list", "-Z", "unstable-options", "--format", "json"], cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, timeout=300)
    tests = []
    for line in r.stdout.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("type") == "test" and m.get("event") == "discovered":
            tests.append((m["name"], bool(m.get("ignore")), m.get("ignore_message") or ""))
    if r.returncode != 0:
        raise RuntimeError("listing failed (exit %d): %s" % (r.returncode, r.stderr.strip()[-300:]))
    return tests


def short_message(stdout):
    """The panic message of a failed test's output (its first lines after "panicked at")."""
    lines = (stdout or "").splitlines()
    for i, l in enumerate(lines):
        if "panicked at" in l:
            return " ".join(x.strip() for x in lines[i + 1:i + 3] if x.strip())[:400]
    return " ".join(lines[-3:])[:400]


class Runner:
    """Runs the selected tests of one binary, surviving hangs and crashes."""

    def __init__(self, exe, cwd, env, threads, hang, budget, errlog):
        self.exe, self.cwd, self.env, self.threads = exe, cwd, env, threads
        self.hang, self.deadline, self.errlog = hang, time.time() + budget, errlog
        self.results = {}  # name -> dict(outcome, duration, stdout)

    def run(self, names):
        batches = [(sorted(names), self.threads)]
        while batches:
            batch, threads = batches.pop(0)
            todo = [n for n in batch if n not in self.results]
            if not todo:
                continue
            if time.time() >= self.deadline:
                for n in todo:
                    self.results[n] = {"outcome": "hang", "duration": 0.0, "stdout": "not run: the binary's time budget ran out"}
                continue
            before = len(self.results)
            status, crashed = self.batch(todo, threads)
            if status == "done":
                continue
            if status == "killed":
                # A hang: the tests that were running with it, and the ones not reached, go on.
                batches.append((todo, threads))
                continue
            if not crashed and len(self.results) == before:
                # Died before running anything: no test to blame but the whole batch.
                for n in todo:
                    self.results[n] = {"outcome": "crash", "duration": 0.0, "stdout": "the test binary died before running it"}
                continue
            # The binary died with `crashed` running: alone, a crashing test is found; the
            # rest of the batch goes on after them.
            if len(crashed) == 1 or threads == 1:
                for n in crashed:
                    self.results[n] = {"outcome": "crash", "duration": 0.0, "stdout": "the test binary died while it ran"}
            else:
                batches[:0] = [([n], 1) for n in crashed]
            batches.append((todo, threads))
        return self.results

    def batch(self, names, threads):
        """Runs `names`: ("done", []) when the binary finished, ("killed", []) when it was
        killed for a hang, ("died", tests that were running) when it died."""
        cmd = [self.exe, "-Z", "unstable-options", "--format", "json", "--report-time", "--include-ignored", "--exact", "--test-threads", str(threads)] + names
        p = subprocess.Popen(cmd, cwd=self.cwd, env=self.env, stdout=subprocess.PIPE, stderr=self.errlog, text=True, start_new_session=True)
        CURRENT[:] = [p]
        lines = queue.Queue()

        def pump():
            for line in p.stdout:
                lines.put(line)
            lines.put(None)

        threading.Thread(target=pump, daemon=True).start()
        running = {}  # name -> start time
        done_suite = False
        killed = False
        while True:
            try:
                line = lines.get(timeout=1.0)
            except queue.Empty:
                line = ""
            if line is None:
                break
            now = time.time()
            if line:
                try:
                    m = json.loads(line)
                except ValueError:
                    m = None
                if m and m.get("type") == "test":
                    ev, n = m.get("event"), m.get("name")
                    if ev == "started":
                        running[n] = now
                    elif ev in ("ok", "failed", "ignored", "timeout"):
                        t0 = running.pop(n, now)
                        self.results[n] = {"outcome": "ok" if ev == "ok" else "failed", "duration": m.get("exec_time", now - t0), "stdout": m.get("stdout", "")}
                elif m and m.get("type") == "suite" and m.get("event") in ("ok", "failed"):
                    done_suite = True
            over = [n for n, t0 in running.items() if now - t0 > self.hang]
            if (over or now >= self.deadline) and not killed:
                for n in over or list(running):
                    self.results[n] = {"outcome": "hang", "duration": now - running[n], "stdout": "killed after %.0f s" % (now - running[n])}
                os.killpg(p.pid, signal.SIGKILL)
                killed = True
        p.wait()
        CURRENT.clear()
        if killed:
            return "killed", []
        if done_suite:
            return "done", []
        return "died", sorted(running)


def classify(rec, known):
    o = rec["outcome"]
    if o == "ignored":
        return "ignored"
    if o == "hang":
        return "HANG"
    if o == "crash":
        return "CRASH"
    if o == "ok":
        return "known" if known else "pass"
    if STALE_MARK in rec.get("stdout", ""):
        return "STALE"
    return "known" if known else "REGRESSION"


def summary(res):
    tests = res["tests"]
    slow = res["slow_limit"]
    out = []
    out.append("kernel gate: %s, tree %s, %s; hang limit %d s, %d test threads" % (res["commit"] or "?", res["tree"], res["finished"], res["hang_limit"], res["threads"]))
    out.append("")
    cols = ("suite", "cases", "run", "pass", "known", "STALE", "REGR", "HANG", "CRASH", "ignored", "slow", "pass %", "wall s")
    out.append("%-26s %6s %6s %6s %6s %6s %6s %6s %6s %8s %5s %7s %8s" % cols)
    tot = dict.fromkeys(("cases", "run", "pass", "known", "STALE", "REGRESSION", "HANG", "CRASH", "ignored", "slow"), 0)
    tot_wall = 0.0
    for b in sorted(res["binaries"]):
        ts = [t for t in tests if t["binary"] == b]
        c = dict.fromkeys(tot, 0)
        c["cases"] = len(ts)
        for t in ts:
            c[t["class"]] += 1
            if t["class"] != "ignored":
                c["run"] += 1
            if t["duration"] > slow:
                c["slow"] += 1
        for k in tot:
            tot[k] += c[k]
        wall = res["binaries"][b].get("wall", 0.0)
        tot_wall += wall
        rate = "%.1f" % (100.0 * (c["pass"] + c["STALE"]) / c["run"]) if c["run"] else "-"
        out.append("%-26s %6d %6d %6d %6d %6d %6d %6d %6d %8d %5d %7s %8.0f" % (b, c["cases"], c["run"], c["pass"], c["known"], c["STALE"], c["REGRESSION"], c["HANG"], c["CRASH"], c["ignored"], c["slow"], rate, wall))
    rate = "%.1f" % (100.0 * (tot["pass"] + tot["STALE"]) / tot["run"]) if tot["run"] else "-"
    out.append("%-26s %6d %6d %6d %6d %6d %6d %6d %6d %8d %5d %7s %8.0f" % ("TOTAL", tot["cases"], tot["run"], tot["pass"], tot["known"], tot["STALE"], tot["REGRESSION"], tot["HANG"], tot["CRASH"], tot["ignored"], tot["slow"], rate, tot_wall))
    out.append("(pass % = (pass + STALE) / run: known failures, regressions, hangs and crashes count as failures)")
    for cls in BAD:
        bad = [t for t in tests if t["class"] == cls]
        out.append("")
        out.append("%s (%d)%s" % (cls, len(bad), ":" if bad else ""))
        for t in bad:
            out.append("  %s %s (%.1f s)%s" % (t["binary"], t["name"], t["duration"], ": " + t["message"] if t.get("message") and cls != "STALE" else ""))
    for c in res.get("checks", []):
        out.append("")
        out.append("CHECK %s: %s" % ("ok" if c["ok"] else "FAILED", c["name"]))
        if not c["ok"] and c["output"]:
            out.append("  " + c["output"].replace("\n", "\n  "))
    for b, info in sorted(res["binaries"].items()):
        if info.get("error"):
            out.append("")
            out.append("ERROR %s: %s" % (b, info["error"]))
    out.append("")
    out.append("slowest 20 cases:")
    for t in sorted((t for t in tests if t["class"] != "ignored"), key=lambda t: -t["duration"])[:20]:
        out.append("  %8.1f s  %-10s %s %s" % (t["duration"], t["class"], t["binary"], t["name"]))
    return "\n".join(out) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--tree", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--commit", default="")
    ap.add_argument("--profile", default="kernel")
    ap.add_argument("--only", default="", help="comma-separated binaries to run (default: all)")
    ap.add_argument("--skip", default="", help="comma-separated binaries to leave out")
    ap.add_argument("--hang", type=float, default=900.0, help="seconds after which a test is a hang")
    ap.add_argument("--binary-timeout", type=float, default=4 * 3600.0, help="seconds for a whole binary")
    ap.add_argument("--slow", type=float, default=60.0, help="seconds above which a case is slow")
    ap.add_argument("--no-build", action="store_true", help="use the binaries of --binaries")
    ap.add_argument("--no-checks", action="store_true", help="skip the `cargo check` builds")
    ap.add_argument("--binaries", default="", help="JSON {name: executable} (with --no-build)")
    a = ap.parse_args()
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)

    tree = os.path.abspath(a.tree)
    crate = os.path.join(tree, "crates/cadrs_kernel")
    os.makedirs(a.out, exist_ok=True)
    threads = int(os.environ.get("RUST_TEST_THREADS", "3"))
    checked = [] if a.no_checks else checks(tree)
    if a.no_build:
        exes = json.load(open(a.binaries))
    else:
        exes = build(tree, a.profile)
        json.dump(exes, open(os.path.join(a.out, "binaries.json"), "w"), indent=1)
    only = set(filter(None, a.only.split(",")))
    skip = set(filter(None, a.skip.split(",")))
    names = sorted(b for b in exes if (not only or b in only) and b not in skip)
    if only - set(exes):
        log("no such test binary: " + ", ".join(sorted(only - set(exes))))
        sys.exit(2)
    known = known_lists(tree)
    env = dict(os.environ, RUST_TEST_THREADS=str(threads), RUST_BACKTRACE="0")
    res = {
        "schema": 1,
        "commit": a.commit,
        "tree": tree,
        "started": datetime.datetime.now().isoformat(timespec="seconds"),
        "finished": "",
        "hang_limit": a.hang,
        "slow_limit": a.slow,
        "threads": threads,
        "binaries": {},
        "checks": checked,
        "tests": [],
    }
    path = os.path.join(a.out, "results.json")
    errlog = open(os.path.join(a.out, "stderr.log"), "w")
    for b in names:
        t0 = time.time()
        log("%s ..." % b)
        try:
            listed = discover(exes[b], crate, env)
        except (RuntimeError, subprocess.TimeoutExpired) as e:
            res["binaries"][b] = {"wall": 0.0, "error": str(e)}
            log("  %s" % e)
            continue
        kn = known.get(b, set())
        run = [n for n, ign, why in listed if not ign or why.startswith("known failure")]
        errlog.write("==== %s\n" % b)
        errlog.flush()
        got = Runner(exes[b], crate, env, threads, a.hang, a.binary_timeout, errlog).run(run) if run else {}
        for n, ign, why in listed:
            r = got.get(n)
            if r is None:
                r = {"outcome": "ignored", "duration": 0.0, "stdout": ""}
                if n in run:
                    r = {"outcome": "crash", "duration": 0.0, "stdout": "no result from the test binary"}
            is_known = n.split("::")[-1] in kn or why.startswith("known failure")
            rec = {"binary": b, "name": n, "outcome": r["outcome"], "duration": round(r["duration"], 3), "known": is_known, "ignore": why if ign else None}
            rec["class"] = classify(dict(r), is_known)
            if rec["class"] in ("REGRESSION", "HANG", "CRASH", "known") and r["outcome"] != "ok":
                rec["message"] = short_message(r.get("stdout", ""))
            res["tests"].append(rec)
        wall = time.time() - t0
        res["binaries"][b] = {"wall": round(wall, 1), "executable": exes[b]}
        bad = [t for t in res["tests"] if t["binary"] == b and t["class"] in BAD]
        log("  %d listed, %d run, %d bad, %.0f s" % (len(listed), len(run), len(bad), wall))
        json.dump(res, open(path, "w"), indent=0)
    res["finished"] = datetime.datetime.now().isoformat(timespec="seconds")
    json.dump(res, open(path, "w"), indent=0)
    text = summary(res)
    open(os.path.join(a.out, "summary.txt"), "w").write(text)
    print(text)
    errors = any(i.get("error") for i in res["binaries"].values()) or not all(c["ok"] for c in checked)
    sys.exit(1 if errors or any(t["class"] in BAD for t in res["tests"]) else 0)


if __name__ == "__main__":
    main()
