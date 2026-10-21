#!/usr/bin/env python3
"""Runs every test binary of cadrs_kernel and classifies each case: the measured scoreboard and
the gate. Normally started by tools/kernel_gate.sh, which exports a clean tree first.

usage: tools/kernel_gate.py --tree DIR --out DIR [--commit SHA] [--only a,b] [--skip a,b]
                            [--hang S] [--binary-timeout S] [--slow S] [--no-build] [--known-only]
                            [--baseline FILE] [--write-baseline FILE]
       tools/kernel_gate.py --make-baseline RESULTS FILE

Builds the tree's tests (`cargo test --profile kernel -p cadrs_kernel --no-run`; the target dir
is $CARGO_TARGET_DIR), lists each binary's tests (libtest JSON, `-Z unstable-options`), and runs
them $RUST_TEST_THREADS at a time (3 by default): the ones that aren't ignored, and the ignored
ones of every kind but three, which are listed, not run:
  - the OCCT suites' cases waiting for a command (`needs …`, Tcl, nothing measured): their body
    is the DRAW script as comments, so they would pass without testing anything;
  - debugging tools (a bare `#[ignore]`) and tests that need an input from the environment
    (`replays …`, `needs a usd-brep checkout`, `needs the data file …`, and smlib_oracle's
    without $CADRS_SMLIB_ORACLE): they return early without it.
--known-only runs only the ignored known failures (`#[ignore = "known failure…"]`).

A test that runs longer than --hang seconds is killed as a hang, and the binary goes on with the
tests it hadn't finished; a binary that dies (a stack overflow, an abort) has the tests it was
running retried one at a time to find the one that crashes. Each case is classified:
  pass        a case that passes
  known       a known failure (tools/occt_known_failures.txt, tools/smlib_known_failures/) that
              still fails
  STALE       a known failure that now passes: take it off the list and regenerate the suite
  STALE-IGNORE  a test ignored with a reason (not a listed known failure, not "slow") that now
              passes: un-ignore it
  REGRESSION  a failure that isn't listed (or an ignored "slow" test that fails)
  HANG        killed after --hang seconds (or the binary's --binary-timeout ran out first)
  CRASH       killed its test binary
  ignored     not run (the ignore reason is kept)
and flagged slow above --slow seconds (60). With a baseline (--baseline FILE, else
<out>/baseline.json when it exists: the durations of an earlier run), a case is also a
SLOW-REGRESSION when it takes more than 3 times and more than 5 s longer than its baseline time
scaled by the run's load: the median ratio of this run's durations to the baseline's over the
cases of at least 1 s, when that is above 1. (libtest reports wall time only, per test; the tests share one process,
so a test's CPU time isn't measurable, and the median ratio stands in for the load.)
--write-baseline FILE writes this run's durations as the next baseline when the gate passes;
--make-baseline RESULTS FILE writes one from an earlier results.json.

Writes <out>/results.json (one record per case) and <out>/summary.txt (a table per suite, the
STALE, STALE-IGNORE, REGRESSION, HANG, CRASH and SLOW-REGRESSION lists, the slowest 20 cases),
and exits 1 when any of those lists isn't empty (2 on a setup error).

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

BAD = ("STALE", "STALE-IGNORE", "REGRESSION", "HANG", "CRASH")
# Ignore reasons of tests that need an input from the environment and return early without it.
INPUT_IGNORES = ("replays ", "needs a usd-brep checkout", "needs the data file")
# Binaries whose tests return early without a program an environment variable names: their
# ignored tests run only when it names one.
NEEDS_ENV = {"smlib_oracle": "CADRS_SMLIB_ORACLE"}
# SLOW-REGRESSION: more than RATIO times and more than MARGIN seconds over the scaled baseline.
SLOW_RATIO, SLOW_MARGIN = 3.0, 5.0
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


def ignored_kind(binary, why, known_only):
    """How an ignored test runs: "known" (an ignored known failure), "slow" (a case too slow for
    the default run), "waiting" (ignored with another reason: it fails today), or None (not run;
    see the module docs)."""
    if why.startswith("known failure"):
        return "known"
    if known_only or binary.startswith("occt_") or not why or why.startswith(INPUT_IGNORES):
        return None
    if binary in NEEDS_ENV and not os.path.isfile(os.environ.get(NEEDS_ENV[binary], "")):
        return None
    return "slow" if why == "slow" else "waiting"


def classify(rec, listed, kind):
    """The class of a test's result: `listed` for a generated known failure (it asserts that it
    still fails), `kind` the ignored_kind of an ignored test (None when it isn't ignored)."""
    o = rec["outcome"]
    if o == "ignored":
        return "ignored"
    if o == "hang":
        return "HANG"
    if o == "crash":
        return "CRASH"
    if listed:
        if o == "ok":
            return "known"
        return "STALE" if STALE_MARK in rec.get("stdout", "") else "known"
    if kind in ("known", "waiting"):
        # An ignored test asserts success: passing, it shouldn't be ignored any more.
        return "STALE-IGNORE" if o == "ok" else "known"
    return "pass" if o == "ok" else "REGRESSION"


def key(t):
    return "%s %s" % (t["binary"], t["name"])


def make_baseline(res):
    """{"binary name": seconds} of the cases a results file ran (hangs and crashes left out)."""
    return {
        "commit": res.get("commit", ""),
        "finished": res.get("finished", ""),
        "durations": {key(t): t["duration"] for t in res["tests"] if t["class"] not in ("ignored", "HANG", "CRASH")},
    }


def slow_regressions(tests, base):
    """Marks the cases much slower than their baseline (t["slow_regression"]); returns the
    run's load ratio: the median of duration / baseline over the cases of at least 1 s, and at
    least 1 (a run faster overall, from a lighter load or faster code, is compared with the
    baseline's own times, so a case that only kept its time isn't flagged)."""
    ratios = sorted(t["duration"] / base[key(t)] for t in tests if t["class"] not in ("ignored", "HANG", "CRASH") and base.get(key(t), 0.0) >= 1.0)
    load = max(1.0, ratios[len(ratios) // 2]) if len(ratios) >= 20 else 1.0
    for t in tests:
        b = base.get(key(t))
        if b is None or t["class"] == "ignored":
            continue
        expected = b * load
        if t["duration"] > SLOW_RATIO * expected and t["duration"] - expected > SLOW_MARGIN:
            t["slow_regression"] = {"baseline": b, "expected": round(expected, 1)}
    return load


def rate_of(c):
    good = c["pass"] + c["STALE"] + c["STALE-IGNORE"]
    return "%.1f" % (100.0 * good / c["run"]) if c["run"] else "-"


def summary(res):
    tests = res["tests"]
    slow = res["slow_limit"]
    out = []
    out.append("kernel gate: %s, tree %s, %s; hang limit %d s, %d test threads" % (res["commit"] or "?", res["tree"], res["finished"], res["hang_limit"], res["threads"]))
    out.append("")
    cols = ("suite", "cases", "run", "pass", "known", "STALE", "ST-IGN", "REGR", "HANG", "CRASH", "SLOWR", "ignored", "slow", "pass %", "wall s")
    row = "%-26s %6s %6s %6s %6s %6s %6s %6s %6s %6s %6s %8s %5s %7s %8s"
    out.append(row % cols)
    tot = dict.fromkeys(("cases", "run", "pass", "known", "STALE", "STALE-IGNORE", "REGRESSION", "HANG", "CRASH", "SLOWR", "ignored", "slow"), 0)
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
            if t.get("slow_regression"):
                c["SLOWR"] += 1
        for k in tot:
            tot[k] += c[k]
        wall = res["binaries"][b].get("wall", 0.0)
        tot_wall += wall
        out.append(row % (b, *[c[k] for k in tot], rate_of(c), "%.0f" % wall))
    out.append(row % ("TOTAL", *[tot[k] for k in tot], rate_of(tot), "%.0f" % tot_wall))
    out.append("(pass % = (pass + STALE + STALE-IGNORE) / run: known failures, regressions, hangs and crashes count as failures)")
    for cls in BAD:
        bad = [t for t in tests if t["class"] == cls]
        out.append("")
        out.append("%s (%d)%s" % (cls, len(bad), ":" if bad else ""))
        for t in bad:
            out.append("  %s %s (%.1f s)%s" % (t["binary"], t["name"], t["duration"], ": " + t["message"] if t.get("message") and cls != "STALE" else ""))
    regs = [t for t in tests if t.get("slow_regression")]
    out.append("")
    if res.get("baseline"):
        out.append("SLOW-REGRESSION (%d)%s  [baseline %s, this run's load ratio %.2f]" % (len(regs), ":" if regs else "", res["baseline"], res.get("load_ratio", 1.0)))
    else:
        out.append("SLOW-REGRESSION: no baseline")
    for t in sorted(regs, key=lambda t: -t["duration"]):
        sr = t["slow_regression"]
        out.append("  %s %s: %.1f s, baseline %.1f s (%.1f s at this run's load)" % (t["binary"], t["name"], t["duration"], sr["baseline"], sr["expected"]))
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
    ap.add_argument("--tree")
    ap.add_argument("--out")
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
    ap.add_argument("--known-only", action="store_true", help="of the ignored tests, run only the known failures")
    ap.add_argument("--baseline", default="", help="durations of an earlier run (default <out>/baseline.json)")
    ap.add_argument("--write-baseline", default="", help="write this run's durations here when the gate passes")
    ap.add_argument("--make-baseline", nargs=2, metavar=("RESULTS", "FILE"), help="write a baseline from a results.json and exit")
    a = ap.parse_args()
    if a.make_baseline:
        json.dump(make_baseline(json.load(open(a.make_baseline[0]))), open(a.make_baseline[1], "w"), indent=0, sort_keys=True)
        return
    if not a.tree or not a.out:
        ap.error("--tree and --out are required")
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
        kinds = {n: ignored_kind(b, why, a.known_only) for n, ign, why in listed if ign}
        run = [n for n, ign, why in listed if not ign or kinds[n]]
        errlog.write("==== %s\n" % b)
        errlog.flush()
        got = Runner(exes[b], crate, env, threads, a.hang, a.binary_timeout, errlog).run(run) if run else {}
        for n, ign, why in listed:
            r = got.get(n)
            if r is None:
                r = {"outcome": "ignored", "duration": 0.0, "stdout": ""}
                if n in run:
                    r = {"outcome": "crash", "duration": 0.0, "stdout": "no result from the test binary"}
            # A generated known failure: on a list (its wrapper asserts that it still fails).
            listed_known = n.split("::")[-1] in kn or why.startswith("known failure, slow (tools/")
            rec = {"binary": b, "name": n, "outcome": r["outcome"], "duration": round(r["duration"], 3), "known": listed_known, "ignore": why if ign else None}
            rec["class"] = classify(dict(r), listed_known, kinds.get(n))
            if rec["class"] in ("REGRESSION", "HANG", "CRASH", "known") and r["outcome"] != "ok":
                rec["message"] = short_message(r.get("stdout", ""))
            res["tests"].append(rec)
        wall = time.time() - t0
        res["binaries"][b] = {"wall": round(wall, 1), "executable": exes[b]}
        bad = [t for t in res["tests"] if t["binary"] == b and t["class"] in BAD]
        log("  %d listed, %d run, %d bad, %.0f s" % (len(listed), len(run), len(bad), wall))
        json.dump(res, open(path, "w"), indent=0)
    res["finished"] = datetime.datetime.now().isoformat(timespec="seconds")
    base_path = a.baseline or os.path.join(a.out, "baseline.json")
    if os.path.exists(base_path):
        base = json.load(open(base_path))
        res["baseline"] = "%s (%s)" % (base_path, base.get("commit") or "?")
        res["load_ratio"] = round(slow_regressions(res["tests"], base["durations"]), 3)
    elif a.baseline:
        log("no baseline at %s: no SLOW-REGRESSION check" % a.baseline)
    json.dump(res, open(path, "w"), indent=0)
    text = summary(res)
    open(os.path.join(a.out, "summary.txt"), "w").write(text)
    print(text)
    errors = any(i.get("error") for i in res["binaries"].values()) or not all(c["ok"] for c in checked)
    failed = errors or any(t["class"] in BAD or t.get("slow_regression") for t in res["tests"])
    if a.write_baseline and not failed:
        json.dump(make_baseline(res), open(a.write_baseline, "w"), indent=0, sort_keys=True)
        log("baseline written to %s" % a.write_baseline)
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
